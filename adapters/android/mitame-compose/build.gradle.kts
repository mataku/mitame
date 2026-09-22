plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.0"
    id("com.vanniktech.maven.publish")
}

android {
    namespace = "io.github.mataku.mitame.compose"
    compileSdk = 37

    defaultConfig {
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
    }
}

dependencies {
    api(project(":mitame"))
    implementation(platform("androidx.compose:compose-bom:2026.09.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-test-junit4")
}

mavenPublishing {
    configure(
        com.vanniktech.maven.publish.AndroidSingleVariantLibrary(
            javadocJar = com.vanniktech.maven.publish.JavadocJar.Empty(),
            sourcesJar = com.vanniktech.maven.publish.SourcesJar.Sources(),
            variant = "release",
        )
    )
    coordinates(artifactId = "mitame-android-compose")
    pom {
        name.set("mitame-android-compose")
        description.set("Compose capture helper for mitame visual regression testing")
    }
}
