# ADR-0005: CI 분리와 릴리즈 서명

- 상태: 채택 (2026-08-30)

## 결정

- `ci.yml` — 모든 브랜치 push와 PR에서 실행. 순서: rustfmt → clippy(`--all-targets -D warnings`) → `cargo test` → `svelte-check`(TypeScript). 하나라도 실패하면 RED. PR 이벤트에서는 `cargo check --workspace --locked` 잡이 추가된다.
- `release.yml` — `main` push에서만 실행. 체크를 다시 통과한 뒤 `tauri android build --apk --target aarch64`로 arm64 APK를 만들고, `zipalign` + `apksigner`로 서명해 GitHub Release(`v<version>-r<run_number>`)에 첨부한다.
- 서명 키는 저장소 Secrets(`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`)에서 온다. 키스토어 원본은 저장소 밖(`%USERPROFILE%\.android\cu-best-before-release.jks`)에 둔다.
- Miri 잡은 두지 않는다. `unsafe`가 없기 때문이며, 생기면 그때 추가한다.
- 액션은 항상 최신 메이저(node24 런타임: checkout@v7, setup-node@v7, setup-java@v6, setup-android@v4, action-gh-release@v3, rust-cache@v2.9+)를 쓴다. GitHub의 Node 20 런타임 지원 중단 경고를 피하기 위해서다. 런너 Node는 24.

## 이유

- 사용자 요구: 빌드보다 먼저 검사해 실패 시 RED, PR에는 빌드 대신 `cargo check`, main push에 자동 빌드·릴리즈.
- 서명 키가 고정되어야 폰에서 기존 앱을 지우지 않고 업데이트된다.
- `gen/android`를 손대지 않고 사후 서명하면 Tauri가 생성한 Gradle 설정을 그대로 유지한다.

## 결과

- Linux 러너에서 Tauri 데스크톱 크레이트를 검사하려면 `libwebkit2gtk-4.1-dev` 등 GTK 의존성을 설치해야 한다(clippy/check 잡).
- 릴리즈 태그에 run_number가 들어가므로 버전 번호를 올리지 않아도 매 merge마다 릴리즈가 만들어진다.
