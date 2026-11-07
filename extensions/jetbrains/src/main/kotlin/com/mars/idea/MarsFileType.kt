package com.mars.idea

import com.intellij.json.JsonFileType
import com.intellij.json.JsonLanguage
import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

class MarsFileType private constructor() : LanguageFileType(JsonLanguage.INSTANCE) {

    override fun getName() = "Mars Notebook"
    override fun getDescription() = "Mars notebook"
    override fun getDefaultExtension() = "mars"
    override fun getIcon(): Icon = JsonFileType.INSTANCE.icon

    companion object {
        @JvmField
        val MARS_FILE_TYPE = MarsFileType()
    }
}
