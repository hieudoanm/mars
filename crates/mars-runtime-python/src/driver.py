"""Project Mars Python kernel driver.

Speaks newline-delimited JSON on stdin/stdout. Cell sources execute in a
shared module namespace so variables persist across cells. User stdout and
stderr are captured and forwarded as ``stream`` messages; the value of a
trailing expression is forwarded as ``display`` data.

Request messages (one JSON object per line on stdin):

- ``{"type": "execute", "id": N, "source": "..."}``
- ``{"type": "inspect", "id": N}``
- ``{"type": "check", "id": N, "source": "..."}``
- ``{"type": "input_response", "text": "..."}``
- ``{"type": "shutdown"}``

Response messages (one JSON object per line on stdout):

- ``{"type": "stream", "id": N, "stream": "stdout"|"stderr", "data": "..."}``
- ``{"type": "display", "id": N, "data": [[mime, data], ...]}``
- ``{"type": "mime", "id": N, "data": {mime: data}, "metadata": {...}}``
- ``{"type": "input_request", "id": N, "prompt": "..."}``
- ``{"type": "result", "id": N, "ok": bool, "error": {...} | null}``
- ``{"type": "inspection", "id": N, "variables": [...]}``
- ``{"type": "check_result", "id": N, "error": {...} | null}``
"""

import ast
import asyncio
import base64
import inspect as _inspect
import io
import json
import os
import queue
import shlex
import subprocess
import sys
import threading
import traceback

sys.path.insert(0, os.getcwd())

REAL_STDOUT = sys.__stdout__
OUTBOX = queue.Queue()
REQUESTS = queue.Queue()
PENDING_INPUT = queue.Queue()
NAMESPACE = {}
SHOWN_FIGURES = set()
CURRENT_ID = {"id": None}
LOOP = None
HIDDEN_NAMES = {"input", "display"}


class Channel:
    """File-like object that forwards writes to the protocol stream."""

    __slots__ = ("name",)

    def __init__(self, name):
        self.name = name

    def write(self, text):
        if text:
            emit({"type": "stream", "stream": self.name, "data": text})
        return len(text)

    def flush(self):
        pass


def emit(message):
    if "id" not in message:
        message["id"] = CURRENT_ID["id"]
    OUTBOX.put(message)


def protocol_writer():
    while True:
        message = OUTBOX.get()
        if message is None:
            return
        REAL_STDOUT.write(json.dumps(message) + "\n")
        REAL_STDOUT.flush()


def get_loop():
    global LOOP
    if LOOP is None or LOOP.is_closed():
        LOOP = asyncio.new_event_loop()
        asyncio.set_event_loop(LOOP)
    return LOOP


def needs_tla(tree):
    return any(
        isinstance(node, (ast.Await, ast.AsyncFor, ast.AsyncWith))
        for node in ast.walk(tree)
    )


def compile_tree(tree, filename, mode):
    flags = ast.PyCF_ALLOW_TOP_LEVEL_AWAIT if needs_tla(tree) else 0
    return compile(tree, filename, mode, flags=flags)


def run_code(code):
    """Execute a compiled code object, driving top-level await if needed."""
    result = eval(code, NAMESPACE)
    if _inspect.isawaitable(result):
        result = get_loop().run_until_complete(result)
    return result


def split_source(source):
    """Split ``source`` into (module tree, trailing expression tree)."""
    tree = ast.parse(source, "<cell>", "exec")
    body = tree.body
    if body and isinstance(body[-1], ast.Expr):
        module_tree = ast.Module(body=body[:-1], type_ignores=[])
        return module_tree, ast.Expression(body[-1].value)
    return tree, None


def display_value(value):
    if value is None:
        return
    bundle, metadata = mime_bundle_for(value)
    parts = []
    extra = {}
    for mime, data in bundle.items():
        if isinstance(data, str):
            parts.append([mime, data])
        else:
            extra[mime] = data
    if parts:
        emit({"type": "display", "data": parts})
    # Non-string mime payloads and formatter metadata cannot ride the
    # display-pairs channel; forward them as a full mime bundle instead.
    if extra or metadata:
        emit({"type": "mime", "data": extra, "metadata": metadata or {}})
    NAMESPACE["Out"] = value
    NAMESPACE["_"] = value


def flush_figures():
    pyplot = sys.modules.get("matplotlib.pyplot")
    if pyplot is None:
        return
    try:
        numbers = list(pyplot.get_fignums())
    except Exception:
        return
    for number in numbers:
        try:
            figure = pyplot.figure(number)
        except Exception:
            continue
        if id(figure) in SHOWN_FIGURES:
            continue
        SHOWN_FIGURES.add(id(figure))
        buffer = io.BytesIO()
        try:
            figure.savefig(buffer, format="png", bbox_inches="tight")
        except Exception:
            continue
        encoded = base64.b64encode(buffer.getvalue()).decode("ascii")
        emit({"type": "display", "data": [["image/png", encoded]]})


def encode_image_data(data):
    """Normalise ``_repr_png_``/``_repr_jpeg_`` payloads to base64 text.

    The IPython contract is ``bytes``; a ``str`` is treated as already
    base64-encoded (a common user mistake) rather than crashing the cell.
    """
    if isinstance(data, str):
        return data
    if isinstance(data, (bytes, bytearray, memoryview)):
        return base64.b64encode(bytes(data)).decode("ascii")
    return None


def mime_bundle_for(value):
    """Build a {mime: data} mapping for ``value``, Jupyter-style."""
    bundle = {}
    mimebundle_fn = getattr(value, "_repr_mimebundle_", None)
    if mimebundle_fn is not None:
        try:
            result = mimebundle_fn()
        except Exception:
            result = None
        if isinstance(result, tuple) and result:
            data, metadata = result[0], (result[1] if len(result) > 1 else {})
        elif isinstance(result, dict):
            data, metadata = result, {}
        else:
            data, metadata = {}, {}
        if isinstance(data, dict):
            bundle.update(data)
            if isinstance(metadata, dict) and metadata:
                return bundle, metadata
    for method, mime in (
        ("_repr_html_", "text/html"),
        ("_repr_png_", "image/png"),
        ("_repr_jpeg_", "image/jpeg"),
        ("_repr_svg_", "image/svg+xml"),
    ):
        method_fn = getattr(value, method, None)
        if method_fn is None:
            continue
        try:
            data = method_fn()
        except Exception:
            continue
        if not data:
            continue
        if mime in ("image/png", "image/jpeg"):
            data = encode_image_data(data)
            if data is None:
                continue
        bundle[mime] = data
    # A ``_repr_mimebundle_`` may supply its own ``text/plain``; only fall
    # back to ``repr`` when nothing provided one.
    if "text/plain" not in bundle:
        try:
            bundle["text/plain"] = repr(value)
        except Exception:
            bundle["text/plain"] = "<unrepresentable>"
    return bundle, {}


def mars_display(*objs, raw=False, include=None, exclude=None, metadata=None, **_kwargs):
    """IPython-style display(): publishes mime bundles for each object."""
    wanted = set(include) if include else None
    blocked = set(exclude) if exclude else set()
    for obj in objs:
        if raw and isinstance(obj, dict):
            bundle = {
                mime: data
                for mime, data in obj.items()
                if isinstance(mime, str)
            }
            emit({"type": "mime", "data": bundle, "metadata": metadata or {}})
            continue
        bundle, obj_metadata = mime_bundle_for(obj)
        merged_metadata = dict(obj_metadata)
        if isinstance(metadata, dict):
            merged_metadata.update(metadata)
        if wanted is not None:
            bundle = {
                mime: data
                for mime, data in bundle.items()
                if any(mime == prefix or mime.startswith(prefix)
                       for prefix in wanted)
            }
        if blocked:
            bundle = {
                mime: data
                for mime, data in bundle.items()
                if not any(mime == prefix or mime.startswith(prefix)
                           for prefix in blocked)
            }
        if bundle:
            emit({"type": "mime", "data": bundle, "metadata": merged_metadata})






def setup_comms():
    """Bridge the ``comm`` package to Project Mars's protocol so widget libraries
    (ipywidgets et al.) get real front-end comms instead of DummyComms.

    Outbound comm traffic is emitted as ``{"type": "comm", ...}`` lines;
    inbound ``comm_msg``/``comm_close`` requests are dispatched to the comm
    manager and acknowledged with ``comm_ack``.
    """
    try:
        import comm as _comm_pkg
        from comm.base_comm import BaseComm
    except Exception:
        return

    class MarsComm(BaseComm):
        def publish_msg(self, msg_type, data=None, metadata=None,
                        buffers=None, **keys):
            payload = {
                "type": "comm",
                "comm_type": msg_type,
                "comm_id": self.comm_id,
                "target_name": self.target_name,
            }
            if data is not None:
                payload["data"] = data
            if metadata:
                payload["metadata"] = metadata
            for key in ("target_module",):
                value = keys.get(key, getattr(self, key, None))
                if value is not None:
                    payload[key] = value
            emit(payload)

    def mars_create_comm(*args, **kwargs):
        return MarsComm(*args, **kwargs)

    _comm_pkg.create_comm = mars_create_comm
    # ipywidgets resolves comms through ipywidgets.comm -> comm.create_comm,
    # which we just replaced; also cover the manager lookup used by
    # comm.open_data-style helpers.
    try:
        import ipywidgets.comm as _ipyw_comm
        _ipyw_comm.create_comm = mars_create_comm
    except Exception:
        pass


def dispatch_comm(request):
    """Route an inbound comm_msg/comm_close to the registered comm."""
    try:
        import comm as _comm_pkg
    except Exception:
        return
    manager = _comm_pkg.get_comm_manager()
    if manager is None:
        return
    comm_id = request.get("comm_id")
    comm_obj = manager.get_comm(comm_id) if comm_id else None
    if comm_obj is None:
        return
    content = {
        "comm_id": comm_id,
        "data": request.get("data") or {},
    }
    msg = {"header": {}, "content": content, "buffers": []}
    try:
        if request.get("type") == "comm_close":
            comm_obj.handle_close(msg)
        else:
            comm_obj.handle_msg(msg)
    except Exception:
        pass

def patch_ipython_display():
    """Route ``IPython.display.display`` to our builtin when IPython is
    importable, so ``from IPython.display import display`` and
    ``ipd.display(...)`` publish mime bundles instead of printing reprs.
    """
    try:
        import IPython.display as ipython_display
    except Exception:
        return
    try:
        ipython_display.display = mars_display
    except Exception:
        pass

def mars_input(prompt=""):
    """input() backed by the kernel's stdin-response channel."""
    prompt_text = "" if prompt is None else str(prompt)
    if prompt_text:
        emit({"type": "stream", "stream": "stdout", "data": prompt_text})
    emit({"type": "input_request", "prompt": prompt_text})
    response = PENDING_INPUT.get()
    if response is None:
        raise EOFError("kernel input channel closed")
    emit({"type": "stream", "stream": "stdout", "data": response + "\n"})
    return response


def run_shell(source):
    """Execute a ``!command`` or ``%pip`` line, forwarding output."""
    stripped = source.lstrip()
    if stripped.startswith("!"):
        command = stripped[1:]
        completed = subprocess.run(
            command, shell=True, capture_output=True, text=True
        )
        if completed.stdout:
            sys.stdout.write(completed.stdout)
            sys.stdout.flush()
        if completed.stderr:
            sys.stderr.write(completed.stderr)
            sys.stderr.flush()
        return completed.returncode == 0
    if stripped.startswith("%pip") or stripped.startswith("%pip3"):
        try:
            args = shlex.split(stripped.split(None, 1)[1] if len(stripped.split(None, 1)) > 1 else "")
        except ValueError:
            sys.stderr.write("mars: could not parse pip arguments\n")
            return False
        completed = subprocess.run(
            [sys.executable, "-m", "pip", *args],
            capture_output=True,
            text=True,
        )
        if completed.stdout:
            sys.stdout.write(completed.stdout)
            sys.stdout.flush()
        if completed.stderr:
            sys.stderr.write(completed.stderr)
            sys.stderr.flush()
        return completed.returncode == 0
    return False


def handle_execute(request):
    request_id = request.get("id")
    source = request.get("source", "")
    CURRENT_ID["id"] = request_id
    error = None
    previous = (sys.stdout, sys.stderr, sys.stdin)
    sys.stdout = Channel("stdout")
    sys.stderr = Channel("stderr")
    # Keep protocol lines on stdin unreachable to user code: input() answers
    # through the input_request/input_response channel instead.
    sys.stdin = io.StringIO("")
    try:
        stripped = source.lstrip()
        if stripped.startswith("!") or stripped.startswith("%pip") or stripped.startswith("%pip3"):
            ok = run_shell(source)
            if not ok:
                error = {
                    "name": "ShellError",
                    "message": "shell command failed",
                    "traceback": [],
                }
        else:
            if stripped.startswith("%"):
                raise RuntimeError(f"unsupported IPython magic: {stripped.splitlines()[0]!r}")
            module_tree, expression_tree = split_source(source)
            module_code = compile_tree(module_tree, "<cell>", "exec")
            run_code(module_code)
            if expression_tree is not None:
                expression_code = compile_tree(expression_tree, "<cell>", "eval")
                value = run_code(expression_code)
                display_value(value)
            flush_figures()
    except BaseException as exc:  # noqa: BLE001 - report every failure kind
        error = {
            "name": type(exc).__name__,
            "message": str(exc),
            "traceback": traceback.format_exception(exc),
        }
    finally:
        sys.stdout, sys.stderr, sys.stdin = previous
        CURRENT_ID["id"] = None
    emit({"type": "result", "id": request_id, "ok": error is None, "error": error})


def handle_inspect(request):
    request_id = request.get("id")
    CURRENT_ID["id"] = request_id
    variables = []
    for name in sorted(NAMESPACE):
        if name.startswith("_") or name in HIDDEN_NAMES:
            continue
        value = NAMESPACE[name]
        try:
            text = repr(value)
            preview = text if len(text) <= 160 else text[:157] + "..."
        except Exception:
            preview = None
        variables.append(
            {"name": name, "type": type(value).__name__, "value": preview}
        )
    CURRENT_ID["id"] = None
    emit({"type": "inspection", "id": request_id, "variables": variables})


def handle_check(request):
    request_id = request.get("id")
    source = request.get("source", "")
    CURRENT_ID["id"] = request_id
    try:
        compile(source, "<cell>", "exec")
        error = None
    except SyntaxError as exc:
        error = {
            "name": "SyntaxError",
            "message": str(exc),
            "lineno": exc.lineno or 1,
            "offset": exc.offset or 1,
        }
    except Exception as exc:  # noqa: BLE001 - e.g. source containing null bytes
        error = {
            "name": type(exc).__name__,
            "message": str(exc),
            "lineno": 1,
            "offset": 1,
        }
    CURRENT_ID["id"] = None
    emit({"type": "check_result", "id": request_id, "error": error})


def stdin_reader():
    """Reads protocol lines; work items queue for the exec loop while
    input responses go straight to the pending-input queue so a cell blocked
    in input() can be answered mid-execution."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except ValueError:
            continue
        request_type = request.get("type")
        if request_type == "input_response":
            PENDING_INPUT.put(request.get("text"))
        elif request_type in ("comm_msg", "comm_close"):
            dispatch_comm(request)
            request_id = request.get("id")
            if request_id is not None:
                emit({"type": "comm_ack", "id": request_id})
        elif request_type == "shutdown":
            REQUESTS.put(None)
            return
        else:
            REQUESTS.put(request)


def install_packages():
    """Install ``metadata.python.packages`` requirements before serving.

    Requirements arrive via ``MARS_PYTHON_PACKAGES`` (JSON list). Failures are
    reported on the stream channel but never abort the session.
    """
    raw = os.environ.get("MARS_PYTHON_PACKAGES")
    if not raw:
        return
    try:
        packages = json.loads(raw)
    except ValueError:
        return
    if not isinstance(packages, list) or not packages:
        return
    args = [sys.executable, "-m", "pip", "install", "--quiet"] + [
        str(package) for package in packages
    ]
    emit({"type": "stream", "stream": "stdout",
          "data": f"Installing Python packages: {', '.join(map(str, packages))}\n"})
    try:
        completed = subprocess.run(
            args, capture_output=True, text=True, timeout=600
        )
    except Exception as exc:  # noqa: BLE001
        emit({"type": "stream", "stream": "stderr",
              "data": f"package install failed: {exc}\n"})
        return
    if completed.returncode != 0:
        emit({"type": "stream", "stream": "stderr",
              "data": (completed.stderr or completed.stdout or "pip install failed")
              + "\n"})
    else:
        emit({"type": "stream", "stream": "stdout", "data": "Packages installed.\n"})


def main():
    install_packages()
    writer = threading.Thread(target=protocol_writer, daemon=True)
    writer.start()
    threading.Thread(target=stdin_reader, daemon=True).start()
    NAMESPACE["input"] = mars_input
    NAMESPACE["display"] = mars_display
    patch_ipython_display()
    setup_comms()
    while True:
        request = REQUESTS.get()
        if request is None:
            break
        request_type = request.get("type")
        if request_type == "execute":
            handle_execute(request)
        elif request_type == "inspect":
            handle_inspect(request)
        elif request_type == "check":
            handle_check(request)
    PENDING_INPUT.put(None)
    OUTBOX.put(None)
    writer.join(timeout=1)


if __name__ == "__main__":
    main()
