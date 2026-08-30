# ADR-0001: Tauri 2 + Svelte 5 화면, 로직은 전부 Rust core 크레이트

- 상태: 채택 (2026-08-30)

## 결정

- 화면은 Tauri 2 WebView 위의 Svelte 5 + TypeScript(SvelteKit 없이 Vite 단독). 라우팅이 필요 없는 3화면 앱이라 SPA 한 장으로 충분하다.
- 로직은 Cargo 워크스페이스의 `crates/core`(Tauri 의존 없음)에 둔다. `src-tauri`는 커맨드 어댑터만 가진 얇은 shell이다.
- 프론트엔드는 계산을 하지 않는다. 후보 슬롯·정규화·렌더링·저장 모두 core가 하고, 화면은 shell이 돌려준 JSON을 편집해 되돌려 보낸다.

## 이유

- 사용자가 "로직은 어차피 써야 하는 Rust로 전부" 쓰기를 원했다.
- core의 인터페이스가 곧 테스트 표면이 되어 Android 없이 `cargo test`만으로 동작을 검증한다(깊은 모듈).
- shell을 얇게 유지하면 Tauri 버전 변화의 영향 범위가 한 파일에 갇힌다(국소성).

## 결과

- core는 `now: NaiveDateTime`과 디렉터리 `Path`를 인자로 받는다. 시각·경로를 읽는 코드는 shell에만 있다.
- 백엔드(core + shell 커맨드)는 GPT 워커에 위임 가능한 자기완결 계약(`docs/BACKEND-CONTRACT.md`)으로 명세된다. 프론트엔드는 Claude가 직접 작성한다.
