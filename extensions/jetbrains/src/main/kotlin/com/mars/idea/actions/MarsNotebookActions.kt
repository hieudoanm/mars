package com.mars.idea.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.ui.Messages
import com.mars.idea.kernel.MarsKernelManager
import com.mars.idea.kernel.cellsFromText
import com.mars.idea.kernel.navigateCellOffset
import com.mars.idea.kernel.notebookLanguage
import com.mars.idea.kernel.readFileText
import com.mars.idea.ui.MarsToolWindow
import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption

private fun editorAndFile(e: AnActionEvent): Pair<com.intellij.openapi.editor.Editor, File>? {
    val editor = e.getData(com.intellij.openapi.actionSystem.CommonDataKeys.EDITOR) ?: return null
    val file = currentMarsPath(e) ?: return null
    return editor to file
}

private fun moveCaret(editor: com.intellij.openapi.editor.Editor, offset: Int) {
    editor.caretModel.moveToOffset(offset.coerceIn(0, editor.document.textLength))
    editor.scrollingModel.scrollToCaret(com.intellij.openapi.editor.ScrollType.CENTER)
}

class NextCellAction : AnAction("Next Cell", "Move caret to the next .mars cell", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = editorAndFile(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val (editor, file) = editorAndFile(e) ?: return
        val text = readFileText(file.toPath()) ?: return
        navigateCellOffset(text, editor.caretModel.offset, +1)?.let { moveCaret(editor, it) }
    }
}

class PreviousCellAction : AnAction("Previous Cell", "Move caret to the previous .mars cell", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = editorAndFile(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val (editor, file) = editorAndFile(e) ?: return
        val text = readFileText(file.toPath()) ?: return
        navigateCellOffset(text, editor.caretModel.offset, -1)?.let { moveCaret(editor, it) }
    }
}

/** Executes every code cell in the notebook sequentially through the kernel. */
class ExecuteAllCellsAction : AnAction("Execute All Cells", "Run all code cells in order", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        val text = readFileText(file.toPath())
        if (text == null) {
            notifyError(project, "Could not read ${file.name}")
            return
        }
        val cells = cellsFromText(text)
        if (cells.isEmpty()) {
            notifyInfo(project, "No code cells found")
            return
        }
        val language = notebookLanguage(text)
        val client = MarsKernelManager.instance.clientFor(file.absolutePath)
        MarsToolWindow.markExecuting()
        ApplicationManager.getApplication().executeOnPooledThread {
            val failures = mutableListOf<Int>()
            cells.forEachIndexed { i, cell ->
                try {
                    val result = client.execute(cell.source, language)
                    val failed = result.outputs.any { it.mime == "text/error" }
                    if (failed) failures += cell.index + 1
                    MarsToolWindow.appendOutput(
                        project,
                        "Cell ${cell.index + 1}: ${if (failed) "error" else "ok"} " +
                            "(${result.durationMillis}ms)",
                    )
                } catch (ex: Exception) {
                    failures += cell.index + 1
                    MarsToolWindow.appendOutput(project, "Cell ${cell.index + 1}: ${ex.message}")
                }
            }
            ApplicationManager.getApplication().invokeLater {
                MarsToolWindow.markIdle()
                MarsToolWindow.refresh(project)
                if (failures.isEmpty()) notifyInfo(project, "Executed ${cells.size} cells")
                else notifyError(project, "Executed ${cells.size} cells; failures in cells $failures")
            }
        }
    }
}

/** Compile-checks every code cell via the kernel `diagnose` request. */
class DiagnoseNotebookAction : AnAction("Check Notebook", "Compile-check all code cells", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        val text = readFileText(file.toPath()) ?: return
        val cells = cellsFromText(text)
        if (cells.isEmpty()) {
            notifyInfo(project, "No code cells found")
            return
        }
        val client = MarsKernelManager.instance.clientFor(file.absolutePath)
        ApplicationManager.getApplication().executeOnPooledThread {
            val report = StringBuilder()
            var count = 0
            cells.forEach { cell ->
                try {
                    val diags = client.diagnose(cell.source)
                    diags.forEach { d ->
                        count++
                        report.append("cell ${cell.index + 1}:${d.line}:${d.column} ")
                            .append("[${d.severity}] ${d.message}\n")
                    }
                } catch (ex: Exception) {
                    report.append("cell ${cell.index + 1}: diagnose failed: ${ex.message}\n")
                }
            }
            ApplicationManager.getApplication().invokeLater {
                MarsToolWindow.appendOutput(
                    project,
                    "Diagnostics ($count):\n" + (report.ifEmpty { "no issues" }),
                )
                if (count == 0) notifyInfo(project, "No diagnostics")
                else notifyError(project, "$count diagnostic(s) — see Mars Kernel tool window")
            }
        }
    }
}

/** Sends a line of text to a kernel blocked on `input()`/`mars_prompt`. */
class SendKernelInputAction : AnAction("Send Input to Kernel", "Write a line to the running kernel's stdin", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = MarsToolWindow.hasActiveExecution()
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e)
        val input = Messages.showInputDialog(
            project,
            "Text to send to the kernel (appended with a newline):",
            "Mars Kernel Input",
            Messages.getQuestionIcon(),
        ) ?: return
        val client = MarsKernelManager.instance.clientFor(file?.absolutePath)
        ApplicationManager.getApplication().executeOnPooledThread {
            runCatching { client.input(input) }
                .onSuccess {
                    ApplicationManager.getApplication().invokeLater {
                        notifyInfo(project, "Sent input to kernel")
                    }
                }
                .onFailure { ex ->
                    ApplicationManager.getApplication().invokeLater {
                        notifyError(project, "Kernel input failed: ${ex.message}")
                    }
                }
        }
    }
}

// ---------------------------------------------------------------------------
// Checkpoints (<notebook>.checkpoints/checkpoint-<epochMillis>.mars)
// ---------------------------------------------------------------------------

private fun checkpointDir(file: File) = File("${file.absolutePath}.checkpoints")

class CreateCheckpointAction : AnAction("Create Checkpoint", "Snapshot the notebook file", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        val dir = checkpointDir(file)
        dir.mkdirs()
        val dest = File(dir, "checkpoint-${System.currentTimeMillis()}.mars")
        runCatching {
            Files.copy(file.toPath(), dest.toPath(), StandardCopyOption.REPLACE_EXISTING)
        }.onSuccess {
            notifyInfo(project, "Checkpoint saved: ${dest.name}")
        }.onFailure { ex ->
            notifyError(project, "Checkpoint failed: ${ex.message}")
        }
    }
}

class RestoreCheckpointAction : AnAction("Restore Checkpoint", "Restore a checkpoint snapshot", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        val dir = checkpointDir(file)
        val checkpoints = dir.listFiles { f -> f.extension == "mars" }
            ?.sortedByDescending { it.lastModified() } ?: emptyList()
        if (checkpoints.isEmpty()) {
            notifyInfo(project, "No checkpoints found for ${file.name}")
            return
        }
        val labels = checkpoints.map { "${it.name} (${it.length()} bytes)" }.toTypedArray()
        val choice = Messages.showDialog(
            project,
            "Restore which checkpoint?",
            "Mars Checkpoints",
            labels,
            0,
            Messages.getQuestionIcon(),
        )
        if (choice < 0 || choice >= checkpoints.size) return
        val selected = checkpoints[choice]
        val confirmed = Messages.showYesNoDialog(
            project,
            "Restore ${selected.name}? The current file will be overwritten.",
            "Mars Checkpoint",
            Messages.getWarningIcon(),
        ) == Messages.YES
        if (!confirmed) return
        runCatching {
            Files.copy(selected.toPath(), file.toPath(), StandardCopyOption.REPLACE_EXISTING)
        }.onSuccess {
            com.intellij.openapi.vfs.LocalFileSystem.getInstance()
                .refreshAndFindFileByIoFile(file)
            notifyInfo(project, "Restored checkpoint ${selected.name}")
        }.onFailure { ex ->
            notifyError(project, "Restore failed: ${ex.message}")
        }
    }
}
