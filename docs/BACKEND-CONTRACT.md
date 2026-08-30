# 백엔드 계약 (core 크레이트 + Tauri shell 커맨드)

이 문서는 `crates/core`와 `src-tauri/src/{lib,commands}.rs`의 **확정된 인터페이스**다. 구현자는 이 시그니처·JSON 형태·규칙을 바꾸지 않는다. 바꿔야만 한다면 구현 보고서에 "계약 이탈" 항목으로 이유와 함께 적는다. 용어는 `CONTEXT.md`, 규칙 배경은 `docs/adr/`을 따른다.

## 0. 공통 규칙

- Rust edition 2024, `unsafe` 금지, 비테스트 코드에서 `unwrap()` 금지(`expect`도 번들 폰트 파싱처럼 실패가 프로그램 버그인 경우에만, 이유 문자열 필수).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`가 모두 통과해야 한다.
- 모든 시각은 `chrono::NaiveDateTime`(기기 로컬, 시간대 없음). serde 직렬화는 chrono 기본(`"2026-08-30T14:00:00"`).
- 모든 public 타입은 `Debug, Clone, PartialEq, Serialize, Deserialize`를 derive한다(Copy 가능한 것은 Copy도).
- core는 Tauri에 의존하지 않는다. `now`와 경로는 항상 인자로 받는다.

## 1. `crates/core` 모듈 구조

```
crates/core/src/
  lib.rs      pub mod domain; pub mod slots; pub mod store; pub mod render; pub mod export; pub mod error;
              pub use domain::*; pub use error::{Error, Result}; (편의 re-export)
  domain.rs   Product, Location, Entry, Section, Sheet, 정규화
  slots.rs    slot_options, 라벨 함수
  store.rs    SheetStore, SheetSummary, RETENTION_DAYS
  render.rs   render_png / render_rgba, IMAGE_WIDTH
  export.rs   public_pictures_dir, export_file_name, export_png, ExportResult, EXPORT_SUBDIR
  error.rs    Error, Result
crates/core/assets/fonts/PretendardStd-Regular.otf, PretendardStd-Bold.otf (이미 존재, include_bytes!)
```

`Cargo.toml`에는 `serde, serde_json, chrono, thiserror, tempfile(dev)`가 이미 있다. 렌더링용으로 `image`(png만), `imageproc`, `ab_glyph`를 **필요 최소 feature로** 추가한다(기본 feature를 끄고 png/드로잉/텍스트만). 순수 Rust 의존성만 허용(C 빌드 스크립트 금지 — Android 크로스 빌드 때문).

## 2. `domain.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Product { Onigiri, Gimbap, Lunchbox, Sandwich, Burger }
// JSON: "onigiri" | "gimbap" | "lunchbox" | "sandwich" | "burger"
// Ord = 위 선언 순서 = 화면·이미지 표시 순서

impl Product {
    pub const ALL: [Product; 5] = [Onigiri, Gimbap, Lunchbox, Sandwich, Burger];
    pub fn label(self) -> &'static str;   // "삼각김밥" "김밥" "도시락" "샌드위치" "햄버거"
    pub fn slot_hours(self) -> [u32; 2];  // Onigiri/Gimbap/Lunchbox → [2, 14], Sandwich/Burger → [10, 22]
    pub fn is_slot(self, at: NaiveDateTime) -> bool; // minute==0 && second==0 && nanosecond==0 && hour ∈ slot_hours
}
impl std::fmt::Display for Product { /* label() */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location { Store, WalkIn }     // JSON: "store" | "walk_in"
impl Location {
    pub const ALL: [Location; 2] = [Store, WalkIn];
    pub fn label(self) -> &'static str;   // "매장" "워크인"
}
impl std::fmt::Display for Location { /* label() */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry { pub at: NaiveDateTime, pub quantity: u32 }

/// 품목 키 5개가 항상 모두 존재한다(빈 Vec 허용). 정규화가 보장한다.
pub type Section = BTreeMap<Product, Vec<Entry>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    pub id: String,                      // created_at.format("%Y%m%d-%H%M%S")
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub sections: BTreeMap<Location, Section>,  // 키 2개 항상 존재
}

impl Sheet {
    pub fn new(now: NaiveDateTime) -> Sheet;      // id/created_at/updated_at = now, 모든 키를 빈 Vec으로 채움
    pub fn sheet_id(at: NaiveDateTime) -> String; // "%Y%m%d-%H%M%S"
    pub fn is_valid_id(id: &str) -> bool;         // 비어있지 않고 [0-9A-Za-z_-]만
    pub fn entries(&self, location: Location, product: Product) -> &[Entry]; // 키가 없으면 &[]
    pub fn is_section_empty(&self, location: Location) -> bool;
    pub fn entry_count(&self) -> u32;             // 모든 구역·품목의 항목 개수
    pub fn total_quantity(&self) -> u32;          // 수량 합(saturating)
    /// 정규화: 빠진 키 채움 → 각 항목 검증(quantity ≥ 1 아니면 Error::InvalidQuantity,
    /// product.is_slot(at) 아니면 Error::InvalidSlot) → at 오름차순 정렬 → 같은 at 병합(수량 saturating 합)
    /// → id 검증(Error::InvalidId) → updated_at = now. created_at은 유지.
    pub fn normalized(self, now: NaiveDateTime) -> Result<Sheet>;
}
```

JSON 예 (프론트엔드가 그대로 주고받는 형태):

```json
{
  "id": "20260830-080215",
  "created_at": "2026-08-30T08:02:15",
  "updated_at": "2026-08-30T08:10:00",
  "sections": {
    "store":   { "onigiri": [{ "at": "2026-08-30T14:00:00", "quantity": 12 }], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] },
    "walk_in": { "onigiri": [], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] }
  }
}
```

## 3. `slots.rs`

```rust
pub const DEFAULT_HORIZON_DAYS: u32 = 14;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotOptions { pub product: Product, pub dates: Vec<DateOption> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateOption { pub date: NaiveDate, pub label: String, pub times: Vec<TimeOption> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeOption { pub at: NaiveDateTime, pub hour: u32, pub label: String }

/// now.date()부터 horizon_days일 뒤 날짜까지, 각 날짜의 slot_hours 중 at > now 인 것만.
/// times가 빈 날짜는 dates에서 제외한다. dates·times 모두 오름차순.
pub fn slot_options(product: Product, now: NaiveDateTime, horizon_days: u32) -> SlotOptions;

pub fn date_label(date: NaiveDate) -> String;       // "8/30 (일)"  월·일 앞자리 0 없음, 요일 한글 한 글자(월화수목금토일)
pub fn time_label(hour: u32) -> String;             // 0→"오전 12시", 2→"오전 2시", 10→"오전 10시", 12→"오후 12시", 14→"오후 2시", 22→"오후 10시"
pub fn entry_label(at: NaiveDateTime) -> String;    // "8/30 14시"  (월/일 + 24시간제 두 자리 + "시") 예: "8/31 02시"
```

예시(ADR-0006): now=2026-08-01 01:00, Lunchbox → 첫 후보 2026-08-01 02:00. now=03:00 → 첫 후보 08-01 14:00. now=02:00 정각 → 02:00 제외. now=23:30 → 오늘 제외, 첫 날짜 08-02. Sandwich now=08-01 09:59 → 08-01 10:00. horizon_days=14이면 dates 길이는 최대 15(오늘 포함).

## 4. `store.rs`

```rust
pub const RETENTION_DAYS: i64 = 30;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSummary { pub id: String, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime, pub entry_count: u32, pub total_quantity: u32 }

#[derive(Debug, Clone)]
pub struct SheetStore { dir: PathBuf }
impl SheetStore {
    pub fn open(dir: impl Into<PathBuf>) -> Result<SheetStore>;  // create_dir_all
    pub fn dir(&self) -> &Path;
    pub fn path_of(&self, id: &str) -> Result<PathBuf>;           // is_valid_id 아니면 Error::InvalidId; dir/<id>.json
    pub fn save(&self, sheet: &Sheet) -> Result<()>;              // serde_json::to_vec_pretty → <id>.json.tmp 에 쓰고 rename(덮어쓰기)
    pub fn load(&self, id: &str) -> Result<Sheet>;                // 없으면 Error::NotFound(id)
    pub fn list(&self) -> Result<Vec<SheetSummary>>;              // *.json 전부 읽어 created_at 내림차순. 읽기/파싱 실패 파일은 건너뜀(전체 실패 금지)
    pub fn delete(&self, id: &str) -> Result<()>;                 // 없으면 Ok (멱등)
    pub fn purge_older_than(&self, now: NaiveDateTime, max_age: chrono::Duration) -> Result<Vec<String>>; // now - updated_at > max_age 인 것 삭제, 삭제한 id 반환(정렬 무관)
}
```

## 5. `render.rs`

```rust
pub const IMAGE_WIDTH: u32 = 1200;
pub fn render_rgba(sheet: &Sheet) -> Result<image::RgbaImage>;  // 높이는 내용으로 결정
pub fn render_png(sheet: &Sheet) -> Result<Vec<u8>>;            // render_rgba → PNG 인코딩
```

레이아웃(픽셀, 너비 1200 고정):

- 배경 흰색(255,255,255). 글자·선 검정(17,17,17). 폰트: Regular = PretendardStd-Regular.otf, Bold = PretendardStd-Bold.otf (`include_bytes!("../assets/fonts/...")`, `ab_glyph::FontRef::try_from_slice`, 실패 시 `Error::Render`).
- 여백 `MARGIN = 48`. 표 너비 = 1200 − 2·48 = 1104.
- 헤더: 왼쪽 제목 "소비기한 조사표" Bold 48px. 오른쪽 끝 정렬로 작성 시각 Regular 32px, 형식 `2026.08.30 (일) 오전 8:02` (`created_at`, 시는 앞자리 0 없음, 분은 두 자리, 오전/오후 규칙은 time_label과 동일). 헤더 높이 96, 그 아래 굵기 4px 가로줄.
- 표: 헤더 줄 아래 24px 띄우고 시작. 열 너비 [구역 200 | 품목 260 | 항목 644]. 모든 선 굵기 3px, 표 바깥 테두리 포함. 셀 안쪽 패딩 20px.
- 구역 순서 매장 → 워크인. 각 구역은 행 묶음 하나. 구역 셀(1열)은 그 묶음 전체 높이를 세로 병합하고 라벨(Bold 40px)을 **세로 가운데·가로 가운데** 정렬.
- 품목 행: 항목이 1개 이상인 품목만, `Product::ALL` 순서. 2열에 품목 라벨 Bold 40px(세로 가운데, 왼쪽 패딩). 3열에 항목을 한 줄에 하나씩 `entry_label(at) + " " + quantity + "개"` (예 `8/30 14시 12개`) Regular 40px, 줄 간격 56px, 위 패딩 20px. 행 높이 = max(96, 항목 수 × 56 + 40).
- 빈 구역(모든 품목 항목 0개): 2·3열을 가로 병합한 높이 240 행 하나. 병합 셀의 네 모서리를 잇는 두 대각선(굵기 4px)으로 X표. 구역 라벨은 그대로 표시.
- 총 높이 = 48 + 96 + 4 + 24 + 표 높이 + 48. 표 높이 = 각 구역 높이 합.
- 글자 세로 배치: `imageproc::drawing::text_size`로 잰 높이를 써서 셀/줄 안에서 가운데 맞춘다. 정확한 서브픽셀은 요구하지 않는다.

테스트(최소): (a) 빈 조사표 → 폭 1200, 높이 = 48+96+4+24+(240×2)+48 = 700, 두 구역 X 대각선 위 픽셀이 어둡고 셀 중앙에서 벗어난 점은 흰색. (b) 항목 3개짜리 품목 하나 → 행 높이 208 반영된 총 높이. (c) `render_png` 결과가 `image::load_from_memory`로 디코드되고 크기가 같다.

## 6. `export.rs`

```rust
pub const EXPORT_SUBDIR: &str = "소비기한";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportResult { pub path: String, pub file_name: String, pub bytes: u64 }

/// Android: ".../Android/data/<pkg>/files/Pictures" → ".../Pictures" (첫 "Android" 구성요소의 부모 + "Pictures").
/// "Android" 구성요소가 없으면 입력을 그대로 반환(데스크톱).
pub fn public_pictures_dir(app_pictures_dir: &Path) -> PathBuf;
pub fn export_file_name(sheet: &Sheet) -> String;  // "소비기한_2026-08-30_0802.png" (created_at)
/// pictures_dir/EXPORT_SUBDIR/ 를 만들고 render_png 결과를 export_file_name으로 저장(같은 이름은 덮어씀).
pub fn export_png(sheet: &Sheet, pictures_dir: &Path) -> Result<ExportResult>;
```

## 7. `error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("조사표를 찾을 수 없습니다: {0}")] NotFound(String),
    #[error("잘못된 조사표 id: {0}")] InvalidId(String),
    #[error("{product}의 기한 시각이 아닙니다: {at}")] InvalidSlot { product: Product, at: NaiveDateTime },
    #[error("수량은 1 이상이어야 합니다")] InvalidQuantity,
    #[error("파일 오류: {0}")] Io(#[from] std::io::Error),
    #[error("JSON 오류: {0}")] Json(#[from] serde_json::Error),
    #[error("이미지 렌더링 오류: {0}")] Render(String),
}
pub type Result<T> = std::result::Result<T, Error>;
```

## 8. Tauri shell (`src-tauri/src/commands.rs`, `lib.rs`)

- `pub struct AppState { pub store: SheetStore }` — `lib.rs`의 `.setup()`에서 `app.path().app_data_dir()?.join("sheets")`로 열고, `purge_older_than(now, Duration::days(RETENTION_DAYS))`를 호출한 뒤 `app.manage(AppState { store })`.
- `now`는 항상 `chrono::Local::now().naive_local()`. shell 어디에도 도메인 규칙을 두지 않는다.
- 커맨드 오류 타입은 `String`(`Error`의 Display). 커맨드 이름·인자 이름은 아래 그대로(프론트엔드가 `invoke(name, { ... })`로 호출).

| 커맨드 | 인자 | 반환 | 동작 |
|---|---|---|---|
| `list_sheets` | — | `Vec<SheetSummary>` | `store.list()` |
| `create_sheet` | — | `Sheet` | `Sheet::new(now)` → `store.save` → 반환 |
| `get_sheet` | `id: String` | `Sheet` | `store.load` |
| `save_sheet` | `sheet: Sheet` | `Sheet` | `sheet.normalized(now)?` → `store.save` → 정규화된 조사표 반환 |
| `delete_sheet` | `id: String` | `()` | `store.delete` |
| `slot_options` | `product: Product` | `SlotOptions` | `slots::slot_options(product, now, DEFAULT_HORIZON_DAYS)` |
| `export_sheet` | `id: String` | `ExportResult` | `store.load` → `app.path().picture_dir()` → `public_pictures_dir` → `export_png` |

- `lib.rs`의 `run()`은 `tauri::generate_handler![commands::list_sheets, …]`를 직접 쓴다(`commands.rs`의 placeholder `handler()`는 삭제).
- Tauri 2 API: `tauri::State<'_, AppState>`, `tauri::AppHandle`, `tauri::Manager` (`app.path()`, `app.manage`). `picture_dir()`는 Android에서 앱 전용 `Android/data/<pkg>/files/Pictures`를 돌려주므로 반드시 `public_pictures_dir`를 거친다(ADR-0003).

## 9. 테스트 목록 (core, `cargo test`)

- domain: `Sheet::new` 키 완비; `normalized` 정렬·병합·InvalidSlot·InvalidQuantity·InvalidId; serde 왕복(위 JSON 예와 동일 형태, `serde_json::to_value` 비교).
- slots: ADR-0006 예시 5개 + horizon 경계(마지막 날짜 = today + 14) + 라벨 3종.
- store: `tempfile::tempdir()`로 save/load 왕복, list 정렬, 깨진 파일 건너뜀, delete 멱등, purge가 30일 초과만 지우고 id 목록 반환.
- render: 5절 테스트 3개.
- export: `public_pictures_dir` Android 경로/데스크톱 경로, `export_file_name`, `export_png`가 tempdir에 파일을 만들고 `bytes`가 파일 크기와 같음.
