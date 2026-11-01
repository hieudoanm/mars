package com.mars.idea

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage

@State(name = "MarsSettings", storages = [Storage("mars.xml")])
class MarsSettings : PersistentStateComponent<MarsSettings> {
    var cliPath: String = "mars"

    override fun getState(): MarsSettings = this
    override fun loadState(state: MarsSettings) {
        cliPath = state.cliPath
    }

    companion object {
        val instance: MarsSettings
            get() = ApplicationManager.getApplication().getService(MarsSettings::class.java)

        var cliPath: String
            get() = instance.cliPath
            set(value) {
                instance.cliPath = value
            }
    }
}
