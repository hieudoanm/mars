package com.mars.idea

import com.intellij.openapi.options.Configurable
import com.intellij.openapi.options.ConfigurationException
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.FormBuilder
import java.awt.BorderLayout
import javax.swing.JComponent
import javax.swing.JPanel

class MarsSettingsConfigurable : Configurable {
    private val cliPathField = TextFieldWithBrowseButton().apply {
        text = MarsSettings.cliPath
    }
    private var panel: JPanel? = null

    override fun getDisplayName() = "Mars"

    override fun createComponent(): JComponent {
        val form = FormBuilder.createFormBuilder()
            .addLabeledComponent(JBLabel("Mars CLI path:"), cliPathField)
            .addComponentFillVertically(JPanel(), 0)
            .panel
        panel = JPanel(BorderLayout()).apply { add(form, BorderLayout.NORTH) }
        return panel!!
    }

    override fun isModified(): Boolean = cliPathField.text != MarsSettings.cliPath

    @Throws(ConfigurationException::class)
    override fun apply() {
        MarsSettings.cliPath = cliPathField.text.trim().ifEmpty { "mars" }
    }

    override fun reset() {
        cliPathField.text = MarsSettings.cliPath
    }

    override fun disposeUIResources() {
        panel = null
    }
}
