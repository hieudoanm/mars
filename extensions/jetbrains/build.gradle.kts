import org.jetbrains.intellij.platform.gradle.IntelliJPlatformType

plugins {
    kotlin("jvm") version "2.0.21"
    id("org.jetbrains.intellij.platform") version "2.1.0"
}

group = "com.mars.idea"
version = "0.0.1"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    intellijPlatform {
        create(IntelliJPlatformType.IntellijIdeaCommunity, "2024.3")
        bundledModule("com.intellij.modules.json")
        instrumentationTools()
    }
    implementation("com.google.code.gson:gson:2.11.0")
}

intellijPlatform {
    pluginConfiguration {
        id = "com.mars.idea"
        name = "Mars Notebooks"
        version = "0.0.1"
        description = """
            Mars notebook (.mars) support: file type, syntax highlighting,
            cell execution via `mars kernel serve`, and a kernel tool window.
        """.trimIndent()
        vendor {
            name = "mars-project"
        }
    }
}
