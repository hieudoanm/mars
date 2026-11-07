package com.mars.idea.actions

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.mars.idea.kernel.MarsKernelManager
import com.mars.idea.kernel.cellAtOffset
import com.mars.idea.kernel.notebookLanguage
import com.mars.idea.kernel.readFileText
import com.mars.idea.ui.MarsToolWindow
import java.io.File

/**
 * Executes the code cell under the caret via `mars kernel serve`.
 * Outputs are shown in a notification and appended to the Mars Kernel
 * tool window; cell outputs in the file are not rewritten (the kernel
 * protocol is execution-only).
 */
class ExecuteCellAction : AnAction("Execute Cell at Caret", "Execute the .mars cell under the caret", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        val editor = e.getData(com.intellij.openapi.actionSystem.CommonDataKeys.EDITOR)
        val offset = editor?.caretModel?.offset ?: 0
        val text = readFileText(file.toPath())
        if (text == null) {
            notifyError(project, "Could not read ${file.name}")
            return
        }
        val cell = cellAtOffset(text, offset)
        if (cell == null) {
            notifyInfo(project, "Caret is not inside a code cell")
            return
        }
        val language = cell.language ?: notebookLanguage(text)
        val client = MarsKernelManager.instance.clientFor(file.absolutePath)
        MarsToolWindow.markExecuting()
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                val result = client.execute(cell.source, language)
                val summary = buildString {
                    append("Cell ${cell.index + 1} executed")
                    if (result.executionCount > 0) append(" (count ${result.executionCount}, ${result.durationMillis}ms)")
                    result.outputs.forEach { o ->
                        append("\n--- ${o.mime} ---\n")
                        append(o.text.take(4000))
                    }
                    if (result.outputs.isEmpty()) append("\n(no output)")
                }
                ApplicationManager.getApplication().invokeLater {
                    MarsToolWindow.appendOutput(project, summary)
                    MarsToolWindow.markIdle()
                    MarsToolWindow.refresh(project)
                }
            } catch (ex: Exception) {
                ApplicationManager.getApplication().invokeLater {
                    MarsToolWindow.appendOutput(project, "Execution failed: ${ex.message}")
                    MarsToolWindow.markIdle()
                    MarsToolWindow.refresh(project)
                }
            }
        }
    }
}
