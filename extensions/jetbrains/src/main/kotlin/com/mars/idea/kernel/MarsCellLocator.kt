package com.mars.idea.kernel

import com.google.gson.JsonArray
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import java.nio.file.Files
import java.nio.file.Path

/** A code cell located in a .mars notebook document. */
data class MarsCell(
    val index: Int,
    val cellType: String,
    val language: String?,
    val source: String,
)

enum class MarsCellKind { CODE, MARKDOWN, RAW, OTHER }

fun kindOf(cellType: String): MarsCellKind = when (cellType) {
    "code" -> MarsCellKind.CODE
    "markdown" -> MarsCellKind.MARKDOWN
    "raw" -> MarsCellKind.RAW
    else -> MarsCellKind.OTHER
}

private fun sourceOf(cell: JsonObject): String {
    val source = cell.get("source") ?: return ""
    return when {
        source.isJsonPrimitive && source.asJsonPrimitive.isString -> source.asString
        source is JsonArray -> source.joinToString("\n") { it.asString }
        else -> ""
    }
}

/** Notebook-level `metadata.language` (e.g. "rust", "python", "r"). */
fun notebookLanguage(text: String): String? {
    return try {
        val root = JsonParser.parseString(text).asJsonObject
        root.getAsJsonObject("metadata")?.get("language")
            ?.takeIf { it.isJsonPrimitive && it.asJsonPrimitive.isString }?.asString
    } catch (_: Exception) {
        null
    }
}

/** Parse code cells from raw .mars JSON text. */
fun cellsFromText(text: String): List<MarsCell> {
    return try {
        val root = JsonParser.parseString(text).asJsonObject
        val cells = root.getAsJsonArray("cells") ?: return emptyList()
        cells.mapIndexedNotNull { i, el ->
            val obj = el.asJsonObject ?: return@mapIndexedNotNull null
            val type = obj.get("cell_type")?.takeIf { it.isJsonPrimitive }?.asString ?: "code"
            if (kindOf(type) != MarsCellKind.CODE) return@mapIndexedNotNull null
            MarsCell(
                index = i,
                cellType = type,
                language = obj.get("language")?.takeIf { it.isJsonPrimitive }?.asString,
                source = sourceOf(obj),
            )
        }
    } catch (_: Exception) {
        emptyList()
    }
}

/** A cell paired with the document range of its `"source"` value. */
data class CellEntry(val cell: MarsCell, val range: IntRange)

/**
 * Pair every cell (in cells[] order) with its `"source"` value's document
 * range, in document order. Returns an empty list on malformed JSON or when
 * the source-key count does not match the cell count.
 */
fun cellEntries(text: String): List<CellEntry> {
    val root = try {
        JsonParser.parseString(text).asJsonObject
    } catch (_: Exception) {
        return emptyList()
    }
    val arr = root.get("cells")?.takeIf { it.isJsonArray }?.asJsonArray ?: return emptyList()
    val all = arr.mapIndexed { i, el ->
        val obj = el.takeIf { it.isJsonObject }?.asJsonObject
        val type = obj?.get("cell_type")?.takeIf { it.isJsonPrimitive }?.asString ?: "code"
        MarsCell(i, type, obj?.get("language")?.takeIf { it.isJsonPrimitive }?.asString, obj?.let { sourceOf(it) } ?: "")
    }

    val sourceKey = Regex("\"source\"\\s*:")
    val keyMatches = sourceKey.findAll(text).toList()
    if (keyMatches.size < all.size) return emptyList()
    return all.mapIndexed { i, cell ->
        val m = keyMatches[i]
        val valueStart = startOfValue(text, m.range.last + 1)
        CellEntry(cell, valueStart..valueEnd(text, valueStart))
    }
}

/**
 * Find the cell under [offset] by pairing each cell (in cells[] order) with
 * its `"source"` key occurrence (in document order). Returns the cell whose
 * source value contains [offset], or null.
 */
fun cellAtOffset(text: String, offset: Int, allCells: Boolean = false): MarsCell? {
    return cellEntries(text).firstOrNull { offset in it.range }?.let { entry ->
        if (!allCells && kindOf(entry.cell.cellType) != MarsCellKind.CODE) null else entry.cell
    }
}

/**
 * Returns the source-range start offset of the cell [delta] positions away
 * from the cell containing [offset] (delta = +1 next, -1 previous). Wraps
 * around the ends. Returns null when the document has no cells or the caret
 * is outside every cell.
 */
fun navigateCellOffset(text: String, offset: Int, delta: Int): Int? {
    val entries = cellEntries(text)
    if (entries.isEmpty()) return null
    val current = entries.indexOfFirst { offset in it.range }
    if (current < 0) return entries.first().range.first
    val next = (current + delta).mod(entries.size)
    return entries[next].range.first
}

private fun startOfValue(text: String, afterColon: Int): Int {
    var i = afterColon
    while (i < text.length && text[i].isWhitespace()) i++
    return i
}

private fun valueEnd(text: String, start: Int): Int {
    if (start >= text.length) return start
    return if (text[start] == '[') {
        val close = text.indexOf(']', start)
        if (close < 0) text.length else close + 1
    } else {
        var i = start + 1
        while (i < text.length) {
            if (text[i] == '\\') i += 2
            else if (text[i] == '"') return i + 1
            else i++
        }
        text.length
    }
}

fun readFileText(path: Path): String? = runCatching { Files.readString(path) }.getOrNull()
