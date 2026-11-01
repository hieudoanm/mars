package com.mars.idea.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.mars.idea.kernel.MarsKernelManager
import com.mars.idea.ui.MarsToolWindow

class InterruptExecutionAction : AnAction("Interrupt Execution", "Cancel the in-flight kernel execution", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = MarsToolWindow.hasActiveExecution()
    }

    override fun actionPerformed(e: AnActionEvent) {
        val file = currentMarsPath(e)
        MarsKernelManager.instance.clientFor(file?.absolutePath).cancel()
    }
}

class RestartKernelAction : AnAction("Restart Kernel", "Reset the Mars kernel session", null) {
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = true
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e)
        val client = MarsKernelManager.instance.clientFor(file?.absolutePath)
        com.intellij.openapi.application.ApplicationManager.getApplication().executeOnPooledThread {
            runCatching { client.reset() }
            ApplicationManager.getApplication().invokeLater {
                notifyInfo(project, "Mars kernel reset")
                MarsToolWindow.refresh(project)
            }
        }
    }
}
