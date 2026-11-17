package com.mars.idea.kernel

import com.google.gson.JsonParser
import com.google.gson.JsonObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread

data class KernelVariable(val name: String, val typeName: String?, val value: String?)
data class KernelDiagnostic(
    val cellIndex: Int?, val line: Int, val column: Int,
    val endLine: Int, val endColumn: Int, val severity: String,
    val code: String?, val message: String,
)
data class ExecutionResult(
    val outputs: List<MarsOutputItem>, val executionCount: Long, val durationMillis: Long,
)
data class MarsOutputItem(val mime: String, val text: String)

class MarsKernelException(message: String) : Exception(message)

/**
 * Speaks the newline-delimited JSON kernel protocol with `mars kernel serve`.
 * One process per connection; requests carry a correlation id.
 */
class MarsKernelClient(private val cliPath: String, private val notebookPath: String?) {
    private var process: Process? = null
    private var stdin: OutputStreamWriter? = null
    private var stdout: BufferedReader? = null
    private var stderrReader: Thread? = null
    private val pending = ConcurrentHashMap<String, (JsonObject) -> Unit>()
    private val listener = Executors.newSingleThreadExecutor { r ->
        Thread(r, "mars-kernel-reader").apply { isDaemon = true }
    }
    private var requestCounter = 0L
    private var onStderr: ((String) -> Unit)? = null

    fun setStderrListener(listener: (String) -> Unit) {
        onStderr = listener
    }

    @Synchronized
    fun ensureStarted() {
        if (process?.isAlive == true) return
        val args = mutableListOf(cliPath, "kernel", "serve")
        if (notebookPath != null) args += listOf("--notebook", notebookPath)
        val pb = ProcessBuilder(args).redirectErrorStream(false)
        val proc = pb.start()
        process = proc
        stdin = OutputStreamWriter(proc.outputStream, Charsets.UTF_8)
        stdout = BufferedReader(InputStreamReader(proc.inputStream, Charsets.UTF_8))
        val err = proc.errorStream
        thread(isDaemon = true) {
            BufferedReader(InputStreamReader(err, Charsets.UTF_8)).useLines { lines ->
                lines.forEach { onStderr?.invoke(it) }
            }
        }
        listener.execute {
            try {
                stdout?.useLines { lines ->
                    lines.forEach { line -> handleLine(line) }
                }
            } catch (_: Exception) {
                // process died; pending requests fail on next write
            }
        }
    }

    private fun handleLine(line: String) {
        val trimmed = line.trim()
        if (trimmed.isEmpty()) return
        val obj = try {
            JsonParser.parseString(trimmed).asJsonObject
        } catch (_: Exception) {
            return
        }
        val id = obj.get("id")?.asString ?: return
        val callback = pending.remove(id) ?: return
        callback(obj)
    }

    fun <T> request(requestObj: JsonObject, timeoutSeconds: Long = 120, parse: (JsonObject) -> T): T {
        ensureStarted()
        val id = "req-${++requestCounter}"
        val envelope = JsonObject().apply {
            addProperty("id", id)
            add("request", requestObj)
        }
        val latch = java.util.concurrent.CountDownLatch(1)
        val box = arrayOfNulls<Any>(1)
        pending[id] = { response ->
            try {
                val resp = response.getAsJsonObject("response") ?: throw MarsKernelException("missing response")
                when (resp.get("type")?.asString) {
                    "error" -> throw MarsKernelException(resp.get("message")?.asString ?: "kernel error")
                    else -> box[0] = parse(response)
                }
            } catch (e: MarsKernelException) {
                box[0] = e
            } finally {
                latch.countDown()
            }
        }
        try {
            stdin?.write(envelope.toString() + "\n")
            stdin?.flush()
        } catch (e: Exception) {
            pending.remove(id)
            throw MarsKernelException("kernel stdin write failed: ${e.message}")
        }
        if (!latch.await(timeoutSeconds, TimeUnit.SECONDS)) {
            pending.remove(id)
            throw MarsKernelException("kernel request timed out")
        }
        val result = box[0]
        if (result is MarsKernelException) throw result
        @Suppress("UNCHECKED_CAST")
        return result as T
    }

    fun execute(source: String, language: String? = null): ExecutionResult = request(
        JsonObject().apply {
            addProperty("type", "execute")
            addProperty("source", source)
            if (language != null) addProperty("language", language)
        },
    ) { envelope ->
        val resp = envelope.getAsJsonObject("response")!!
        val outputs = resp.getAsJsonArray("outputs")?.map { el ->
            val o = el.asJsonObject
            val type = o.get("type")?.asString ?: "text"
            when (type) {
                "text" -> MarsOutputItem("text/plain", o.get("text")?.asString ?: "")
                "error" -> MarsOutputItem(
                    "text/error",
                    "${o.get("name")?.asString ?: "Error"}: ${o.get("message")?.asString ?: ""}\n" +
                        (o.getAsJsonArray("traceback")?.joinToString("\n") { it.asString } ?: ""),
                )
                else -> MarsOutputItem("text/plain", o.toString())
            }
        } ?: emptyList()
        ExecutionResult(
            outputs,
            resp.get("execution_count")?.asLong ?: 0,
            resp.get("duration_millis")?.asLong ?: 0,
        )
    }

    fun variables(): List<KernelVariable> = request(
        JsonObject().apply { addProperty("type", "variables") },
    ) { envelope ->
        envelope.getAsJsonObject("response")!!.getAsJsonArray("variables")?.map { el ->
            val v = el.asJsonObject
            KernelVariable(
                v.get("name")?.asString ?: "?",
                v.get("typeName")?.takeIf { !it.isJsonNull }?.asString,
                v.get("value")?.takeIf { !it.isJsonNull }?.asString,
            )
        } ?: emptyList()
    }

    fun status(): Pair<String, Long> = request(
        JsonObject().apply { addProperty("type", "status") },
    ) { envelope ->
        val resp = envelope.getAsJsonObject("response")!!
        (resp.get("status")?.asString ?: "unknown") to
            (resp.get("execution_count")?.asLong ?: 0)
    }

    fun diagnose(source: String): List<KernelDiagnostic> = request(
        JsonObject().apply {
            addProperty("type", "diagnose")
            addProperty("source", source)
        },
    ) { envelope ->
        envelope.getAsJsonObject("response")!!.getAsJsonArray("diagnostics")?.map { el ->
            val d = el.asJsonObject
            KernelDiagnostic(
                d.get("cellIndex")?.takeIf { !it.isJsonNull }?.asInt,
                d.get("line")?.asInt ?: 0,
                d.get("column")?.asInt ?: 0,
                d.get("endLine")?.asInt ?: 0,
                d.get("endColumn")?.asInt ?: 0,
                d.get("severity")?.asString ?: "error",
                d.get("code")?.takeIf { !it.isJsonNull }?.asString,
                d.get("message")?.asString ?: "",
            )
        } ?: emptyList()
    }

    fun input(text: String) {
        request(
            JsonObject().apply {
                addProperty("type", "input")
                addProperty("text", text)
            },
            timeoutSeconds = 10,
        ) { it }
    }

    fun cancel() {
        runCatching {
            request(JsonObject().apply { addProperty("type", "cancel") }, timeoutSeconds = 10) { it }
        }
    }

    fun reset() {
        request(JsonObject().apply { addProperty("type", "reset") }, timeoutSeconds = 30) { it }
    }

    fun dispose() {
        runCatching { process?.destroy() }
        process = null
    }
}
