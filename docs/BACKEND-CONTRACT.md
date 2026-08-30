# 백엔드 계약 (core 크레이트 + Tauri shell 커맨드)

이 문서는 `crates/core`와 `src-tauri/src/{lib,commands}.rs`의 **확정된 인터페이스**다. 구현자는 이 시그니처·JSON 형태·규칙을 바꾸지 않는다. 바꿔야만 한다면 구현 보고서에 "계약 이탈" 항목으로 이유와 함께 적는다. 용어는 `CONTEXT.md`, 규칙 배경은 `docs/adr/`을 따른다.

> 2026-08-30 아키텍처 리뷰(ADR-0007, ADR-0008) 반영판. 첫 구현 보고서는 `docs/BACKEND-REPORT.md`.

## 0. 공통 규칙

- Rust edition 2024, `unsafe` 금지(`#![forbid(unsafe_code)]`), 비테스트 코드에서 `unwrap()` 금지(`expect`도 실패가 프로그램 버그인 경우에만, 이유 문자열 필수).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`가 모두 통과해야 한다.
- 모든 시각은 `chrono::NaiveDateTime`(기기 로컬, 시간대 없음). serde 직렬화는 chrono 기본(`"2026-08-30T14:00:00"`).
- 모든 public 타입은 `Debug, Clone, PartialEq, Serialize, Deserialize`를 derive한다(Copy 가능한 것은 Copy도).
- core는 Tauri에 의존하지 않는다. `now`와 경로는 항상 인자로 받는다.
- 화면에 보이는 문자열은 core가 만든다(ADR-0008). 프론트엔드는 라벨을 계산하지 않는다.

## 1. `crates/core` 모듈 구조

```
crates/core/src/
  lib.rs      pub mod domain; error; export; render; slots; store; view;  pub use domain::*; pub use error::{Error, Result};
  domain.rs   Product, Location, Entry, Section, Sheet, MAX_QUANTITY, 정규화
  slots.rs    slot_options / slot_options_with / default_slot_options, 라벨 함수
  store.rs    SheetStore(open / open_in / create / purge_expired …), SheetSummary, RETENTION_DAYS, SHEETS_SUBDIR
  render.rs   render_png / render_rgba, IMAGE_WIDTH
  export.rs   public_pictures_dir, export_file_name, export_png, ExportResult, EXPORT_SUBDIR
  view.rs     EntryView, SectionView, SheetView, Catalog, CatalogItem
  error.rs    Error, Result
crates/core/assets/fonts/Pretendard-Regular.otf, Pretendard-Bold.otf (include_bytes!)
```

의존성: `serde, serde_json, chrono, thiserror`, 렌더링용 `image`(png만) · `imageproc`(기본 feature 해제) · `ab_glyph`, dev `tempfile`. 순수 Rust만(C 빌드 스크립트 금지 — Android 크로스 빌드 때문).

## 2. `domain.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Product { Onigiri, Gimbap, Lunchbox, Sandwich, Burger }
// JSON: "onigiri" | "gimbap" | "lunchbox" | "sandwich" | "burger"
// Ord = 위 선언 순서 = 화면·이미지 표시 순서

impl Product {
    pub const ALL: [Product; 5];
    pub fn label(self) -> &'static str;   // "삼각김밥" "김밥" "도시락" "샌드위치" "햄버거"
    pub fn slot_hours(self) -> [u32; 2];  // Onigiri/Gimbap/Lunchbox → [2, 14], Sandwich/Burger → [10, 22]
    pub fn is_slot(self, at: NaiveDateTime) -> bool; // minute==0 && second==0 && nanosecond==0 && hour ∈ slot_hours
}
impl std::fmt::Display for Product { /* label() */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location { Store, WalkIn }     // JSON: "store" | "walk_in"
impl Location { pub const ALL: [Location; 2]; pub fn label(self) -> &'static str; /* "매장" "워크인" */ }
impl std::fmt::Display for Location { /* label() */ }

/// 항목 하나의 수량 상한. 정규화가 초과를 거부하고, 같은 슬롯 병합은 여기서 포화한다.
pub const MAX_QUANTITY: u32 = 999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry { pub at: NaiveDateTime, pub quantity: u32 }

/// 품목 키 5개가 항상 모두 존재한다(빈 Vec 허용). 정규화가 보장한다.
pub type Section = BTreeMap<Product, Vec<Entry>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    pub id: String,                      // created_at.format("%Y%m%d-%H%M%S") (+ "-2", "-3"… 충돌 시)
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub sections: BTreeMap<Location, Section>,  // 키 2개 항상 존재
}

impl Sheet {
    pub fn new(now: NaiveDateTime) -> Sheet;      // id/created_at/updated_at = now, 모든 키를 빈 Vec으로 채움
    pub fn sheet_id(at: NaiveDateTime) -> String; // "%Y%m%d-%H%M%S"
    pub fn is_valid_id(id: &str) -> bool;         // 비어있지 않고 [0-9A-Za-z_-]만
    pub fn entries(&self, location: Location, product: Product) -> &[Entry]; // 키가 없으면 &[]
    pub fn is_section_empty(&self, location: Location) -> bool;  // 렌더링의 "빈 구역" 판정도 이것을 쓴다
    pub fn entry_count(&self) -> u32;
    pub fn total_quantity(&self) -> u32;          // saturating
    /// 정규화: 빠진 키 채움 → 각 항목 검증(quantity ∉ 1..=MAX_QUANTITY → Error::InvalidQuantity,
    /// product.is_slot(at) 아니면 Error::InvalidSlot) → at 오름차순 정렬 → 같은 at 병합(MAX_QUANTITY에서 포화)
    /// → id 검증(Error::InvalidId) → updated_at = now. created_at은 유지. 저장 형식은 라벨 없는 `Sheet`다.
    pub fn normalized(self, now: NaiveDateTime) -> Result<Sheet>;
}
```

저장 JSON 예:

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

pub struct SlotOptions { pub product: Product, pub dates: Vec<DateOption> }
pub struct DateOption { pub date: NaiveDate, pub label: String, #[serde(default)] pub past: bool, pub times: Vec<TimeOption> }
pub struct TimeOption { pub at: NaiveDateTime, pub hour: u32, pub label: String }

/// now.date()부터 horizon_days일 뒤 날짜까지, 각 날짜의 slot_hours 중 at > now 인 것만. 빈 날짜 제외, 오름차순.
pub fn slot_options(product: Product, now: NaiveDateTime, horizon_days: u32) -> SlotOptions;
/// 위에 더해 `include`가 이 품목의 슬롯이면 후보에 없더라도 끼워 넣는다(편집 중인 항목용).
/// 새로 끼워 넣은 날짜가 now 이전이면 past = true. 품목의 슬롯이 아닌 시각은 무시.
pub fn slot_options_with(product, now, horizon_days, include: Option<NaiveDateTime>) -> SlotOptions;
/// shell 진입점: DEFAULT_HORIZON_DAYS 고정.
pub fn default_slot_options(product, now, include: Option<NaiveDateTime>) -> SlotOptions;

pub fn date_label(date: NaiveDate) -> String;       // "8/30 (일)"  월·일 앞자리 0 없음, 요일 한글 한 글자
pub fn meridiem(hour: u32) -> (&'static str, u32);  // (오전|오후, 12시간제 시) 0→(오전,12), 12→(오후,12)
pub fn time_label(hour: u32) -> String;             // "오전 2시" "오후 10시"
pub fn sheet_label(at: NaiveDateTime) -> String;    // "8/30 (일) 오전 8:02" (조사표 작성 시각, 화면용)
pub fn entry_label(at: NaiveDateTime) -> String;    // "8/30 14시"  (24시간제 두 자리; PNG와 화면 공용)
```

예시(ADR-0006): now=2026-08-01 01:00, Lunchbox → 첫 후보 08-01 02:00. now=03:00 → 08-01 14:00. now=02:00 정각 → 02:00 제외. now=23:30 → 오늘 제외. Sandwich now=09:59 → 10:00. horizon 14 → dates 최대 15.

## 4. `store.rs`

```rust
pub const RETENTION_DAYS: i64 = 30;
pub const SHEETS_SUBDIR: &str = "sheets";

pub struct SheetSummary { pub id: String, pub created_at: NaiveDateTime, pub created_label: String, pub updated_at: NaiveDateTime, pub entry_count: u32, pub total_quantity: u32 }

pub struct SheetStore { /* dir + 프로세스 내 쓰기 잠금 */ }
impl SheetStore {
    pub fn open(dir: impl Into<PathBuf>) -> Result<SheetStore>;         // create_dir_all
    pub fn open_in(app_data_dir: impl AsRef<Path>) -> Result<SheetStore>; // app_data_dir/SHEETS_SUBDIR
    pub fn dir(&self) -> &Path;
    pub fn path_of(&self, id: &str) -> Result<PathBuf>;
    pub fn create(&self, now: NaiveDateTime) -> Result<Sheet>;         // Sheet::new(now); 같은 초 충돌 시 "-2","-3"…; 저장 후 반환
    pub fn save(&self, sheet: &Sheet) -> Result<()>;                   // tmp + rename(원자적, Windows 덮어쓰기 처리)
    pub fn load(&self, id: &str) -> Result<Sheet>;                     // 없으면 Error::NotFound(id)
    pub fn list(&self) -> Result<Vec<SheetSummary>>;                   // created_at 내림차순, 깨진 파일은 건너뜀
    pub fn delete(&self, id: &str) -> Result<()>;                      // 멱등
    pub fn purge_expired(&self, now: NaiveDateTime) -> Result<Vec<String>>;              // RETENTION_DAYS 적용
    pub fn purge_older_than(&self, now: NaiveDateTime, max_age: chrono::Duration) -> Result<Vec<String>>;
}
```

## 5. `render.rs`

```rust
pub const IMAGE_WIDTH: u32 = 1200;
pub fn render_rgba(sheet: &Sheet) -> Result<image::RgbaImage>;
pub fn render_png(sheet: &Sheet) -> Result<Vec<u8>>;
```

레이아웃(픽셀, 너비 1200 고정): 여백 48, 헤더 96(제목 Bold 48 "소비기한 조사표", 오른쪽 끝 Regular 32 `2026.08.30 (일) 오전 8:02`), 밑줄 4, 간격 24, 표 열 [200 | 260 | 644], 선 3, 셀 패딩 20. 구역 셀은 세로 병합·가운데 정렬(Bold 40). 품목 행은 항목 있는 품목만(`Product::ALL` 순), 항목은 한 줄에 하나 `entry_label + " " + quantity + "개"`(Regular 40, 줄 56, 위 패딩 20), 행 높이 = max(96, n×56+40). 빈 구역(`Sheet::is_section_empty`)은 2·3열 병합 240 행에 대각선 X(굵기 4). 총 높이 = 48+96+4+24+표+48.

## 6. `export.rs`

```rust
pub const EXPORT_SUBDIR: &str = "소비기한";
pub struct ExportResult { pub path: String, pub file_name: String, pub bytes: u64 }
pub fn public_pictures_dir(app_pictures_dir: &Path) -> PathBuf;  // ".../Android/data/<pkg>/files/Pictures" → ".../Pictures"; Android 없으면 그대로
pub fn export_file_name(sheet: &Sheet) -> String;                // "소비기한_2026-08-30_0802.png"
pub fn export_png(sheet: &Sheet, pictures_dir: &Path) -> Result<ExportResult>; // pictures_dir/소비기한/<name>, 덮어쓰기
```

## 7. `view.rs` (ADR-0008)

```rust
pub struct EntryView { pub at: NaiveDateTime, pub quantity: u32, pub label: String }   // label = entry_label(at)
pub type SectionView = BTreeMap<Product, Vec<EntryView>>;
pub struct SheetView { pub id: String, pub created_at: NaiveDateTime, pub created_label: String /* sheet_label */, pub updated_at: NaiveDateTime, pub sections: BTreeMap<Location, SectionView> /* 키 완비 */ }
impl From<&Sheet> for SheetView;   // 빠진 키를 빈 Vec으로 채운다

pub struct CatalogItem<K> { pub key: K, pub label: String }
pub struct Catalog { pub products: Vec<CatalogItem<Product>>, pub locations: Vec<CatalogItem<Location>> }
impl Catalog { pub fn current() -> Catalog }  // Product::ALL / Location::ALL 순서
```

뷰 JSON은 저장 JSON의 상위 집합이다(`created_label`, 항목의 `label`이 추가). `Sheet`는 모르는 필드를 무시하므로 프론트엔드는 뷰를 그대로 `save_sheet`에 보낸다.

## 8. `error.rs`

```rust
pub enum Error {
    #[error("조사표를 찾을 수 없습니다: {0}")] NotFound(String),
    #[error("잘못된 조사표 id: {0}")] InvalidId(String),
    #[error("{product}의 기한 시각이 아닙니다: {at}")] InvalidSlot { product: Product, at: NaiveDateTime },
    #[error("수량은 1 이상 {max} 이하여야 합니다", max = MAX_QUANTITY)] InvalidQuantity,
    #[error("파일 오류: {0}")] Io(#[from] std::io::Error),
    #[error("JSON 오류: {0}")] Json(#[from] serde_json::Error),
    #[error("이미지 렌더링 오류: {0}")] Render(String),
}
pub type Result<T> = std::result::Result<T, Error>;
```

## 9. Tauri shell (`src-tauri/src/commands.rs`, `lib.rs`)

- `pub struct AppState { pub store: SheetStore }` — `setup()`에서 `SheetStore::open_in(app.path().app_data_dir()?)`, `purge_expired(now)`, `app.manage`.
- `now`는 항상 `chrono::Local::now().naive_local()`. shell은 도메인 상수를 조립하지 않는다(정책은 core 진입점이 적용).
- 커맨드 오류 타입은 `String`(`Error`의 Display).

| 커맨드 | 인자 | 반환 | 동작 |
|---|---|---|---|
| `catalog` | — | `Catalog` | `Catalog::current()` |
| `list_sheets` | — | `Vec<SheetSummary>` | `store.list()` |
| `create_sheet` | — | `SheetView` | `store.create(now)` |
| `get_sheet` | `id: String` | `SheetView` | `store.load` → 뷰 |
| `save_sheet` | `sheet: Sheet` (뷰 JSON 허용) | `SheetView` | `normalized(now)` → `store.save` → 뷰 |
| `delete_sheet` | `id: String` | `()` | `store.delete` |
| `slot_options` | `product: Product, include: Option<NaiveDateTime>` | `SlotOptions` | `default_slot_options(product, now, include)` |
| `export_sheet` | `id: String` | `ExportResult` | `store.load` → `picture_dir()` → `public_pictures_dir` → `export_png` |

프론트엔드 호출 예: `invoke("slot_options", { product: "lunchbox", include: "2026-08-29T14:00:00" | null })`.

## 10. 테스트 (core, `cargo test`, 55개)

- domain: 키 완비, 정렬·병합·상한 포화, InvalidSlot/InvalidQuantity(0과 1000)/InvalidId, serde 왕복.
- slots: ADR-0006 예시 5개, horizon 경계, include(과거 날짜 삽입·기존 날짜 병합·중복 없음·비슬롯 무시), 라벨(date/time/entry/sheet/요일).
- store: save/load 왕복, list 정렬·깨진 파일 건너뜀, delete 멱등, purge 경계, `open_in`, `create` id 충돌 회피, `purge_expired`.
- render: 빈 조사표 700px + X표, 항목 3개 행 높이, PNG 디코드, 테두리·병합 규칙, 헤더 시각.
- export: 경로 파생, 파일명, 저장·덮어쓰기.
- view: 키 채움·라벨, 뷰→Sheet 왕복, 카탈로그 순서.
