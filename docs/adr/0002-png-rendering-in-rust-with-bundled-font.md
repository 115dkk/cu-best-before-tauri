# ADR-0002: PNG는 Rust에서 그리고 Pretendard Std를 번들한다

- 상태: 채택 (2026-08-30)

## 결정

- 저장 이미지는 core의 `render::render_png(&Sheet)`가 `image` + `imageproc` + `ab_glyph`로 직접 그린다. WebView 캔버스를 쓰지 않는다.
- 한글 폰트는 Pretendard Std 1.3.9 Regular/Bold(OTF, SIL OFL 1.1)를 `include_bytes!`로 번들한다(약 320KB × 2). 라이선스 사본을 폰트 옆에 둔다.

## 이유

- ADR-0001의 "로직은 Rust" 원칙. 표 레이아웃(행 생략, X표, 높이 계산)은 도메인 규칙이라 테스트되어야 한다.
- 시스템 폰트 의존을 없애 기기·에뮬레이터·CI 어디서나 같은 픽셀이 나온다.
- Std 판은 KS X 1001 한글 2,350자 범위라 이 앱의 모든 문자열을 덮으면서 완전판(5MB+)보다 훨씬 작다.

## 결과

- 사용자가 폰트 크기·색을 고를 수 없다(개인용 앱이라 불필요).
- 렌더 결과는 PNG 디코드 후 크기·픽셀 검사로 단위 테스트한다.
