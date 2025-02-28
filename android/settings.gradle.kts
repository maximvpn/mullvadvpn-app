pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

rootProject.name = "MullvadVPN"

include(
    ":app",
    ":service",
    ":tile",
    ":widget"
)
include(
    ":lib:billing",
    ":lib:common",
    ":lib:common-test",
    ":lib:daemon-grpc",
    ":lib:endpoint",
    ":lib:map",
    ":lib:model",
    ":lib:payment",
    ":lib:resource",
    ":lib:shared",
    ":lib:talpid",
    ":lib:theme"
)
include(
    ":test",
    ":test:arch",
    ":test:common",
    ":test:e2e",
    ":test:mockapi"
)
