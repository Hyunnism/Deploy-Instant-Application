# Add project specific ProGuard rules here.
# Deploy Instant ProGuard / R8 Rules for Release Build

# Keep Tauri core classes and plugins
-keep class app.tauri.** { *; }
-keep interface app.tauri.** { *; }

# Keep Deploy Instant application classes, activities, services, receivers
-keep class com.hyunnism.deployinstant.** { *; }
-keep interface com.hyunnism.deployinstant.** { *; }

# Keep native methods across all classes for JNI linkage
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep AndroidX WebKit & JavaScript interfaces
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
-keep class androidx.webkit.** { *; }

# Keep Kotlin intrinsics and metadata
-dontwarn kotlin.**
-dontwarn javax.annotation.**
