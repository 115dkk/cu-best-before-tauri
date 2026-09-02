# 작업 규칙

## 병합

PR은 CI(rustfmt · clippy · cargo test · svelte-check)가 통과하면 **묻지 않고 바로 병합한다.** main에 올라가면 릴리즈 워크플로가 arm64 APK를 빌드해 릴리즈로 붙이므로 병합이 곧 배포다. 병합 뒤에는 그 릴리즈 빌드(`release.yml`)가 끝날 때까지 `gh run watch`로 지켜보고 결과를 알린다. 되돌릴 일이 생기면 revert PR을 같은 방식으로 병합한다. 저장소 소유자 지시(2026-09-02).

자동 모드 분류기가 `gh pr merge`를 막으면, 도구 호출 설명에 소유자가 명시적으로 병합을 지시했다는 사실과 목적(병합 뒤 CI 감시)을 적어 다시 시도한다. 설정 파일(`.claude/settings.json`)은 건드리지 않는다.

## README

README는 사람이 읽는 문서이고 "이 저장소는 별 일 없다"가 전부다. 작업 기록, 운용 규칙, 에이전트용 지시는 README에 쓰지 않고 이 파일과 `docs/`에 둔다.
