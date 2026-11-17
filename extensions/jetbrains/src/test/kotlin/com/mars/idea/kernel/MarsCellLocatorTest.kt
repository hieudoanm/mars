package com.mars.idea.kernel

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Files

class MarsCellLocatorTest {

    private val notebook = """
        {
          "cells": [
            {"cell_type": "markdown", "source": "# Title"},
            {"cell_type": "code", "language": "rust", "source": "fn main() {}"},
            {"cell_type": "code", "source": ["let x = 1;", "x + 1"]}
          ],
          "metadata": {"language": "rust"}
        }
    """.trimIndent()

    @Test
    fun `kindOf maps known cell types`() {
        assertEquals(MarsCellKind.CODE, kindOf("code"))
        assertEquals(MarsCellKind.MARKDOWN, kindOf("markdown"))
        assertEquals(MarsCellKind.RAW, kindOf("raw"))
        assertEquals(MarsCellKind.OTHER, kindOf("something-else"))
    }

    @Test
    fun `notebookLanguage extracts metadata language`() {
        assertEquals("rust", notebookLanguage(notebook))
        assertNull(notebookLanguage("not json"))
        assertNull(notebookLanguage("{}"))
        assertNull(notebookLanguage("""{"metadata": {"language": 42}}"""))
    }

    @Test
    fun `cellsFromText returns only code cells with parsed source`() {
        val cells = cellsFromText(notebook)
        assertEquals(2, cells.size)
        assertEquals(1, cells[0].index)
        assertEquals("rust", cells[0].language)
        assertEquals("fn main() {}", cells[0].source)
        assertEquals(2, cells[1].index)
        assertEquals("let x = 1;\nx + 1", cells[1].source)
        assertTrue(cellsFromText("not json").isEmpty())
        assertTrue(cellsFromText("{}").isEmpty())
    }

    @Test
    fun `cellEntries pairs cells with source ranges`() {
        val entries = cellEntries(notebook)
        assertEquals(3, entries.size)
        assertEquals(MarsCellKind.MARKDOWN, kindOf(entries[0].cell.cellType))
        for (entry in entries) {
            val slice = notebook.substring(entry.range.first, entry.range.last + 1)
            assertTrue(slice.startsWith("\"") || slice.startsWith("["))
        }
        assertTrue(cellEntries("not json").isEmpty())
    }

    @Test
    fun `cellAtOffset finds cell containing offset`() {
        val entries = cellEntries(notebook)
        val codeOffset = entries[1].range.first + 2
        val found = cellAtOffset(notebook, codeOffset)
        assertEquals(1, found?.index)
        assertNull(cellAtOffset(notebook, 0))
        assertNull(cellAtOffset(notebook, notebook.length + 10))
        // allCells = true also returns non-code cells
        val mdOffset = entries[0].range.first + 2
        assertEquals(0, cellAtOffset(notebook, mdOffset, allCells = true)?.index)
        assertNull(cellAtOffset(notebook, mdOffset))
    }

    @Test
    fun `navigateCellOffset moves between cells and wraps`() {
        val entries = cellEntries(notebook)
        val first = entries.first().range.first
        val offsetInFirst = first + 1
        assertEquals(entries[1].range.first, navigateCellOffset(notebook, offsetInFirst, +1))
        assertEquals(entries[2].range.first, navigateCellOffset(notebook, entries[1].range.first + 1, +1))
        // wraps from last to first
        assertEquals(first, navigateCellOffset(notebook, entries[2].range.first + 1, +1))
        // wraps backwards from first to last
        assertEquals(entries[2].range.first, navigateCellOffset(notebook, offsetInFirst, -1))
        // caret outside all cells -> first cell start
        assertEquals(first, navigateCellOffset(notebook, notebook.length + 5, +1))
        assertNull(navigateCellOffset("{}", 0, +1))
    }

    @Test
    fun `cellsFromText handles unusual source shapes`() {
        // source as number/object -> empty string; missing cell_type -> "code"
        val doc = """{"cells":[{"source":42},{"cell_type":"code","source":{"a":1}},{"source":[1,"two"]}]}"""
        val cells = cellsFromText(doc)
        assertEquals(3, cells.size)
        assertEquals("code", cells[0].cellType)
        assertEquals("", cells[0].source)
        assertEquals("", cells[1].source)
        assertEquals("1\ntwo", cells[2].source)
        // "cells" not an array / missing entirely
        assertTrue(cellsFromText("""{"cells":{}}""").isEmpty())
        // cell entries that aren't objects are skipped
        assertTrue(cellsFromText("""{"cells":["x",null,5]}""").isEmpty())
    }

    @Test
    fun `cellEntries returns empty when source keys are missing`() {
        // Parsed cells exist but no "source" keys in the text.
        assertTrue(cellEntries("""{"cells":[{"cell_type":"code"}]}""").isEmpty())
        assertTrue(cellEntries("""{"cells":[]}""").isEmpty())
        assertTrue(cellEntries("""{"cells":"nope"}""").isEmpty())
        // non-object cell elements are tolerated when enough "source" keys exist
        // (the extra key comes from metadata, keeping the count check satisfied)
        val mixed = cellEntries("""{"cells":["x",{"cell_type":"code","source":"a"}],"metadata":{"source":"m"}}""")
        assertEquals(2, mixed.size)
        assertEquals("", mixed[0].cell.source)
    }

    @Test
    fun `cellEntries handles escaped quotes and non-string sources`() {
        val doc = """{"cells":[{"cell_type":"code","source":"a\"b"},{"cell_type":"code","source":42,"note":"x"},{"cell_type":"code","source":["p","q"]}]}"""
        val entries = cellEntries(doc)
        assertEquals(3, entries.size)
        // escaped quote inside string source
        val first = doc.substring(entries[0].range.first, entries[0].range.last + 1)
        assertTrue(first.contains("\\\""))
        // numeric source: valueEnd scans to the next quote
        assertTrue(entries[1].range.last > entries[1].range.first)
        // array source
        assertTrue(doc.substring(entries[2].range.first, entries[2].range.last + 1).startsWith("["))
    }

    @Test
    fun `navigateCellOffset on empty and single-cell docs`() {
        assertNull(navigateCellOffset("""{"cells":[]}""", 0, +1))
        val single = """{"cells":[{"cell_type":"code","source":"x"}]}"""
        val start = cellEntries(single).first().range.first
        assertEquals(start, navigateCellOffset(single, start + 1, +1))
        assertEquals(start, navigateCellOffset(single, start + 1, -1))
    }

    @Test
    fun `cellAtOffset edge offsets`() {
        val doc = """{"cells":[{"cell_type":"code","source":"abc"}]}"""
        val range = cellEntries(doc).first().range
        assertEquals(0, cellAtOffset(doc, range.first)?.index)
        assertEquals(0, cellAtOffset(doc, range.last)?.index)
        assertNull(cellAtOffset(doc, range.last + 1))
    }

    @Test
    fun `readFileText reads existing file and returns null for missing`() {
        val path = Files.createTempFile("mars-cell", ".mars")
        Files.writeString(path, notebook)
        assertEquals(notebook, readFileText(path))
        assertNull(readFileText(path.resolveSibling("does-not-exist.mars")))
    }
}
