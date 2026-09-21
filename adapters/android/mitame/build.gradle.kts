plugins {
    id("com.android.library")
    id("maven-publish")
}

android {
    namespace = "io.github.mataku.mitame"
    compileSdk = 36

    defaultConfig {
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

group = "io.github.mataku"
version = "0.1.0"

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = project.group.toString()
            artifactId = "mitame-android"
            version = project.version.toString()
            afterEvaluate {
                from(components["release"])
            }
            pom {
                name.set("mitame-android")
                description.set("Capture adapter for mitame visual regression testing")
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
