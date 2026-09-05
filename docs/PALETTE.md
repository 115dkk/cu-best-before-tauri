# 화면·아이콘 색상

2026-09-06 변경: 차콜 바탕, 따뜻한 흰색 글자, 청록색 강조.

소비기한을 빠르게 읽는 도구이므로 화면 배치는 유지하고 색상만 변경한다.
기한 시각 휠은 상품 표기 및 조사 항목과 같은 24시간제 두 자리(`02시`, `14시`, `10시`, `22시`)를 사용한다.

## 참고 자료

- [WGSN·Coloro: 2026년 대표색 Transformative Teal](https://www.wgsn.com/en/blog/colour-year-2026-transformative-teal)
- [WGSN·Coloro: 2026년 봄·여름 주요 색상](https://www.wgsn.com/en/blog/coloro-x-wgsn-introduce-key-colours-s-s-26)

위 자료는 색상 유행 예측이며 앱 사용자 선호도 조사 결과는 아니다.
이 앱에서는 청록을 강조색으로 사용하고, 장시간 읽는 배경과 글자는 차분한 중간색으로 정했다.

## 색상 기준

| 용도 | 색상 |
| --- | --- |
| 앱·안드로이드 시작 화면 배경 | `#111615` |
| 시각 선택 시트 | `#1a211f` |
| 카드 | `#222b28` |
| 선택 영역 | `#2d3935` |
| 본문 | `#f2f1e9` |
| 보조 글자 | `#adb9b2` |
| 흐린 글자·아이콘 | `#94a49b` |
| 강조 | `#77cbbb` |
| 강조 버튼 위 글자 | `#10241f` |

불투명한 카드 위 본문 대비는 약 12.8:1, 강조 버튼의 글자 대비는 약 8.5:1이다.
휠 가장자리의 마스크·비활성 버튼에는 투명도가 별도로 적용된다.
삭제·오류는 기존의 붉은색으로 구분한다. 안드로이드의 테마 아이콘은 운영체제가 색을 정한다.

## 아이콘 재생성

원본 `src-tauri/icons/source/app-icon-mockup.png`의 문서·시계 모양을 유지하면서 내장 이미지 편집 도구로 색조를 변경했다.
편집 지시는 차콜 타일, 따뜻한 흰색 문서, 청록색 시계·접힌 모서리로 바꾸고 나머지 구도와 문자를 유지하는 것이었다.
앱에 쓰는 작은 아이콘에서는 기존 추출 도구가 제목 글자를 제거한다.

```sh
cargo run --release --locked --manifest-path scripts/iconize/Cargo.toml -- src-tauri/icons/source/app-icon-mockup.png src-tauri/icons/source
npm run tauri -- icon src-tauri/icons/source/app-icon.json
```

이 저장소에서는 Tauri CLI가 안드로이드 프로젝트를 감지해 `src-tauri/gen/android/app/src/main/res/mipmap-*/`의 아이콘을 직접 갱신한다.
아이콘 생성 후에도 `values/colors.xml`의 `app_background`가 앱 배경과 같은지 확인한다.
