# Mars Julia kernel driver.
#
# Speaks newline-delimited JSON on stdin/stdout. Cell sources are parsed with
# Meta.parseall and evaluated in a dedicated module so bindings persist across
# cells. User stdout/stderr are captured and forwarded as ``stream`` messages;
# the value of a trailing visible expression is forwarded as ``display`` data
# built from MIME reprs, and display() calls are forwarded as ``mime`` bundles.
#
# Cell code runs in module MarsKernel, which shadows ``display`` and
# ``readline`` with Mars-aware versions backed by the kernel protocol.
#
# Request messages (one JSON object per line on stdin):
#
# - ``{"type": "execute", "id": N, "source": "..."}``
# - ``{"type": "inspect", "id": N}``
# - ``{"type": "check", "id": N, "source": "..."}``
# - ``{"type": "input_response", "text": "..."}``
# - ``{"type": "shutdown"}``
#
# Response messages (one JSON object per line on stdout):
#
# - ``{"type": "stream", "id": N, "stream": "stdout"|"stderr", "data": "..."}``
# - ``{"type": "display", "id": N, "data": [[mime, data], ...]}``
# - ``{"type": "mime", "id": N, "data": {mime: data}, "metadata": {...}}``
# - ``{"type": "input_request", "id": N, "prompt": "..."}``
# - ``{"type": "result", "id": N, "ok": bool, "error": {...} | null}``
# - ``{"type": "inspection", "id": N, "variables": [...]}``
# - ``{"type": "check_result", "id": N, "error": {...} | null}``
#
# Known limitations:
#
# - User code that reads ``stdin`` directly (e.g. ``read(stdin)``) consumes
#   kernel protocol lines.
# - ``readline(prompt; stream=..., limit=...)`` with keyword arguments falls
#   back to Base.readline and reads protocol stdin.
# - A timed-out or cancelled cell kills the driver; the next cell starts a
#   fresh Julia session and loses all state.

using Base64

const REAL_STDOUT = Base.stdout
const REQUESTS = Channel{Any}(Inf)
const PENDING_INPUT = Channel{Union{String, Nothing}}(Inf)
const CURRENT_ID = Ref{Any}(nothing)
const M = Module(:MarsKernel)

const DISPLAY_MIMES = (
    ("text/html", MIME"text/html"()),
    ("image/svg+xml", MIME"image/svg+xml"()),
    ("image/png", MIME"image/png"()),
    ("image/jpeg", MIME"image/jpeg"()),
    ("application/pdf", MIME"application/pdf"()),
    ("text/latex", MIME"text/latex"()),
)

const SILENT_HEADS = Set{Symbol}([
    :(=),
    :(function),
    :(macro),
    :(module),
    :(baremodule),
    :(import),
    :(using),
    :(export),
    :(global),
    :(local),
    :(const),
    :(struct),
    :(mutable),
    :(abstract),
    :(primitive),
    :(type),
])

# ---------------------------------------------------------------------------
# Minimal JSON writer
# ---------------------------------------------------------------------------

function json_write_string(io::IO, s::AbstractString)
    print(io, '"')
    for c in s
        if c == '"'
            print(io, "\\\"")
        elseif c == '\\'
            print(io, "\\\\")
        elseif c == '\n'
            print(io, "\\n")
        elseif c == '\r'
            print(io, "\\r")
        elseif c == '\t'
            print(io, "\\t")
        elseif c == '\b'
            print(io, "\\b")
        elseif c == '\f'
            print(io, "\\f")
        elseif UInt32(c) < 0x20
            print(io, "\\u", lpad(string(UInt32(c), base = 16), 4, '0'))
        else
            print(io, c)
        end
    end
    print(io, '"')
    return nothing
end

function json_write(io::IO, v)
    if v === nothing
        print(io, "null")
    elseif v isa Bool
        print(io, v ? "true" : "false")
    elseif v isa Integer
        print(io, v)
    elseif v isa AbstractFloat
        if isnan(v) || isinf(v)
            print(io, "null")
        else
            print(io, v)
        end
    elseif v isa AbstractString
        json_write_string(io, v)
    elseif v isa AbstractDict
        print(io, '{')
        first = true
        for (k, val) in v
            first || print(io, ',')
            first = false
            json_write_string(io, String(k))
            print(io, ':')
            json_write(io, val)
        end
        print(io, '}')
    elseif v isa Pair
        print(io, '{')
        json_write_string(io, String(v.first))
        print(io, ':')
        json_write(io, v.second)
        print(io, '}')
    elseif v isa Tuple || v isa AbstractVector
        print(io, '[')
        for (i, item) in enumerate(v)
            i == 1 || print(io, ',')
            json_write(io, item)
        end
        print(io, ']')
    else
        json_write_string(io, sprint(show, v))
    end
    return nothing
end

function json_dumps(v)
    io = IOBuffer()
    json_write(io, v)
    return String(take!(io))
end

# ---------------------------------------------------------------------------
# Minimal JSON parser (objects, arrays, strings, numbers, literals)
# ---------------------------------------------------------------------------

mutable struct JsonParser
    data::Vector{UInt8}
    pos::Int
end

JsonParser(s::AbstractString) = JsonParser(collect(codeunits(s)), 1)

function jskip(p::JsonParser)
    while p.pos <= length(p.data)
        b = p.data[p.pos]
        if b == 0x20 || b == 0x09 || b == 0x0a || b == 0x0d
            p.pos += 1
        else
            break
        end
    end
    return nothing
end

function jexpect(p::JsonParser, byte::UInt8)
    jskip(p)
    if p.pos <= length(p.data) && p.data[p.pos] == byte
        p.pos += 1
    else
        throw(ErrorException("invalid JSON"))
    end
    return nothing
end

function jliteral(p::JsonParser, lit::AbstractString)
    for b in codeunits(lit)
        if p.pos <= length(p.data) && p.data[p.pos] == b
            p.pos += 1
        else
            throw(ErrorException("invalid JSON literal"))
        end
    end
    return nothing
end

function jparse_hex4(p::JsonParser)
    value = 0
    for _ in 1:4
        p.pos <= length(p.data) || throw(ErrorException("invalid JSON unicode escape"))
        b = p.data[p.pos]
        p.pos += 1
        value <<= 4
        if 0x30 <= b <= 0x39
            value += b - 0x30
        elseif 0x41 <= b <= 0x46
            value += b - 0x41 + 10
        elseif 0x61 <= b <= 0x66
            value += b - 0x61 + 10
        else
            throw(ErrorException("invalid JSON unicode escape"))
        end
    end
    return value
end

function jparse_string(p::JsonParser)
    p.pos <= length(p.data) && p.data[p.pos] == 0x22 ||
        throw(ErrorException("invalid JSON string"))
    p.pos += 1
    out = UInt8[]
    while true
        p.pos > length(p.data) && throw(ErrorException("unterminated JSON string"))
        b = p.data[p.pos]
        if b == 0x22
            p.pos += 1
            return String(out)
        elseif b == 0x5c
            p.pos += 1
            p.pos > length(p.data) && throw(ErrorException("unterminated JSON escape"))
            e = p.data[p.pos]
            p.pos += 1
            if e == 0x22
                push!(out, 0x22)
            elseif e == 0x5c
                push!(out, 0x5c)
            elseif e == 0x2f
                push!(out, 0x2f)
            elseif e == 0x62
                push!(out, 0x08)
            elseif e == 0x66
                push!(out, 0x0c)
            elseif e == 0x6e
                push!(out, 0x0a)
            elseif e == 0x72
                push!(out, 0x0d)
            elseif e == 0x74
                push!(out, 0x09)
            elseif e == 0x75
                cp = jparse_hex4(p)
                if 0xD800 <= cp <= 0xDBFF
                    (p.pos + 1 <= length(p.data) &&
                     p.data[p.pos] == 0x5c && p.data[p.pos + 1] == 0x75) ||
                        throw(ErrorException("invalid JSON surrogate pair"))
                    p.pos += 2
                    lo = jparse_hex4(p)
                    0xDC00 <= lo <= 0xDFFF ||
                        throw(ErrorException("invalid JSON surrogate pair"))
                    cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00)
                elseif 0xDC00 <= cp <= 0xDFFF
                    throw(ErrorException("invalid JSON lone surrogate"))
                end
                append!(out, codeunits(string(Char(cp))))
            else
                throw(ErrorException("invalid JSON escape"))
            end
        elseif b < 0x20
            throw(ErrorException("invalid JSON control character"))
        else
            push!(out, b)
            p.pos += 1
        end
    end
end

function jparse_number(p::JsonParser)
    start = p.pos
    if p.pos <= length(p.data) && p.data[p.pos] == 0x2d
        p.pos += 1
    end
    while p.pos <= length(p.data) && p.data[p.pos] >= 0x30 && p.data[p.pos] <= 0x39
        p.pos += 1
    end
    is_float = false
    if p.pos <= length(p.data) && p.data[p.pos] == 0x2e
        is_float = true
        p.pos += 1
        while p.pos <= length(p.data) && p.data[p.pos] >= 0x30 && p.data[p.pos] <= 0x39
            p.pos += 1
        end
    end
    if p.pos <= length(p.data) && (p.data[p.pos] == 0x65 || p.data[p.pos] == 0x45)
        is_float = true
        p.pos += 1
        if p.pos <= length(p.data) && (p.data[p.pos] == 0x2b || p.data[p.pos] == 0x2d)
            p.pos += 1
        end
        while p.pos <= length(p.data) && p.data[p.pos] >= 0x30 && p.data[p.pos] <= 0x39
            p.pos += 1
        end
    end
    text = String(p.data[start:(p.pos - 1)])
    if is_float
        return parse(Float64, text)
    end
    value = tryparse(Int64, text)
    return value === nothing ? parse(Float64, text) : value
end

function jparse_array(p::JsonParser)
    p.pos += 1
    items = Any[]
    jskip(p)
    if p.pos <= length(p.data) && p.data[p.pos] == 0x5d
        p.pos += 1
        return items
    end
    while true
        push!(items, jparse_value(p))
        jskip(p)
        p.pos > length(p.data) && throw(ErrorException("invalid JSON array"))
        b = p.data[p.pos]
        p.pos += 1
        if b == 0x5d
            return items
        elseif b == 0x2c
            continue
        else
            throw(ErrorException("invalid JSON array"))
        end
    end
end

function jparse_object(p::JsonParser)
    p.pos += 1
    dict = Dict{String, Any}()
    jskip(p)
    if p.pos <= length(p.data) && p.data[p.pos] == 0x7d
        p.pos += 1
        return dict
    end
    while true
        jskip(p)
        p.pos <= length(p.data) && p.data[p.pos] == 0x22 ||
            throw(ErrorException("invalid JSON object key"))
        key = jparse_string(p)
        jexpect(p, 0x3a)
        dict[key] = jparse_value(p)
        jskip(p)
        p.pos > length(p.data) && throw(ErrorException("invalid JSON object"))
        b = p.data[p.pos]
        p.pos += 1
        if b == 0x7d
            return dict
        elseif b == 0x2c
            continue
        else
            throw(ErrorException("invalid JSON object"))
        end
    end
end

function jparse_value(p::JsonParser)
    jskip(p)
    p.pos > length(p.data) && throw(ErrorException("invalid JSON"))
    b = p.data[p.pos]
    if b == 0x7b
        return jparse_object(p)
    elseif b == 0x5b
        return jparse_array(p)
    elseif b == 0x22
        return jparse_string(p)
    elseif b == 0x74
        jliteral(p, "true")
        return true
    elseif b == 0x66
        jliteral(p, "false")
        return false
    elseif b == 0x6e
        jliteral(p, "null")
        return nothing
    else
        return jparse_number(p)
    end
end

function json_parse_line(line::AbstractString)
    return jparse_value(JsonParser(line))
end

# ---------------------------------------------------------------------------
# Protocol output and captured-stream IO
# ---------------------------------------------------------------------------

function send_msg(msg)
    msg isa Dict || (msg = Dict(msg))
    haskey(msg, "id") || (msg["id"] = CURRENT_ID[])
    print(REAL_STDOUT, json_dumps(msg), '\n')
    flush(REAL_STDOUT)
    return nothing
end

function emit_stream(stream::Symbol, data::AbstractString)
    isempty(data) && return nothing
    send_msg(Dict("type" => "stream", "stream" => String(stream), "data" => data))
    return nothing
end

function utf8_lossy(bytes::Vector{UInt8})
    io = IOBuffer()
    i = 1
    n = length(bytes)
    while i <= n
        b = bytes[i]
        if b < 0x80
            write(io, b)
            i += 1
        else
            len = if b < 0xE0
                2
            elseif b < 0xF0
                3
            elseif b < 0xF8
                4
            else
                0
            end
            valid = len > 0 && i + len - 1 <= n
            if valid
                for j in 1:(len - 1)
                    c = bytes[i + j]
                    if !(0x80 <= c < 0xC0)
                        valid = false
                        break
                    end
                end
            end
            if valid
                write(io, bytes[i:(i + len - 1)])
                i += len
            else
                write(io, UInt8[0xEF, 0xBF, 0xBD])
                i += 1
            end
        end
    end
    return String(take!(io))
end

mutable struct ProtocolIO <: IO
    stream::Symbol
    buf::Vector{UInt8}
end

ProtocolIO(name::Symbol) = ProtocolIO(name, UInt8[])

function Base.unsafe_write(io::ProtocolIO, p::Ptr{UInt8}, n::UInt)
    append!(io.buf, unsafe_wrap(Vector{UInt8}, p, n))
    emit_ready(io)
    return nothing
end

Base.unsafe_read(io::ProtocolIO, ::Ptr{UInt8}, ::UInt) = 0
Base.eof(io::ProtocolIO) = false
Base.isreadable(io::ProtocolIO) = false
Base.iswritable(io::ProtocolIO) = true

function Base.unsafe_flush(io::ProtocolIO)
    emit_ready(io; force = true)
    return nothing
end

function emit_ready(io::ProtocolIO; force::Bool = false)
    nl = findlast(==(UInt8('\n')), io.buf)
    if nl === nothing
        force || return nothing
        nl = length(io.buf)
    end
    nl == 0 && return nothing
    chunk = io.buf[1:nl]
    deleteat!(io.buf, 1:nl)
    emit_stream(io.stream, utf8_lossy(chunk))
    return nothing
end

# ---------------------------------------------------------------------------
# MIME display helpers
# ---------------------------------------------------------------------------

function mime_payload(mime, v, plain)
    r = try
        repr(mime, v)
    catch
        nothing
    end
    r === nothing && return nothing
    if r isa AbstractString
        # Julia falls back to the text/plain repr for MIME types an object does
        # not implement; skip payloads identical to the plain representation so
        # plain values are not mislabeled as html/images.
        return (isempty(r) || r == plain) ? nothing : r
    elseif r isa AbstractVector && eltype(r) == UInt8
        return isempty(r) ? nothing : base64encode(r)
    end
    return nothing
end

function plain_repr(v)
    r = try
        repr(MIME"text/plain"(), v)
    catch
        nothing
    end
    if r isa AbstractString
        return r
    end
    return try
        sprint(show, v)
    catch
        "<unrepresentable>"
    end
end

function display_parts(v)
    plain = plain_repr(v)
    parts = Vector{Vector{String}}()
    for (name, mime) in DISPLAY_MIMES
        payload = mime_payload(mime, v, plain)
        payload === nothing || push!(parts, [name, payload])
    end
    push!(parts, ["text/plain", plain])
    return parts
end

function bundle_for(v)
    plain = plain_repr(v)
    bundle = Dict{String, Any}()
    for (name, mime) in DISPLAY_MIMES
        payload = mime_payload(mime, v, plain)
        payload === nothing || (bundle[name] = payload)
    end
    bundle["text/plain"] = plain
    return bundle
end

function mars_display(args...; kwargs...)
    if !isempty(args) && args[1] isa IO
        return Base.display(args[1], args[2:end]...)
    end
    isempty(args) && return nothing
    mime_filter = nothing
    if args[1] isa MIME
        mime_filter = args[1]
        args = args[2:end]
    end
    isempty(args) && return nothing
    for obj in args
        if mime_filter !== nothing
            name = string(mime_filter)
            payload = try
                repr(mime_filter, obj)
            catch
                nothing
            end
            if payload isa AbstractVector && eltype(payload) == UInt8
                payload = base64encode(payload)
            elseif !(payload isa AbstractString) || isempty(payload)
                payload = nothing
            end
            if payload !== nothing
                send_msg(Dict(
                    "type" => "mime",
                    "data" => Dict(name => payload),
                    "metadata" => Dict{String, Any}(),
                ))
            end
        else
            send_msg(Dict(
                "type" => "mime",
                "data" => bundle_for(obj),
                "metadata" => Dict{String, Any}(),
            ))
        end
    end
    return nothing
end

function mars_readline(args...; kwargs...)
    if !isempty(kwargs)
        return Base.readline(args...; kwargs...)
    end
    prompt = isempty(args) ? "" : string(args[1])
    isempty(prompt) || emit_stream(:stdout, prompt)
    send_msg(Dict("type" => "input_request", "prompt" => prompt))
    response = take!(PENDING_INPUT)
    response === nothing && throw(EOFError("kernel input channel closed"))
    emit_stream(:stdout, response * "\n")
    return response
end

# ---------------------------------------------------------------------------
# Cell evaluation
# ---------------------------------------------------------------------------

function parse_cell(source::AbstractString)
    parsed = Meta.parseall(source)
    parsed === nothing && return Expr[]
    if parsed isa Expr && parsed.head === :toplevel
        return collect(parsed.args)
    elseif parsed isa Expr
        return Expr[parsed]
    end
    return Expr[]
end

function is_silent(ex)
    ex isa Expr || return false
    h = ex.head
    h in SILENT_HEADS && return true
    if h === :call && !isempty(ex.args) &&
            ex.args[1] in (:setindex!, :setproperty!)
        return true
    end
    if h === :. && !isempty(ex.args) && ex.args[1] isa Expr && is_silent(ex.args[1])
        return true
    end
    return false
end

function eval_cell(source::AbstractString)
    exprs = parse_cell(source)
    isempty(exprs) && return (nothing, false)
    value = nothing
    has_value = false
    for (i, ex) in enumerate(exprs)
        v = Core.eval(M, ex)
        if i == length(exprs) && !is_silent(ex)
            value = v
            has_value = true
        end
    end
    return (value, has_value)
end

function record_ans(v)
    prev = Base.isdefined(M, :ans) ? Base.getglobal(M, :ans) : nothing
    Base.setglobal!(M, :ans_prev, prev)
    Base.setglobal!(M, :ans, v)
    return nothing
end

function handle_execute(req)
    id = get(req, "id", nothing)
    source = get(req, "source", "")
    CURRENT_ID[] = id
    out_io = ProtocolIO(:stdout)
    err_io = ProtocolIO(:stderr)
    cell_error = Ref{Any}(nothing)
    cell_bt = Ref{Any}(nothing)
    cell_value = Ref{Any}(nothing)
    cell_has_value = Ref{Bool}(false)
    try
        if isdefined(Base, :redirect_stderr)
            redirect_stdout(() -> redirect_stderr(() -> try
                v, hv = eval_cell(source)
                cell_value[] = v
                cell_has_value[] = hv
            catch err
                cell_error[] = err
                cell_bt[] = catch_backtrace()
            end end), out_io)
        else
            redirect_stdout(() -> try
                v, hv = eval_cell(source)
                cell_value[] = v
                cell_has_value[] = hv
            catch err
                cell_error[] = err
                cell_bt[] = catch_backtrace()
            end, out_io)
        end
    catch err
        cell_error[] = err
        cell_bt[] = catch_backtrace()
    end
    emit_ready(out_io; force = true)
    emit_ready(err_io; force = true)
    if cell_error[] === nothing && cell_has_value[] && cell_value[] !== nothing
        record_ans(cell_value[])
        send_msg(Dict("type" => "display", "data" => display_parts(cell_value[])))
    end
    ok = cell_error[] === nothing
    payload = ok ? nothing : error_payload(cell_error[], cell_bt[])
    send_msg(Dict("type" => "result", "id" => id, "ok" => ok, "error" => payload))
    CURRENT_ID[] = nothing
    return nothing
end

function error_payload(err, bt)
    name = last(split(string(typeof(err)), '.'))
    msg = try
        sprint(showerror, err)
    catch
        "error"
    end
    trace_io = IOBuffer()
    try
        Base.StackTraces.show(IOContext(trace_io, :limit => true), bt)
    catch
        try
            Base.StackTraces.show(IOContext(trace_io, :limit => true), stacktrace(bt))
        catch
            try
                Base.show_backtrace(IOContext(trace_io, :limit => true), bt)
            catch
            end
        end
    end
    trace_text = String(take!(trace_io))
    lines = collect(split(trace_text, '\n'; keepempty = false))
    length(lines) > 20 && (lines = lines[(end - 19):end])
    return Dict("name" => name, "message" => msg, "traceback" => lines)
end

function handle_inspect(req)
    skip = Set([:ans, :ans_prev, :display, :readline])
    names = sort!(collect(names(M, all = true)); by = string)
    variables = Any[]
    for s in names
        s isa Symbol || continue
        str = String(s)
        (startswith(str, "_") || startswith(str, "mars") || s in skip) && continue
        val = try
            Base.getglobal(M, s)
        catch
            continue
        end
        tname = try
            string(typeof(val))
        catch
            "Any"
        end
        preview = try
            r = repr(MIME"text/plain"(), val)
            r = r isa AbstractString ? r : sprint(show, val)
            length(r) > 160 ? r[1:157] * "..." : r
        catch
            nothing
        end
        push!(variables, Dict("name" => str, "type" => tname, "value" => preview))
    end
    send_msg(Dict("type" => "inspection", "id" => get(req, "id", nothing), "variables" => variables))
    return nothing
end

function handle_check(req)
    source = get(req, "source", "")
    err = nothing
    try
        Meta.parseall(source)
    catch e
        err = e
    end
    payload = nothing
    if err !== nothing
        payload = Dict(
            "name" => last(split(string(typeof(err)), '.')),
            "message" => try
                sprint(showerror, err)
            catch
                ""
            end,
            "lineno" => 1,
            "offset" => 1,
        )
    end
    send_msg(Dict("type" => "check_result", "id" => get(req, "id", nothing), "error" => payload))
    return nothing
end

# ---------------------------------------------------------------------------
# Driver entry point
# ---------------------------------------------------------------------------

function stdin_reader()
    for line in eachline(Base.stdin)
        req = try
            json_parse_line(line)
        catch
            nothing
        end
        req isa Dict || continue
        t = get(req, "type", "")
        if t == "input_response"
            txt = get(req, "text", nothing)
            put!(PENDING_INPUT, txt isa String ? txt : nothing)
        elseif t == "shutdown"
            put!(REQUESTS, nothing)
            return
        else
            put!(REQUESTS, req)
        end
    end
    put!(REQUESTS, nothing)
    return nothing
end

# Install Mars-aware display/readline inside the cell module. The bindings
# shadow the Base exports imported via the implicit `using Base`.
for (name, fn) in ((:display, mars_display), (:readline, mars_readline))
    try
        Base.setglobal!(M, name, fn)
    catch
        Core.eval(M, Expr(:(=), name, QuoteNode(fn)))
    end
end

Base.Threads.@spawn(stdin_reader())

while true
    req = take!(REQUESTS)
    req === nothing && break
    t = get(req, "type", "")
    if t == "execute"
        handle_execute(req)
    elseif t == "inspect"
        handle_inspect(req)
    elseif t == "check"
        handle_check(req)
    end
end

exit(0)
