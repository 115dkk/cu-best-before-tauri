# ADR-0003: 공용 Pictures 폴더는 앱 외부 저장소 경로에서 파생해 직접 쓴다

- 상태: 채택 (2026-08-30), 실기기 검증 후 보강 — 직접 쓰기 + **명시적 MediaStore 스캔**

## 배경

Tauri 2의 `app.path().picture_dir()`는 Android에서 `getExternalFilesDir(DIRECTORY_PICTURES)` = `/storage/emulated/0/Android/data/<pkg>/files/Pictures`를 돌려준다. 이는 앱 전용 폴더라 갤러리에 보이지 않고 앱 삭제 시 함께 사라진다. 사용자는 기기 공용 사진 폴더 저장을 요구했다.

## 결정

- core의 `export::public_pictures_dir(app_pictures_dir)`가 경로에서 `Android` 구성요소를 찾아 그 부모(외부 저장소 루트)에 `Pictures`를 붙인다. `Android` 구성요소가 없으면(데스크톱 개발 환경) 입력 경로를 그대로 쓴다.
- 파일은 `<root>/Pictures/소비기한/<name>.png`에 `std::fs`로 직접 쓴다. Android 11+ 스코프드 스토리지는 앱이 공용 미디어 폴더에 **자기 파일을 새로 만드는 것**을 권한 없이 허용하고, FUSE가 MediaStore에 자동 등록한다.
- 그래서 `minSdk = 30`이다.

## 대안 (채택 안 함, 필요 시 전환)

- Kotlin Tauri 플러그인으로 `MediaStore.Images` insert — 정석이지만 Kotlin 코드·플러그인 크레이트가 추가된다. 직접 경로 쓰기가 기기에서 실패하면 이 경로로 간다.
- Rust `jni`로 MediaStore 호출 — `unsafe` 한 줄이 필요해 사용자의 unsafe 회피 원칙과 충돌한다.

## 보강 (실기기 결과)

- API 36 에뮬레이터는 직접 경로로 쓴 파일을 FUSE가 자동 등록했지만, **Galaxy S26 Ultra / Android 16은 파일은 생기되 갤러리에 잡히지 않았다**(루트 탐색기로 파일 존재 확인).
- 그래서 내보내기 직후 shell이 Kotlin 플러그인 `MediaScanPlugin`(앱 모듈 안, `gen/android/app/src/main/java/dev/dkk115/cubestbefore/`)의 `scanFile`을 호출해 `MediaStore.scanFile`로 등록한다. Rust 쪽은 `src-tauri/src/media_scan.rs`(Android에서만 활성, 데스크톱은 no-op). 별도 플러그인 크레이트나 `unsafe`는 없다.
- `ExportResult.media_uri`에 등록된 content uri를 담아 화면이 '갤러리에 저장됨'을 구분해 보여준다.

## 보강 2 (2026-09-02): 다시 내보내면 새 파일을 만든다

- 같은 이름으로 덮어쓰고 다시 스캔하면 MediaStore는 기존 행을 갱신할 뿐 새 사진으로 취급하지 않아, 갤러리 맨 앞에 올라오지 않았다(실기기). 며칠 뒤 고친 조사표를 다시 저장하면 찾기 어려웠다.
- 그래서 `export_png`는 같은 조사표의 내보내기마다 새 이름(`<줄기>.png`, `<줄기> (2).png`, `(3)` …)으로 쓰고, 이전 파일은 지운다. 새 파일을 먼저 쓴 뒤 지우므로 실패해도 파일이 하나는 남는다. 지운 경로는 `ExportResult.removed`로 올라오고, shell이 `MediaScanPlugin.forgetFile`(없는 경로를 다시 스캔해 행을 내리는 스캐너 관례)로 인덱스에서도 내린다.

## 결과

- core의 경로 파생은 순수 함수라 단위 테스트한다. 실제 쓰기 가능 여부는 Android 에뮬레이터(API 36)에서 `adb shell ls /sdcard/Pictures/소비기한`으로 확인한다.
