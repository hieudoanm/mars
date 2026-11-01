package com.mars.idea.kernel

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.mars.idea.MarsSettings
import java.util.concurrent.ConcurrentHashMap

@Service(Service.Level.APP)
class MarsKernelManager {
    private val clients = ConcurrentHashMap<String, MarsKernelClient>()
    private var listener: ((String) -> Unit)? = null

    fun setStderrListener(l: (String) -> Unit) {
        listener = l
    }

    fun clientFor(notebookPath: String?): MarsKernelClient {
        val key = notebookPath ?: ""
        return clients.computeIfAbsent(key) {
            MarsKernelClient(MarsSettings.cliPath, notebookPath).apply {
                setStderrListener { msg -> listener?.invoke(msg) }
            }
        }
    }

    fun disposeAll() {
        clients.values.forEach { it.dispose() }
        clients.clear()
    }

    companion object {
        val instance: MarsKernelManager
            get() = ApplicationManager.getApplication().getService(MarsKernelManager::class.java)
    }
}
