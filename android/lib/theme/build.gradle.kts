plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.theme"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig { minSdk = Versions.Android.minSdkVersion }

    buildFeatures { compose = true }

    composeOptions { kotlinCompilerExtensionVersion = Versions.kotlinCompilerExtensionVersion }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions { jvmTarget = Versions.jvmTarget }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

dependencies {
    implementation(Dependencies.Compose.material3)
    implementation(Dependencies.Compose.ui)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.AndroidX.dataStorePreferences)
    implementation(Dependencies.Koin.core)
}
