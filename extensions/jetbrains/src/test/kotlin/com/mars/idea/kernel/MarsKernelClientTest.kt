package com.mars.idea.kernel

import com.google.gson.JsonParser
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.PosixFilePermissions

class MarsKernelClientTest {

    /** A fake `mars kernel serve` that answers newline-delimited JSON requests. */
    private fun fakeKernel(script: String): Path {
        val path = Files.createTempDirectory("mars-fake-kernel")
            .resolve("mars")
        Files.writeString(path, script)
        Files.setPosixFilePermissions(path, PosixFilePermissions.fromString("rwxr-xr-x"))
        return path
    }

    private val responder = """#!/usr/bin/env bash
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  case "${'$'}line" in
    *'"type":"execute"'*) resp='{"type":"execute","outputs":[{"type":"text","text":"hello"},{"type":"error","name":"ValueError","message":"bad","traceback":["line1","line2"]}],"execution_count":7,"duration_millis":42}' ;;
    *'"type":"variables"'*) resp='{"type":"variables","variables":[{"name":"x","typeName":"i32","value":"1"},{"name":"y","typeName":null,"value":null}]}' ;;
    *'"type":"status"'*) resp='{"type":"status","status":"ready","execution_count":3}' ;;
    *'"type":"diagnose"'*) resp='{"type":"diagnose","diagnostics":[{"cellIndex":0,"line":1,"column":2,"endLine":1,"endColumn":5,"severity":"warning","code":null,"message":"unused"}]}' ;;
    *'"type":"input"'*) resp='{"type":"input","ok":true}' ;;
    *'"type":"reset"'*) resp='{"type":"reset","ok":true}' ;;
    *'"type":"cancel"'*) resp='{"type":"cancel","ok":true}' ;;
    *'"type":"boom"'*) resp='{"type":"error","message":"kernel exploded"}' ;;
    *) resp='{"type":"ok"}' ;;
  esac
  printf '{"id":"%s","response":%s}\n' "${'$'}id" "${'$'}resp"
done
"""

    @Test
    fun `execute parses outputs and counts`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        val result = client.execute("1 + 1", "rust")
        assertEquals(7, result.executionCount)
        assertEquals(42, result.durationMillis)
        assertEquals(2, result.outputs.size)
        assertEquals(MarsOutputItem("text/plain", "hello"), result.outputs[0])
        assertTrue(result.outputs[1].text.startsWith("ValueError: bad"))
        assertTrue(result.outputs[1].text.contains("line1"))
        client.dispose()
    }

    @Test
    fun `variables parses entries with null fields`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        val vars = client.variables()
        assertEquals(2, vars.size)
        assertEquals("x", vars[0].name)
        assertEquals("i32", vars[0].typeName)
        assertEquals("1", vars[0].value)
        assertEquals("y", vars[1].name)
        client.dispose()
    }

    @Test
    fun `status returns status and count`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        assertEquals("ready" to 3L, client.status())
        client.dispose()
    }

    @Test
    fun `diagnose parses diagnostics`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        val diags = client.diagnose("fn main() {}")
        assertEquals(1, diags.size)
        assertEquals(0, diags[0].cellIndex)
        assertEquals("warning", diags[0].severity)
        assertEquals("unused", diags[0].message)
        client.dispose()
    }

    @Test
    fun `input reset cancel and dispose complete`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        client.input("answer")
        client.reset()
        client.cancel()
        client.dispose()
    }

    @Test
    fun `kernel error response throws MarsKernelException`() {
        val client = MarsKernelClient(fakeKernel(responder).toString(), null)
        try {
            client.request(JsonParser.parseString("""{"type":"boom"}""").asJsonObject) { it }
            fail("expected MarsKernelException")
        } catch (e: MarsKernelException) {
            assertEquals("kernel exploded", e.message)
        }
        client.dispose()
    }

    @Test
    fun `stderr listener receives kernel stderr`() {
        val script = """#!/usr/bin/env bash
echo "kernel log line" >&2
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf '{"id":"%s","response":{"type":"ok"}}\n' "${'$'}id"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), "/tmp/fake.mars")
        val received = java.util.concurrent.CopyOnWriteArrayList<String>()
        client.setStderrListener { received.add(it) }
        client.request(JsonParser.parseString("""{"type":"ping"}""").asJsonObject) { it }
        // stderr reader is async; wait briefly
        val deadline = System.currentTimeMillis() + 5000
        while (received.isEmpty() && System.currentTimeMillis() < deadline) Thread.sleep(50)
        assertTrue(received.any { it.contains("kernel log line") })
        client.dispose()
    }

    @Test
    fun `sparse responses fall back to defaults`() {
        // Every response is a bare {"type":"ok"} — all optional fields missing.
        val script = """#!/usr/bin/env bash
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf '{"id":"%s","response":{"type":"ok"}}\n' "${'$'}id"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        val exec = client.execute("x")
        assertTrue(exec.outputs.isEmpty())
        assertEquals(0, exec.executionCount)
        assertEquals(0, exec.durationMillis)
        assertTrue(client.variables().isEmpty())
        assertEquals("unknown" to 0L, client.status())
        assertTrue(client.diagnose("x").isEmpty())
        client.dispose()
    }

    @Test
    fun `partial responses use field defaults`() {
        val script = """#!/usr/bin/env bash
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  case "${'$'}line" in
    *'"type":"execute"'*) resp='{"type":"execute","outputs":[{"type":"mystery"},{"type":"error"}],"execution_count":2}' ;;
    *'"type":"variables"'*) resp='{"type":"variables","variables":[{}]}' ;;
    *'"type":"diagnose"'*) resp='{"type":"diagnose","diagnostics":[{"severity":"info"}]}' ;;
    *) resp='{"type":"ok"}' ;;
  esac
  printf '{"id":"%s","response":%s}\n' "${'$'}id" "${'$'}resp"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        val exec = client.execute("x")
        assertEquals(2, exec.outputs.size)
        assertEquals("text/plain", exec.outputs[0].mime) // unknown output type
        assertEquals("text/error", exec.outputs[1].mime) // error with no fields
        assertTrue(exec.outputs[1].text.startsWith("Error: "))
        val vars = client.variables()
        assertEquals("?", vars[0].name)
        val diags = client.diagnose("x")
        assertEquals("info", diags[0].severity)
        client.dispose()
    }

    @Test
    fun `response without response envelope throws`() {
        val script = """#!/usr/bin/env bash
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf '{"id":"%s"}\n' "${'$'}id"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        try {
            client.request(JsonParser.parseString("""{"type":"ping"}""").asJsonObject) { it }
            fail("expected MarsKernelException")
        } catch (e: MarsKernelException) {
            assertEquals("missing response", e.message)
        } finally {
            client.dispose()
        }
    }

    @Test
    fun `request fails when kernel process exits immediately`() {
        val script = """#!/usr/bin/env bash
exit 0
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        try {
            client.request(
                JsonParser.parseString("""{"type":"ping"}""").asJsonObject,
                timeoutSeconds = 2,
            ) { it }
            fail("expected MarsKernelException")
        } catch (e: MarsKernelException) {
            // either stdin write failure or timeout — both are kernel errors
        } finally {
            // cancel() swallows kernel errors via runCatching
            client.cancel()
            client.dispose()
            client.dispose()
        }
    }

    @Test
    fun `protocol noise lines are ignored`() {
        val script = """#!/usr/bin/env bash
echo ""
echo "   "
echo "garbage line"
echo '[1,2,3]'
echo '{"response":{"type":"ok"}}'
echo '{"id":"unknown-id","response":{"type":"ok"}}'
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf '{"id":"%s","response":{"type":"ok"}}\n' "${'$'}id"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        client.request(JsonParser.parseString("""{"type":"ping"}""").asJsonObject) { it }
        client.dispose()
    }

    @Test
    fun `request times out when kernel never responds`() {
        val script = """#!/usr/bin/env bash
while IFS= read -r line; do sleep 30; done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        try {
            client.request(
                JsonParser.parseString("""{"type":"ping"}""").asJsonObject,
                timeoutSeconds = 1,
            ) { it }
            fail("expected timeout")
        } catch (e: MarsKernelException) {
            assertEquals("kernel request timed out", e.message)
        } finally {
            client.dispose()
        }
    }

    @Test
    fun `unparseable and id-less lines are ignored`() {
        val script = """#!/usr/bin/env bash
echo "not json at all"
echo '{"no_id":true}'
while IFS= read -r line; do
  id=$(printf '%s' "${'$'}line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf '{"id":"%s","response":{"type":"ok"}}\n' "${'$'}id"
done
"""
        val client = MarsKernelClient(fakeKernel(script).toString(), null)
        client.request(JsonParser.parseString("""{"type":"ping"}""").asJsonObject) { it }
        client.dispose()
    }
}
