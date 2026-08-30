# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile
# Tauri loads Android plugin classes by name (reflection) and dispatches
# @Command methods reflectively; keep the app's own plugin intact under R8.
-keep @app.tauri.annotation.TauriPlugin class dev.dkk115.cubestbefore.** { *; }
-keep @app.tauri.annotation.InvokeArg class dev.dkk115.cubestbefore.** { *; }
-keepclassmembers class dev.dkk115.cubestbefore.** {
  @app.tauri.annotation.Command <methods>;
}
