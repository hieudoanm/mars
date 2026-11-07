package com.mars.idea.actions

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.mars.idea.MarsFileType
import com.mars.idea.MarsSettings
import java.io.File
import java.nio.file.Files

private const val NEW_NOTEBOOK_TEMPLATE = """{
  "format_version": 1,
  "metadata": {
    "title": "Untitled",
    "language": "rust"
  },
  "cells": [
    {
      "id": "cell-1",
      "cell_type": "markdown",
      "source": "# Untitled Mars notebook",
      "metadata": {}
    },
    {
      "id": "cell-2",
      "cell_type": "code",
      "source": "fn main() {\n    println!(\"Hello from Mars!\");\n}",
      "metadata": {},
      "execution_count": null,
      "outputs": []
    }
  ]
}
"""

private fun notify(project: Project, message: String, type: NotificationType = NotificationType.INFORMATION) {
    ApplicationManager.getApplication().invokeLater {
        NotificationGroupManager.getInstance().getNotificationGroup("Mars")
            .createNotification(message, type).notify(project)
    }
}

internal fun notifyError(project: Project, message: String) = notify(project, message, NotificationType.ERROR)
internal fun notifyInfo(project: Project, message: String) = notify(project, message, NotificationType.INFORMATION)

/** Runs a `mars` CLI command off the EDT and reports the outcome. */
internal fun runCli(project: Project, args: List<String>, onSuccess: ((String) -> Unit)? = null) {
    val cmd = listOf(MarsSettings.cliPath) + args
    ApplicationManager.getApplication().executeOnPooledThread {
        try {
            val pb = ProcessBuilder(cmd).redirectErrorStream(true)
            val proc = pb.start()
            val output = proc.inputStream.bufferedReader().readText()
            val code = proc.waitFor()
            ApplicationManager.getApplication().invokeLater {
                if (code == 0) {
                    onSuccess?.invoke(output)
                    if (onSuccess == null) notifyInfo(project, "mars ${args.firstOrNull()} succeeded")
                } else {
                    notifyError(project, "mars ${args.firstOrNull()} failed (exit $code):\n${output.take(500)}")
                }
            }
        } catch (e: Exception) {
            ApplicationManager.getApplication().invokeLater {
                notifyError(project, "Failed to run mars: ${e.message}")
            }
        }
    }
}

/** Returns the filesystem path of the .mars file backing the current editor, or null. */
internal fun currentMarsPath(e: AnActionEvent): File? {
    val file = e.getData(com.intellij.openapi.actionSystem.CommonDataKeys.PSI_FILE)?.virtualFile
        ?: e.getData(com.intellij.openapi.actionSystem.CommonDataKeys.VIRTUAL_FILE)
    if (file == null || file.fileType != MarsFileType.MARS_FILE_TYPE) return null
    return File(file.path)
}

class NewNotebookAction : AnAction("New Mars Notebook", "Create a new .mars notebook", null) {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val dir = com.intellij.openapi.fileChooser.FileChooserFactory.getInstance()
            .createFileChooser(
                com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
                    .createSingleFolderDescriptor(),
                project, null,
            ).choose(project).firstOrNull() ?: return
        val dirPath = dir.path
        var name = "untitled.mars"
        var counter = 1
        while (File(dirPath, name).exists()) {
            counter++
            name = "untitled-$counter.mars"
        }
        val file = File(dirPath, name)
        Files.writeString(file.toPath(), NEW_NOTEBOOK_TEMPLATE)
        openFile(project, file)
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

private fun openFile(project: Project, file: File) {
    val vFile = LocalFileSystem.getInstance().refreshAndFindFileByIoFile(file) ?: return
    FileEditorManager.getInstance(project).openFile(vFile, true)
}

abstract class MarsFileAction(text: String, desc: String) : AnAction(text, desc, null) {
    abstract fun argsFor(file: File): List<String>

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = currentMarsPath(e) != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        runCli(project, argsFor(file))
    }
}

class RunNotebookAction : MarsFileAction("Run Notebook", "Execute all cells with `mars run`") {
    override fun argsFor(file: File) = listOf("run", file.absolutePath)
}

class ExportHtmlAction : MarsFileAction("Export to HTML", "Export the notebook to standalone HTML") {
    override fun argsFor(file: File) = listOf("export", file.absolutePath, "html", htmlPath(file))
}

class ExportMarkdownAction : MarsFileAction("Export to Markdown", "Export the notebook to Markdown") {
    override fun argsFor(file: File) = listOf("export", file.absolutePath, "markdown", file.absolutePath.removeSuffix(".mars") + ".md")
}

class ExportIpynbAction : MarsFileAction("Export to Jupyter (.ipynb)", "Export the notebook to .ipynb") {
    override fun argsFor(file: File) = listOf("export", file.absolutePath, "ipynb", file.absolutePath.removeSuffix(".mars") + ".ipynb")
}

class InspectNotebookAction : MarsFileAction("Inspect Notebook", "Show notebook metadata via `mars inspect`") {
    override fun argsFor(file: File) = listOf("inspect", file.absolutePath)

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val file = currentMarsPath(e) ?: return
        runCli(project, listOf("inspect", file.absolutePath)) { output ->
            notifyInfo(project, "mars inspect:\n${output.take(1000)}")
        }
    }
}

private fun htmlPath(file: File) = file.absolutePath.removeSuffix(".mars") + ".html"
