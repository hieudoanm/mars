import org.jetbrains.intellij.platform.gradle.IntelliJPlatformType

plugins {
    kotlin("jvm") version "2.4.21"
    id("org.jetbrains.intellij.platform") version "2.20.1"
    id("org.jetbrains.kotlinx.kover") version "0.9.11"
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
    }
    implementation("com.google.code.gson:gson:2.15.0")
    testImplementation("junit:junit:4.13.2")
}

tasks.test {
    useJUnit()
}

// Coverage is gated on the pure-Kotlin cell locator and kernel client;
// IntelliJ UI/actions classes and the ApplicationManager-bound kernel
// manager require the IDE test framework and are excluded from the gate.
kover {
    reports {
        filters {
            excludes {
                classes(
                    "com.mars.idea.ui.*",
                    "com.mars.idea.actions.*",
                    "com.mars.idea.Mars*",
                    "com.mars.idea.kernel.MarsKernelManager",
                )
            }
        }
        verify {
            rule {
                minBound(80, coverageUnits = kotlinx.kover.gradle.plugin.dsl.CoverageUnit.LINE)
                minBound(80, coverageUnits = kotlinx.kover.gradle.plugin.dsl.CoverageUnit.BRANCH)
            }
        }
    }
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
            name = "project-mars"
        }
    }
}
