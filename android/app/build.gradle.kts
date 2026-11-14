// Module de l'application (ADR-0009) : versions dans gradle/libs.versions.toml.
// [impl->req~compte-systeme-dedie~1]

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "fr.lemesre.unicontacts"
    compileSdk = 37

    defaultConfig {
        // Identifiant provisoire tant que OQ-007 (nom) n'est pas tranchée.
        applicationId = "fr.lemesre.unicontacts"
        minSdk = 33
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildFeatures {
        compose = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            // Produits par « sh tools/android.sh natif » (ADR-0009), jamais
            // versionnés.
            jniLibs.directories.add("build/rust/jniLibs")
            kotlin.directories.add("build/generated/uniffi")
        }
    }

    lint {
        warningsAsErrors = true
        abortOnError = true
    }
}

// Bibliothèque Rust et liaisons Kotlin, avant toute construction : la
// logique vit dans tools/android.sh, Gradle ne fait que l'appeler.
val natifRust = tasks.register<Exec>("natifRust") {
    workingDir = rootDir.parentFile
    commandLine("sh", "tools/android.sh", "natif")
}
tasks.named("preBuild") { dependsOn(natifRust) }

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.viewmodel.compose)
    implementation(libs.core.ktx)
    implementation(libs.coroutines.android)
    implementation("${libs.jna.get()}@aar")
    testImplementation(libs.junit)
}
