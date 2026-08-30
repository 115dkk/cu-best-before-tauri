//! CU 소비기한 조사표의 도메인 로직 전부.
//!
//! 이 크레이트는 Tauri에 의존하지 않는다(ADR-0001). 현재 시각과 파일 경로는 언제나
//! 호출자가 인자로 넘기므로, Android 없이 `cargo test`만으로 동작을 검증할 수 있다.
//!
//! - [`domain`] — 품목·구역·항목·조사표와 정규화
//! - [`slots`] — 후보 슬롯 계산과 라벨
//! - [`store`] — 조사표 JSON 파일 저장소
//! - [`render`] — 조사표 PNG 렌더링
//! - [`export`] — 공용 사진 폴더 경로 유도와 파일 저장
//! - [`error`] — 공통 오류 타입

#![forbid(unsafe_code)]

pub mod domain;
pub mod error;
pub mod export;
pub mod render;
pub mod slots;
pub mod store;

pub use domain::*;
pub use error::{Error, Result};
