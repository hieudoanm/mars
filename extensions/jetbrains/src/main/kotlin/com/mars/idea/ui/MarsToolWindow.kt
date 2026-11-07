package com.mars.idea.ui

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.mars.idea.kernel.MarsKernelManager
import java.awt.BorderLayout
import java.awt.Dimension
import javax.swing.JButton
import javax.swing.JPanel
import javax.swing.JScrollPane
import javax.swing.JTextArea
import javax.swing.SwingUtilities
import javax.swing.SwingWorker

/** Shared kernel state + UI hooks for the Mars Kernel tool window. */
object MarsToolWindow {
    @Volatile var executing = false
        private set
    @Volatile var statusText = "idle"
        private set
    @Volatile var execCount = 0L
        private set

    private var panel: MarsKernelPanel? = null

    fun hasActiveExecution() = executing

    fun markExecuting() {
        executing = true
        SwingUtilities.invokeLater { panel?.refreshStatus() }
    }

    fun markIdle() {
        executing = false
        SwingUtilities.invokeLater { panel?.refreshStatus() }
    }

    fun appendOutput(project: Project, text: String) {
        SwingUtilities.invokeLater { panel?.appendOutput(text) }
    }

    fun refresh(project: Project) {
        val file = com.intellij.openapi.fileEditor.FileEditorManager.getInstance(project)
            .selectedFiles.firstOrNull()?.path
        val client = MarsKernelManager.instance.clientFor(file)
        object : SwingWorker<Pair<String, Long>, Void>() {
            override fun doInBackground(): Pair<String, Long> {
                return runCatching { client.status() }.getOrElse {
                    "disconnected" to 0L
                }
            }

            override fun done() {
                runCatching {
                    val (status, count) = get()
                    statusText = status
                    execCount = count
                    panel?.refreshStatus()
                    panel?.refreshVariables(client)
                }
            }
        }.execute()
    }

    internal fun attach(p: MarsKernelPanel) {
        panel = p
    }
}

class MarsKernelPanel(private val project: Project) : JPanel(BorderLayout()) {
    private val statusArea = JTextArea(3, 50).apply {
        isEditable = false
        font = java.awt.Font("Monospaced", java.awt.Font.PLAIN, 12)
    }
    private val variablesArea = JTextArea(8, 50).apply {
        isEditable = false
        font = java.awt.Font("Monospaced", java.awt.Font.PLAIN, 12)
    }
    private val outputArea = JTextArea(10, 50).apply {
        isEditable = false
        font = java.awt.Font("Monospaced", java.awt.Font.PLAIN, 12)
    }
    private val refreshButton = JButton("Refresh").apply {
        addActionListener { MarsToolWindow.refresh(project) }
    }
    private val interruptButton = JButton("Interrupt").apply {
        addActionListener {
            val file = com.intellij.openapi.fileEditor.FileEditorManager.getInstance(project)
                .selectedFiles.firstOrNull()?.path
            MarsKernelManager.instance.clientFor(file).cancel()
        }
    }

    init {
        MarsToolWindow.attach(this)
        val buttons = JPanel().apply {
            add(refreshButton)
            add(interruptButton)
        }
        add(buttons, BorderLayout.NORTH)
        val varsScroll = JScrollPane(variablesArea).apply { preferredSize = Dimension(400, 140) }
        val outScroll = JScrollPane(outputArea)
        val split = JPanel(BorderLayout()).apply {
            add(varsScroll, BorderLayout.NORTH)
            add(outScroll, BorderLayout.CENTER)
        }
        add(split, BorderLayout.CENTER)
        refreshStatus()
    }

    fun refreshStatus() {
        statusArea.text = buildString {
            append("Status: ${MarsToolWindow.statusText}")
            append(if (MarsToolWindow.executing) " (executing)" else "")
            append("\nExecutions: ${MarsToolWindow.execCount}")
        }
    }

    fun refreshVariables(client: com.mars.idea.kernel.MarsKernelClient) {
        object : SwingWorker<String, Void>() {
            override fun doInBackground(): String = runCatching {
                client.variables().joinToString("\n") {
                    "${it.name}: ${it.typeName ?: "?"} = ${it.value ?: "?"}"
                }
            }.getOrElse { "(unavailable: ${it.message})" }
            override fun done() {
                variablesArea.text = get()
            }
        }.execute()
    }

    fun appendOutput(text: String) {
        outputArea.append("\n$text\n")
        outputArea.caretPosition = outputArea.document.length
    }

    fun appendStderr(msg: String) {
        outputArea.append("[kernel] $msg\n")
    }
}

class MarsToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = MarsKernelPanel(project)
        val content = ContentFactory.getInstance().createContent(panel, "", false)
        toolWindow.contentManager.addContent(content)
        MarsKernelManager.instance.setStderrListener { msg ->
            ApplicationManager.getApplication().invokeLater { panel.appendStderr(msg) }
        }
    }

    override fun shouldBeAvailable(project: Project) = true
}
