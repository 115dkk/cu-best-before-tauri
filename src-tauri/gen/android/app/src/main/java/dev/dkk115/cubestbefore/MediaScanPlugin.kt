package dev.dkk115.cubestbefore

import android.app.Activity
import android.media.MediaScannerConnection
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class ScanFileArgs {
  lateinit var path: String
}

@InvokeArg
class ForgetFileArgs {
  lateinit var path: String
}

/**
 * Registers a file the Rust side wrote into shared storage with MediaStore so
 * gallery apps show it (ADR-0003). Direct-path writes are not reliably
 * auto-indexed on every OEM firmware (observed on Samsung / Android 16).
 */
@TauriPlugin
class MediaScanPlugin(private val activity: Activity) : Plugin(activity) {
  @Command
  fun scanFile(invoke: Invoke) {
    val args = invoke.parseArgs(ScanFileArgs::class.java)
    val file = File(args.path)
    if (!file.isFile) {
      invoke.reject("file not found: ${args.path}")
      return
    }
    // Resolves once the media scanner has indexed the file (uri is null if it refused it).
    MediaScannerConnection.scanFile(
      activity.applicationContext,
      arrayOf(file.absolutePath),
      arrayOf("image/png")
    ) { _, uri ->
      val result = JSObject()
      result.put("uri", uri?.toString())
      invoke.resolve(result)
    }
  }

  /**
   * Rescans a path that no longer exists so MediaStore drops its stale row
   * (the media scanner reconciles a missing file by deleting its entry). Resolves
   * when the scanner is done; never rejects for a missing file.
   */
  @Command
  fun forgetFile(invoke: Invoke) {
    val args = invoke.parseArgs(ForgetFileArgs::class.java)
    MediaScannerConnection.scanFile(
      activity.applicationContext,
      arrayOf(File(args.path).absolutePath),
      arrayOf("image/png")
    ) { _, _ -> invoke.resolve(JSObject()) }
  }
}
