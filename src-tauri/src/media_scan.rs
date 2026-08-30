//! Android: register an exported file with MediaStore so gallery apps show it.
//!
//! Direct-path writes into `Pictures/` are attributed to the app by the FUSE layer, but
//! not every firmware indexes them automatically (Samsung / Android 16 did not). The
//! Kotlin side (`MediaScanPlugin.kt` in `gen/android`) calls `MediaStore.scanFile`.
//! On desktop this module is a no-op so the shell compiles everywhere.

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(target_os = "android")]
mod android {
    use serde::{Deserialize, Serialize};
    use tauri::Runtime;
    use tauri::plugin::PluginHandle;

    #[derive(Serialize)]
    struct ScanFileRequest<'a> {
        path: &'a str,
    }

    #[derive(Deserialize)]
    struct ScanFileResponse {
        uri: Option<String>,
    }

    /// Handle to the Kotlin `MediaScanPlugin`.
    pub struct MediaScan<R: Runtime>(pub(super) PluginHandle<R>);

    impl<R: Runtime> MediaScan<R> {
        /// Scans `path` into MediaStore; returns the content uri when the scanner reports one.
        pub fn scan_file(&self, path: &str) -> Result<Option<String>, String> {
            let response: ScanFileResponse = self
                .0
                .run_mobile_plugin("scanFile", ScanFileRequest { path })
                .map_err(|error| error.to_string())?;
            Ok(response.uri)
        }
    }
}

#[cfg(target_os = "android")]
pub use android::MediaScan;

/// Plugin that registers the Kotlin `MediaScanPlugin` on Android and does nothing elsewhere.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("media-scan")
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                let handle =
                    _api.register_android_plugin("dev.dkk115.cubestbefore", "MediaScanPlugin")?;
                _app.manage(MediaScan(handle));
            }
            Ok(())
        })
        .build()
}

/// Registers an exported file with the device's media index. Returns the MediaStore uri
/// when available; `Ok(None)` on desktop or when the scanner gave no uri.
pub fn register_export<R: Runtime>(
    app: &tauri::AppHandle<R>,
    path: &str,
) -> Result<Option<String>, String> {
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        app.state::<MediaScan<R>>()
            .scan_file(path)
            .map_err(|error| format!("갤러리 등록 실패: {error}"))
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, path);
        Ok(None)
    }
}
