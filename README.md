개인용 앱입니다.

공개 저장소인 이유는 GitHub Actions 를 무료로 쓰기 위해서입니다. 재사용을 상정하고 만들지 않았습니다.

모든 코드는 전적으로 제 기분에 따라 수정됩니다.

Issue는 제 필요에 의해 운용되며 대부분의 타인 이슈는 즉시 Close할 것입니다. 버그 제보는 받아들일 **수도** 있습니다.

라이선스는 MIT입니다. 다만 이것은 격식을 갖춘 "사용 허가"라기보다 **"알아서 마음대로 하라"**에 가깝습니다. 허락을 구하거나 알릴 필요 없이 복사·수정·재배포·판매 무엇이든 하세요. 대신 그 결과에 대해 저는 아무 책임도 지지 않고, 아무것도 보장하지 않습니다. (번들된 Pretendard 폰트만 SIL OFL 1.1을 따릅니다 — `crates/core/assets/fonts/LICENSE-Pretendard.txt`.)

---

## 무엇을 하는 앱인가

편의점 다섯 품목(삼각김밥·김밥·도시락·샌드위치·햄버거)의 소비기한을 매장/워크인 구역별로 적고, 표 한 장짜리 PNG로 `Pictures/소비기한/`에 저장하는 개인용 Android 앱. 요구사항은 [docs/SPEC.md](docs/SPEC.md), 용어는 [CONTEXT.md](CONTEXT.md), 결정 기록은 [docs/adr](docs/adr)에 있다.

## 구조

| 경로 | 역할 |
| --- | --- |
| `crates/core` | 도메인 로직 전부(품목·슬롯·조사표 저장·PNG 렌더링). Tauri 의존 없음, `cargo test`로 검증 |
| `src-tauri` | Tauri 2 shell. 커맨드가 core를 호출하는 얇은 어댑터 |
| `src` | Svelte 5 + TypeScript 화면 |
| `.github/workflows` | `ci.yml`(fmt·clippy·test·타입검사, PR에는 cargo check 추가), `release.yml`(main push → APK 서명·릴리즈) |

## 개발

```powershell
npm install
npm run check                 # svelte-check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npx tauri dev                 # 데스크톱 창에서 화면 확인

# Android (ANDROID_HOME / NDK_HOME / JAVA_HOME 필요)
npx tauri android dev
npx tauri android build --apk --target aarch64
```

릴리즈 서명 키는 저장소 Secrets(`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`)에서 온다.

### 에뮬레이터/기기 확인

- CI의 **Android debug APK** 워크플로(`workflow_dispatch`, x86_64/aarch64 선택)가 디버그 APK 아티팩트를 만든다. `gh run download <run-id>` 후 `adb install`. 러너마다 디버그 서명 키가 새로 생성되므로 이전 CI 빌드가 설치돼 있으면 `adb uninstall dev.dkk115.cubestbefore` 후 설치한다(`install -r`는 서명 불일치로 조용히 실패한다).
- 로컬 Windows 빌드(`scripts/android-debug-build.ps1`)는 Tauri CLI의 심볼릭 링크 단계와 Gradle 변환 캐시 이동이 이 환경(개발자 모드 꺼짐, 비관리자 셸)에서 막혀 완주하지 못했다. 개발자 모드가 켜진 PC에서는 `npx tauri android build --debug --target x86_64 --apk`가 그대로 된다.
- 2026-08-30 API 36 에뮬레이터 실측: 조사표 생성·항목 추가·자동 저장·PNG 내보내기(`/sdcard/Pictures/소비기한/…png`, MediaStore 자동 등록, 소유 패키지 = 이 앱) 모두 정상.
