# 백엔드 구현 보고서 (BACKEND-CONTRACT.md 1–9절)

`crates/core` 전체와 `src-tauri/src/{commands,lib}.rs`를 계약대로 구현했다. 이 문서는
구현 결과, 검증 명령 결과, 계약 이탈 여부, 그리고 ADR-0003이 기대는 Android 스코프드
스토리지 동작의 근거를 남긴다.

## 1. 구현한 것

| 파일 | 내용 |
|---|---|
| `crates/core/src/lib.rs` | 모듈 선언, `pub use domain::*`, `pub use error::{Error, Result}`, `#![forbid(unsafe_code)]` |
| `crates/core/src/domain.rs` | `Product`(5) · `Location`(2) · `Entry` · `Section` · `Sheet`, `normalized` |
| `crates/core/src/slots.rs` | `slot_options`, `date_label` / `time_label` / `entry_label`, `DEFAULT_HORIZON_DAYS` |
| `crates/core/src/store.rs` | `SheetStore`(tmp+rename 원자적 저장), `SheetSummary`, `RETENTION_DAYS` |
| `crates/core/src/render.rs` | `render_rgba` / `render_png`, `IMAGE_WIDTH`, 표 레이아웃 전부 |
| `crates/core/src/export.rs` | `public_pictures_dir`, `export_file_name`, `export_png`, `ExportResult`, `EXPORT_SUBDIR` |
| `crates/core/src/error.rs` | `Error` 7종, `Result` |
| `src-tauri/src/commands.rs` | `AppState` + 커맨드 7개 |
| `src-tauri/src/lib.rs` | `setup()`에서 저장소 개방 · 30일 정리 · `manage`, `generate_handler!` 7개 |

`crates/core/Cargo.toml`에 렌더링 의존성을 최소 feature로 추가했다. 셋 다 순수 Rust라
C 빌드 스크립트가 없다(아래 검증 4번이 이를 확인한다).

```toml
image     = { version = "0.25", default-features = false, features = ["png"] }
imageproc = { version = "0.25", default-features = false }   # rayon 등 기본 feature 해제
ab_glyph  = { version = "0.2",  default-features = false, features = ["std"] }
```

`src-tauri/Cargo.toml`은 손대지 않았다. `picture_dir()` · `app_data_dir()`는 `tauri` 기본
기능이고, 앱이 직접 정의한 커맨드는 capability 항목이 필요 없다.

## 프론트엔드에 연결할 목록

`invoke(name, args)` 형태 그대로다. 오류는 전부 `String`(한국어 메시지).

| 커맨드 | 인자 | 반환 |
|---|---|---|
| `list_sheets` | — | `SheetSummary[]` (작성 시각 내림차순) |
| `create_sheet` | — | `Sheet` |
| `get_sheet` | `{ id }` | `Sheet` |
| `save_sheet` | `{ sheet }` | `Sheet` (정규화된 것) |
| `delete_sheet` | `{ id }` | `null` |
| `slot_options` | `{ product }` | `SlotOptions` |
| `export_sheet` | `{ id }` | `ExportResult` |

JSON 표기: `Product` = `onigiri|gimbap|lunchbox|sandwich|burger`,
`Location` = `store|walk_in`, 시각 = `"2026-08-30T14:00:00"`.

호출 인자 JSON은 정확히 다음 형태다.

```json
list_sheets/create_sheet: {}
get_sheet/delete_sheet/export_sheet: { "id": "20260830-080215" }
save_sheet: { "sheet": { "id": "20260830-080215", "created_at": "2026-08-30T08:02:15", "updated_at": "2026-08-30T08:10:00", "sections": { "store": { "onigiri": [{ "at": "2026-08-30T14:00:00", "quantity": 12 }], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] }, "walk_in": { "onigiri": [], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] } } } }
slot_options: { "product": "lunchbox" }
```

반환 JSON 형태(시각과 배열 길이는 예시)는 다음과 같다.

```json
list_sheets: [{ "id": "20260830-080215", "created_at": "2026-08-30T08:02:15", "updated_at": "2026-08-30T08:10:00", "entry_count": 1, "total_quantity": 12 }]
create_sheet/get_sheet/save_sheet: { "id": "20260830-080215", "created_at": "2026-08-30T08:02:15", "updated_at": "2026-08-30T08:10:00", "sections": { "store": { "onigiri": [], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] }, "walk_in": { "onigiri": [], "gimbap": [], "lunchbox": [], "sandwich": [], "burger": [] } } }
delete_sheet: null
slot_options: { "product": "lunchbox", "dates": [{ "date": "2026-08-30", "label": "8/30 (일)", "times": [{ "at": "2026-08-30T14:00:00", "hour": 14, "label": "오후 2시" }] }] }
export_sheet: { "path": "/storage/emulated/0/Pictures/소비기한/소비기한_2026-08-30_0802.png", "file_name": "소비기한_2026-08-30_0802.png", "bytes": 123456 }
```

실제 `Display` 오류 문자열 예: `조사표를 찾을 수 없습니다: missing`,
`잘못된 조사표 id: ../x`, `도시락의 기한 시각이 아닙니다: 2026-08-30 13:00:00`,
`수량은 1 이상이어야 합니다`, `파일 오류: ...`, `JSON 오류: ...`,
`이미지 렌더링 오류: ...`.

## 검증 로그 요약

다섯 명령을 그대로 실행했다. 전부 통과.

| # | 명령 | 결과 |
|---|---|---|
| 1 | `cargo fmt --all` | exit 0. 이어 실행한 `cargo fmt --all --check`도 무출력 통과 |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | `Finished \`dev\` profile ... in 1.59s` — 경고 0 |
| 3 | `cargo test --workspace` | `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (나머지 4개 타깃은 테스트 0개) |
| 4 | `cargo check -p cu-best-before-core --target aarch64-linux-android` | `Finished \`dev\` profile ... in 0.47s` |
| 5 | `cargo check --workspace` | `Finished \`dev\` profile ... in 1.35s` |

4번은 NDK/C 툴체인 없이 통과했다. 렌더링 의존성이 실제로 순수 Rust라는 뜻이고,
ADR-0002가 요구한 "Android 크로스 빌드를 막지 않는 의존성" 조건이 지켜졌다.

테스트 45개 구성(계약 9절):

- `domain` 9개 — 키 완비, 정렬·병합(포화 덧셈 포함), `InvalidSlot` / `InvalidQuantity` /
  `InvalidId`, 계약서 JSON 예와 동일 형태의 serde 왕복(`serde_json::to_value` 비교)
- `slots` 12개 — ADR-0006 예시 5개, horizon 경계(마지막 날짜 = today + 14, 길이 15/14),
  오름차순·비지 않음 불변식, 라벨 3종 + 요일 7종
- `store` 7개 — save/load 왕복(임시 파일 잔여 없음), list 정렬 + 깨진 파일·비 json 건너뜀,
  delete 멱등, purge가 30일 **초과**만 지우고(경계값은 남김) id 목록 반환, `updated_at` 기준 확인
- `render` 8개 — 계약 5절 (a)(b)(c) 3개 + 열 너비 합, 행 생략, 헤더 시각 포맷, 테두리, 병합 규칙
- `export` 5개 — Android/데스크톱/상대 경로, 파일명, tempdir 저장 + `bytes` 일치 + 덮어쓰기

### 규칙 준수

- `unsafe`: 두 크레이트 모두 `#![forbid(unsafe_code)]`. 컴파일러가 강제한다.
- `unwrap()`: 전체 0개. `unwrap_or` / `unwrap_or_default` / `is_none_or`만 쓴다.
- `expect`: 비테스트 코드에는 `src-tauri/src/lib.rs`의
  `.expect("error while running tauri application")` 하나뿐이다(기존 스캐폴드의
  Tauri 관용구, 이유 문자열 있음). 번들 폰트 파싱은 `expect`를 쓰지 않고
  계약 5절대로 `Error::Render`로 돌린다.

## 계약 이탈

**없다.** 시그니처·JSON 형태·상수·레이아웃 수치를 모두 그대로 구현했다.
다만 계약서 본문의 **예시 문자열 하나가 달력과 어긋난다**.

- 계약 5절 헤더 예시 `2026.08.30 (토) 오전 8:02`, 3절 `date_label` 예시 `8/30 (토)` (보고 후 계약서 예시는 `(일)`로 정정됨).
- 2026-08-30은 실제로 **일요일**이다(2026-08-29가 토요일).
- 고정된 규칙은 "요일 한글 한 글자(월화수목금토일)"라는 **포맷**이므로, 구현은 날짜에서
  요일을 계산한다. 즉 `date_label(2026-08-30) == "8/30 (일)"`이다.
- 구현이 계약의 규칙에서 벗어난 것은 아니고 예시 글자만 어긋난 것이라 이탈로 세지 않는다.
  테스트는 두 날짜(8/29 토, 8/30 일)를 함께 확인해 포맷과 요일 계산을 모두 고정한다.

참고로 `rustfmt`가 `lib.rs`의 `pub mod` 선언을 알파벳순으로 재정렬한다. 계약 1절의
나열 순서와 다르지만 `cargo fmt --all --check`가 강제하는 형식이라 그대로 두었다.

### 구현하며 확정한 세부 (계약이 열어 둔 부분)

- 빈 구역 X표는 병합 셀의 네 꼭짓점을 그대로 잇고, 굵기 4px는 선에 수직인 방향으로
  평행 이동한 Bresenham 선 4개를 겹쳐 만든다. 대각선 테스트는 픽셀 한두 개 흔들림을
  감안해 세로 ±3px를 훑는다.
- 표 선 그리기: 구역 경계선은 표 전체 너비를 가로지르고, 같은 구역 안 행 경계선은
  세로 병합된 구역 열(1열)을 건드리지 않는다. 빈 구역 행에는 품목/항목 경계선이 없다.
- `list`의 정렬은 `created_at` 내림차순이며 같은 시각이면 id 내림차순으로 안정화했다.
- `purge_older_than`은 `now - updated_at > max_age`(엄격 초과)라 정확히 30일은 남는다.
- `store.list()` / `purge_older_than`은 읽기·파싱 실패 파일을 조용히 건너뛴다.
- `SheetStore`의 clone들이 공유하는 쓰기 잠금으로 자동 저장 요청이 겹쳐도 동일한
  `<id>.json.tmp`를 동시에 건드리지 않는다. Windows의 rename 덮어쓰기 제한은 기존 파일을
  제거한 뒤 다시 rename하는 호환 경로로 처리한다.

## 조사 결과

### Android 공용 Pictures 직접 쓰기 (ADR-0003 근거)

`export::public_pictures_dir`는 `.../Android/data/<pkg>/files/Pictures`에서 첫 `Android`
구성요소의 부모를 취해 `Pictures`를 붙인다 →
`/storage/emulated/0/Pictures`. 그 아래 `소비기한/`을 만들고 `std::fs::write`로 PNG를 쓴다.
minSdk 30에서 이것이 성립하는 근거는 다음과 같다.

**문서로 확정된 것**

1. **직접 경로 접근이 API 30에서 복구되었다.** "To help your app work more smoothly with
   third-party media libraries, Android 11 allows you to use APIs other than the
   `MediaStore` API to access media files from shared storage using direct file paths" —
   허용 API로 "The `File` API"와 "Native libraries, such as `fopen()`"를 명시한다.
   Rust `std::fs`는 결국 `open(2)`/`write(2)`라 이 경로에 해당한다.
   <https://developer.android.com/about/versions/11/privacy/storage>,
   <https://developer.android.com/training/data-storage/shared/media>
2. **자기 파일에는 저장소 권한이 필요 없다.** "On devices that run Android 10 or higher,
   you don't need storage-related permissions to access and modify media files that your
   app owns" / "If you don't have any storage-related permissions, you can access files in
   your app-specific directory as well as media files that are attributed to your app
   using the `File` API." 앱이 새로 만드는 파일은 그 앱에 귀속되므로 여기 해당한다.
   <https://developer.android.com/training/data-storage/shared/media>
3. **`WRITE_EXTERNAL_STORAGE`는 API 30에서 무의미하다.** "If your app targets Android 11,
   both the `WRITE_EXTERNAL_STORAGE` permission and the `WRITE_MEDIA_STORAGE` privileged
   permission no longer provide any additional access." → 매니페스트에 넣을 이유가 없다.
   <https://developer.android.com/about/versions/11/privacy/storage>
4. **`requestLegacyExternalStorage`는 무시된다.** "After you update your app to target
   Android 11, the system ignores the `requestLegacyExternalStorage` flag." → 이 앱은
   레거시 저장소에 기대지 않으며 매니페스트 조작이 필요 없다.
   <https://developer.android.com/about/versions/11/privacy/storage>
5. **`Pictures/`는 이미지 컬렉션이다.** 시스템이 자동 스캔하는 well-defined 컬렉션 표에서
   `DCIM/`과 `Pictures/`의 이미지가 `MediaStore.Images`에 대응한다. 앨범 하위 폴더
   (`Pictures/MyVacationPictures` 예시)도 정상 사용 형태다 → `Pictures/소비기한/` OK.
   <https://developer.android.com/training/data-storage/shared/media>
6. **FUSE가 파일 시스템과 MediaProvider DB를 잇는다.** AOSP: Android 11+에서
   "MediaProvider becomes the file system handler (for FUSE) for external storage",
   "MediaProvider can intercept kernel calls", 목적은 "making the file system on external
   storage and the MediaProvider database consistent".
   <https://source.android.com/docs/core/storage/scoped>
   구현 측 진입점은 `MediaProvider.insertFileIfNecessaryForFuse` → `insertFileForFuse`이고,
   경로 기반 생성 동작은 CTS `ScopedStorageTest`가 검증한다.
   <https://android.googlesource.com/platform/packages/providers/MediaProvider/+/refs/heads/master/src/com/android/providers/media/MediaProvider.java>,
   <https://android.googlesource.com/platform/cts/+/037cc38f9ad/hostsidetests/scopedstorage/src/android/scopedstorage/cts/ScopedStorageTest.java>

**주의 / 아직 기기 확인이 필요한 것**

- "경로로 쓴 파일이 **즉시** 갤러리에 뜬다"를 한 문장으로 못 박은 공식 문서는 없다.
  DB 일관성은 AOSP FUSE 설계와 CTS로 뒷받침되지만, 갤러리 앱 UI 갱신에는 지연이 있을 수
  있다. ADR-0003이 예정한 에뮬레이터/실기기 확인
  (`adb shell ls /sdcard/Pictures/소비기한`)은 여전히 필요하다.
- 미디어 디렉터리는 파일 형식을 가린다. `Pictures/`에는 이미지만 쓸 수 있으므로 PNG는
  문제없지만, 나중에 JSON 같은 것을 같은 폴더에 쓰려 하면 실패한다.
- 직접 경로 쓰기는 FUSE를 거쳐 무작위 읽기·쓰기가 "up to twice as slow"다. 수백 KB짜리
  PNG를 한 번 쓰는 용도에는 영향이 없다.
- 다른 앱이 만든 파일을 **읽는** 것은 별개다(`READ_EXTERNAL_STORAGE` / API 33+
  `READ_MEDIA_IMAGES`). 이 앱은 자기가 만든 파일만 다루므로 해당 없다.
- 일부 OEM 펌웨어에서 동작이 다르다는 커뮤니티 보고가 있으나 문서로 확인된 바는 없다.
  실패하면 ADR-0003의 대안(Kotlin 플러그인으로 `MediaStore.Images` insert)으로 전환한다.

`public_pictures_dir`는 순수 함수라 Android 경로/데스크톱 경로/상대 경로를 단위 테스트로
고정해 두었다. 기기에서 확인할 것은 "그 경로에 쓸 수 있는가" 하나로 좁혀진다.

### 선택한 crate 버전과 이유

직접 의존성은 `image 0.25`(lockfile 0.25.10), `imageproc 0.25`(0.25.1),
`ab_glyph 0.2`(0.2.32)다. `image`는 PNG feature만, `imageproc`은 기본 feature를 끄고,
`ab_glyph`는 `std`만 켰다. 세 버전은 `draw_text_mut`/`text_size`가 `ab_glyph` 폰트를
받는 호환 조합이고, 불필요한 이미지 코덱·rayon·C 빌드를 제외하면서 Android aarch64
크로스 체크를 통과했다.

## 남은 위험

- Android API 30+의 직접 경로 쓰기와 FUSE/MediaStore 등록은 공식 문서·AOSP·CTS로
  확인했지만, Galaxy S26 Ultra/Android 16에서 실제 저장 및 갤러리 표시 시점은 아직
  에뮬레이터·실기기로 확인하지 않았다. 실패 시 ADR-0003의 MediaStore 플러그인 대안이 필요하다.
- 갤러리 UI는 MediaStore 등록 뒤에도 앱별 캐시 갱신 때문에 즉시 표시되지 않을 수 있다.
- 파일 저장은 프로세스 내 동시 요청을 직렬화하고 Windows 덮어쓰기를 처리하지만, 파일 제거와
  rename 사이에 프로세스가 종료되면 기존 파일이 사라질 수 있다. 개인용 단일 프로세스 앱에서
  범위는 작지만 완전한 Windows 원자 교체가 필요하면 플랫폼 전용 API나 버전 파일 전략이 필요하다.
