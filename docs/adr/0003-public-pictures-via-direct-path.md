# ADR-0003: 공용 Pictures 폴더는 앱 외부 저장소 경로에서 파생해 직접 쓴다

- 상태: 채택 (2026-08-30) — 에뮬레이터/실기기 검증으로 확정, 실패 시 대안으로 전환

## 배경

Tauri 2의 `app.path().picture_dir()`는 Android에서 `getExternalFilesDir(DIRECTORY_PICTURES)` = `/storage/emulated/0/Android/data/<pkg>/files/Pictures`를 돌려준다. 이는 앱 전용 폴더라 갤러리에 보이지 않고 앱 삭제 시 함께 사라진다. 사용자는 기기 공용 사진 폴더 저장을 요구했다.

## 결정

- core의 `export::public_pictures_dir(app_pictures_dir)`가 경로에서 `Android` 구성요소를 찾아 그 부모(외부 저장소 루트)에 `Pictures`를 붙인다. `Android` 구성요소가 없으면(데스크톱 개발 환경) 입력 경로를 그대로 쓴다.
- 파일은 `<root>/Pictures/소비기한/<name>.png`에 `std::fs`로 직접 쓴다. Android 11+ 스코프드 스토리지는 앱이 공용 미디어 폴더에 **자기 파일을 새로 만드는 것**을 권한 없이 허용하고, FUSE가 MediaStore에 자동 등록한다.
- 그래서 `minSdk = 30`이다.

## 대안 (채택 안 함, 필요 시 전환)

- Kotlin Tauri 플러그인으로 `MediaStore.Images` insert — 정석이지만 Kotlin 코드·플러그인 크레이트가 추가된다. 직접 경로 쓰기가 기기에서 실패하면 이 경로로 간다.
- Rust `jni`로 MediaStore 호출 — `unsafe` 한 줄이 필요해 사용자의 unsafe 회피 원칙과 충돌한다.

## 결과

- core의 경로 파생은 순수 함수라 단위 테스트한다. 실제 쓰기 가능 여부는 Android 에뮬레이터(API 36)에서 `adb shell ls /sdcard/Pictures/소비기한`으로 확인한다.
