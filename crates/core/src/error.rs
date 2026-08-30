//! The single error type shared by every `core` module.
//!
//! The Tauri shell converts these to `String` via `Display`, so the messages are
//! user-facing Korean text rather than developer diagnostics.

use chrono::NaiveDateTime;

use crate::domain::Product;

/// Everything that can go wrong inside `core`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No sheet with this id exists in the store.
    #[error("조사표를 찾을 수 없습니다: {0}")]
    NotFound(String),
    /// The id is empty or contains characters that are unsafe in a file name.
    #[error("잘못된 조사표 id: {0}")]
    InvalidId(String),
    /// The instant is not one of the product's two daily expiry slot hours.
    #[error("{product}의 기한 시각이 아닙니다: {at}")]
    InvalidSlot { product: Product, at: NaiveDateTime },
    /// An entry carried a quantity below 1.
    #[error("수량은 1 이상이어야 합니다")]
    InvalidQuantity,
    /// Filesystem failure while reading or writing a sheet or an export.
    #[error("파일 오류: {0}")]
    Io(#[from] std::io::Error),
    /// A stored sheet could not be encoded or decoded.
    #[error("JSON 오류: {0}")]
    Json(#[from] serde_json::Error),
    /// The PNG renderer failed, e.g. the bundled font would not parse.
    #[error("이미지 렌더링 오류: {0}")]
    Render(String),
}

/// `core`'s result alias.
pub type Result<T> = std::result::Result<T, Error>;
