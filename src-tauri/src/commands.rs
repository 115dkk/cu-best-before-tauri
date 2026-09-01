//! Tauri 커맨드 — `now`와 경로를 구해 core에 넘기고 결과를 JSON으로 돌려주는 어댑터.
//!
//! 도메인 규칙은 하나도 두지 않는다(ADR-0001). 화면에 보일 문자열은 core의 뷰(ADR-0008)가
//! 만들고, 오류는 core `Error`의 `Display`를 그대로 `String`으로 넘겨 토스트에 쓴다.

use chrono::{Local, NaiveDateTime};
use cu_best_before_core::export::{self, ExportResult};
use cu_best_before_core::slots::{self, SlotOptions};
use cu_best_before_core::store::{SheetStore, SheetSummary};
use cu_best_before_core::view::{Catalog, SheetView};
use cu_best_before_core::{Product, Sheet};
use tauri::{AppHandle, Manager, State};

use crate::media_scan;

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

/// 품목·구역 목록과 라벨(표시 순서). 화면이 시작할 때 한 번 받는다.
#[tauri::command]
pub fn catalog() -> Catalog {
    Catalog::current()
}

/// 저장된 조사표 요약 목록(최신순).
#[tauri::command]
pub fn list_sheets(state: State<'_, AppState>) -> Result<Vec<SheetSummary>, String> {
    state.store.list().map_err(message)
}

/// 빈 조사표를 만들어 즉시 저장하고 돌려준다.
#[tauri::command]
pub fn create_sheet(state: State<'_, AppState>) -> Result<SheetView, String> {
    let sheet = state.store.create(now()).map_err(message)?;
    Ok(SheetView::new(&sheet, now()))
}

/// id로 조사표를 읽는다.
#[tauri::command]
pub fn get_sheet(state: State<'_, AppState>, id: String) -> Result<SheetView, String> {
    let sheet = state.store.load(&id).map_err(message)?;
    Ok(SheetView::new(&sheet, now()))
}

/// 조사표를 정규화해 저장하고, 정규화된 조사표를 돌려준다.
/// 인자로는 뷰 JSON을 그대로 보내도 된다(라벨 필드는 무시된다).
#[tauri::command]
pub fn save_sheet(state: State<'_, AppState>, sheet: Sheet) -> Result<SheetView, String> {
    let sheet = sheet.normalized(now()).map_err(message)?;
    state.store.save(&sheet).map_err(message)?;
    Ok(SheetView::new(&sheet, now()))
}

/// 조사표를 지운다(멱등).
#[tauri::command]
pub fn delete_sheet(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.store.delete(&id).map_err(message)
}

/// 품목의 후보 슬롯 목록. `include`는 편집 중인 항목의 슬롯(지났더라도 휠에 남긴다).
#[tauri::command]
pub fn slot_options(product: Product, include: Option<NaiveDateTime>) -> SlotOptions {
    slots::default_slot_options(product, now(), include)
}

/// 조사표를 기기 공용 사진 폴더에 PNG로 저장하고, 갤러리가 보도록 MediaStore에 등록한다.
/// 이전 내보내기 파일은 지우고 MediaStore에서도 내린다.
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
    let mut result = export::export_png(&sheet, &pictures).map_err(message)?;
    // 이전 내보내기는 파일이 이미 지워졌다. 인덱스 정리 실패는 사용자에게 알릴 일이 아니라 무시한다.
    for removed in &result.removed {
        let _ = media_scan::forget_export(&app, removed);
    }
    // 파일은 이미 저장됐다. 등록 실패는 오류로 알리되, 파일이 사라지는 것은 아니다.
    result.media_uri = media_scan::register_export(&app, &result.path)?;
    Ok(result)
}
