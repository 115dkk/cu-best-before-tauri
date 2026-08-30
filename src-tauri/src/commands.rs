//! Tauri 커맨드 — `now`와 경로를 구해 core에 넘기고 결과를 JSON으로 돌려주는 어댑터.
//!
//! 도메인 규칙은 하나도 두지 않는다(ADR-0001). 오류는 core `Error`의 `Display`를
//! 그대로 `String`으로 넘겨 프론트엔드가 토스트에 쓴다.

use chrono::{Local, NaiveDateTime};
use cu_best_before_core::export::{self, ExportResult};
use cu_best_before_core::slots::{self, DEFAULT_HORIZON_DAYS, SlotOptions};
use cu_best_before_core::store::{SheetStore, SheetSummary};
use cu_best_before_core::{Product, Sheet};
use tauri::{AppHandle, Manager, State};

/// 앱 전체가 공유하는 조사표 저장소.
pub struct AppState {
    pub store: SheetStore,
}

/// 기기 로컬 현재 시각. 시간대 변환은 하지 않는다(ADR-0006).
fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

/// core 오류를 프론트엔드가 그대로 보여줄 수 있는 문자열로.
fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// 저장된 조사표 요약 목록(최신순).
#[tauri::command]
pub fn list_sheets(state: State<'_, AppState>) -> Result<Vec<SheetSummary>, String> {
    state.store.list().map_err(message)
}

/// 빈 조사표를 만들어 즉시 저장하고 돌려준다.
#[tauri::command]
pub fn create_sheet(state: State<'_, AppState>) -> Result<Sheet, String> {
    let sheet = Sheet::new(now());
    state.store.save(&sheet).map_err(message)?;
    Ok(sheet)
}

/// id로 조사표를 읽는다.
#[tauri::command]
pub fn get_sheet(state: State<'_, AppState>, id: String) -> Result<Sheet, String> {
    state.store.load(&id).map_err(message)
}

/// 조사표를 정규화해 저장하고, 정규화된 조사표를 돌려준다.
#[tauri::command]
pub fn save_sheet(state: State<'_, AppState>, sheet: Sheet) -> Result<Sheet, String> {
    let sheet = sheet.normalized(now()).map_err(message)?;
    state.store.save(&sheet).map_err(message)?;
    Ok(sheet)
}

/// 조사표를 지운다(멱등).
#[tauri::command]
pub fn delete_sheet(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.store.delete(&id).map_err(message)
}

/// 품목의 후보 슬롯 목록.
#[tauri::command]
pub fn slot_options(product: Product) -> SlotOptions {
    slots::slot_options(product, now(), DEFAULT_HORIZON_DAYS)
}

/// 조사표를 기기 공용 사진 폴더에 PNG로 저장한다.
#[tauri::command]
pub fn export_sheet(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ExportResult, String> {
    let sheet = state.store.load(&id).map_err(message)?;
    // Android에서 picture_dir()는 앱 전용 폴더를 준다. 공용 폴더로 반드시 바꾼다(ADR-0003).
    let app_pictures = app.path().picture_dir().map_err(message)?;
    let pictures = export::public_pictures_dir(&app_pictures);
    export::export_png(&sheet, &pictures).map_err(message)
}
