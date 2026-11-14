// Projet Android d'unicontacts (ADR-0006, ADR-0009).
// [impl->req~compte-systeme-dedie~1]
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "unicontacts"
include(":app")
