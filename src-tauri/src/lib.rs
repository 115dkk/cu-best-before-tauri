//! Tauri shell: wires the pure `cu_best_before_core` logic to the WebView.
//! Commands live in `commands.rs`; this file only assembles the app.

#![forbid(unsafe_code)]

mod commands;

use chrono::{Duration, Local};
use commands::AppState;
use cu_best_before_core::store::{RETENTION_DAYS, SheetStore};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?.join("sheets");
            let store = SheetStore::open(dir)?;
            // 앱 시작마다 30일이 지난 조사표를 정리한다(ADR-0004).
            store.purge_older_than(Local::now().naive_local(), Duration::days(RETENTION_DAYS))?;
            app.manage(AppState { store });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
