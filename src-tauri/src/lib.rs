//! Tauri shell: wires the pure `cu_best_before_core` logic to the WebView.
//! Commands live in `commands.rs`; this file only assembles the app.

#![forbid(unsafe_code)]

mod commands;

use chrono::Local;
use commands::AppState;
use cu_best_before_core::store::SheetStore;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let store = SheetStore::open_in(app.path().app_data_dir()?)?;
            // 앱 시작마다 보존 기간이 지난 조사표를 정리한다(ADR-0004).
            store.purge_expired(Local::now().naive_local())?;
            app.manage(AppState { store });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::catalog,
            commands::list_sheets,
            commands::create_sheet,
            commands::get_sheet,
            commands::save_sheet,
            commands::delete_sheet,
            commands::slot_options,
            commands::export_sheet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
