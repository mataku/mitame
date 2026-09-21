plugins {
    id("com.android.library")
}

android {
    namespace = "io.github.mataku.mitame.example"
    compileSdk = 36

    defaultConfig {
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    testOptions {
        unitTests.isIncludeAndroidResources = true
    }
}

dependencies {
    implementation(project(":mitame"))
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.robolectric:robolectric:4.15.1")
    testImplementation("androidx.test:core:1.7.0")
}

tasks.withType<Test>().configureEach {
    environment("MITAME_OUTPUT_DIR", System.getenv("MITAME_OUTPUT_DIR") ?: rootProject.file(".mitame/current").absolutePath)
}
