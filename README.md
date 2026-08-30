개인용 앱입니다.

공개 저장소인 이유는 GitHub Actions 를 무료로 쓰기 위해서입니다. 재사용을 상정하고 만들지 않았습니다.

모든 코드는 전적으로 제 기분에 따라 수정됩니다.

Issue는 제 필요에 의해 운용되며 대부분의 타인 이슈는 즉시 Close할 것입니다. 버그 제보는 받아들일 **수도** 있습니다.

라이선스는 MIT입니다. 그냥 님 멋대로 쓰라는 겁니다. 그러나 그 결과는 전혀 책임지지 않습니다. 번들된 Pretendard 폰트만 SIL OFL 1.1을 따릅니다. `crates/core/assets/fonts/LICENSE-Pretendard.txt`

### 앱 아이콘

원본 목업은 `src-tauri/icons/source/app-icon-mockup.png`. `scripts/iconize`(워크스페이스 밖의 작은 Rust 도구)가 목업에서 적응형 아이콘 레이어(전체 타일 / 배경 / 전경 글리프 / 단색)를 뽑아 `app-icon.json` 매니페스트로 만들고, `npx tauri icon src-tauri/icons/source/app-icon.json`이 데스크톱 아이콘과 Android mipmap(anydpi-v26 적응형·단색 포함)을 생성한다. 목업의 작은 제목 글자는 런처 크기에서 읽히지 않아 전경에서 제거한다.

```powershell
cargo run --release --manifest-path scripts/iconize/Cargo.toml -- src-tauri/icons/source/app-icon-mockup.png src-tauri/icons/source
npx tauri icon src-tauri/icons/source/app-icon.json
```
