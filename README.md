개인용 앱입니다.

공개 저장소인 이유는 GitHub Actions 를 무료로 쓰기 위해서입니다. 재사용을 상정하고 만들지 않았습니다.

모든 코드는 전적으로 제 기분에 따라 수정됩니다.

Issue는 제 필요에 의해 운용되며 대부분의 타인 이슈는 즉시 Close할 것입니다. 버그 제보는 받아들일 **수도** 있습니다.

라이선스는 MIT입니다. 그냥 님 멋대로 쓰라는 겁니다. 그러나 그 결과는 전혀 책임지지 않습니다. 번들된 Pretendard 폰트만 SIL OFL 1.1을 따릅니다. `crates/core/assets/fonts/LICENSE-Pretendard.txt`

### 병합 방침

PR은 CI(rustfmt · clippy · cargo test · svelte-check)가 통과하면 **묻지 않고 바로 병합한다.** 작업을 대신하는 에이전트(Claude Code)도 마찬가지다. main에 올라가면 릴리즈 워크플로가 arm64 APK를 빌드해 릴리즈로 붙이므로, 병합이 곧 배포다. 병합 뒤에는 그 릴리즈 빌드가 끝날 때까지 지켜보고 결과를 알린다. 이는 저장소 소유자의 지시(2026-09-02)이며, 되돌릴 일이 생기면 revert PR을 같은 방식으로 병합한다.

에이전트가 이 방침대로 움직이려면 Claude Code 설정에 `gh pr merge` 허용 규칙이 있어야 한다. `.claude/settings.json`(저장소에 두면 어느 머신에서나 같다)에 아래를 넣는다.

```json
{ "permissions": { "allow": ["Bash(gh pr merge:*)"] } }
```
