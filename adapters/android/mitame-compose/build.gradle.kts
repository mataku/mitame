plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.0"
    id("maven-publish")
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

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

group = "io.github.mataku"

dependencies {
    api(project(":mitame"))
    implementation(platform("androidx.compose:compose-bom:2026.09.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-test-junit4")
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = project.group.toString()
            artifactId = "mitame-android-compose"
            version = project.version.toString()
            afterEvaluate {
                from(components["release"])
            }
            pom {
                name.set("mitame-android-compose")
                description.set("Compose capture helper for mitame visual regression testing")
                url.set("https://github.com/mataku/mitame")
                licenses {
                    license {
                        name.set("MIT")
                        url.set("https://github.com/mataku/mitame/blob/main/LICENSE")
                    }
                }
            }
        }
    }
}
