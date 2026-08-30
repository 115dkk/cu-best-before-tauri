//! 조사표를 PNG 한 장으로 그린다(ADR-0002).
//!
//! 시스템 폰트를 쓰지 않고 Pretendard Std Regular/Bold를 번들해, 기기·에뮬레이터·CI
//! 어디서나 같은 픽셀이 나오게 한다. 레이아웃 규칙(행 생략, 빈 구역 X표, 행 높이)은
//! 도메인 규칙이라 단위 테스트 대상이다.

use ab_glyph::{FontRef, PxScale};
use chrono::{Datelike, NaiveDateTime, Timelike};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;

use crate::domain::{Entry, Location, Product, Sheet};
use crate::error::{Error, Result};
use crate::slots;

/// 저장 이미지의 고정 너비. 높이는 내용으로 결정된다.
pub const IMAGE_WIDTH: u32 = 1200;

/// 이미지 바깥 여백.
const MARGIN: i32 = 48;
/// 표 전체 너비 = `IMAGE_WIDTH` − 2 × `MARGIN`.
const TABLE_WIDTH: i32 = IMAGE_WIDTH as i32 - 2 * MARGIN;
/// 열 너비: 구역 / 품목 / 항목.
const COL_LOCATION: i32 = 200;
const COL_PRODUCT: i32 = 260;
const COL_ENTRIES: i32 = 644;
/// 표의 세로선 x 좌표.
const COL_X0: i32 = MARGIN;
const COL_X1: i32 = COL_X0 + COL_LOCATION;
const COL_X2: i32 = COL_X1 + COL_PRODUCT;
const COL_X3: i32 = COL_X2 + COL_ENTRIES;
/// 표 선 굵기.
const LINE: i32 = 3;
/// 헤더 밑줄과 빈 구역 X표의 굵기.
const RULE: i32 = 4;
/// 헤더 높이와 헤더 밑줄 아래 여백.
const HEADER_HEIGHT: i32 = 96;
const HEADER_GAP: i32 = 24;
/// 표가 시작하는 y 좌표.
const TABLE_TOP: i32 = MARGIN + HEADER_HEIGHT + RULE + HEADER_GAP;
/// 셀 안쪽 패딩.
const CELL_PADDING: i32 = 20;
/// 빈 구역 행의 높이.
const EMPTY_ROW_HEIGHT: i32 = 240;
/// 품목 행의 최소 높이.
const MIN_ROW_HEIGHT: i32 = 96;
/// 항목 한 줄의 높이.
const ENTRY_LINE_HEIGHT: i32 = 56;
/// 항목 목록 위아래 패딩 합.
const ENTRY_BLOCK_PADDING: i32 = 2 * CELL_PADDING;

const TITLE: &str = "소비기한 조사표";
const TITLE_SIZE: f32 = 48.0;
const META_SIZE: f32 = 32.0;
const LABEL_SIZE: f32 = 40.0;
const ENTRY_SIZE: f32 = 40.0;

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const INK: Rgba<u8> = Rgba([17, 17, 17, 255]);

const REGULAR_FONT: &[u8] = include_bytes!("../assets/fonts/PretendardStd-Regular.otf");
const BOLD_FONT: &[u8] = include_bytes!("../assets/fonts/PretendardStd-Bold.otf");

/// 번들 폰트 두 벌.
struct Fonts<'a> {
    regular: FontRef<'a>,
    bold: FontRef<'a>,
}

impl Fonts<'static> {
    fn bundled() -> Result<Fonts<'static>> {
        let regular = FontRef::try_from_slice(REGULAR_FONT)
            .map_err(|error| Error::Render(format!("Regular 폰트를 읽을 수 없습니다: {error}")))?;
        let bold = FontRef::try_from_slice(BOLD_FONT)
            .map_err(|error| Error::Render(format!("Bold 폰트를 읽을 수 없습니다: {error}")))?;
        Ok(Fonts { regular, bold })
    }
}

/// 표의 한 행.
enum Row<'a> {
    /// 항목이 하나 이상 있는 품목 행.
    Product {
        product: Product,
        entries: &'a [Entry],
    },
    /// 다섯 품목이 모두 비어 X표로 채우는 행.
    Empty,
}

/// 한 구역이 차지하는 행 묶음.
struct Block<'a> {
    location: Location,
    rows: Vec<(Row<'a>, i32)>,
}

impl Block<'_> {
    fn height(&self) -> i32 {
        self.rows.iter().map(|(_, height)| *height).sum()
    }
}

/// 품목 행 하나의 높이.
fn product_row_height(entry_count: usize) -> i32 {
    let count = i32::try_from(entry_count).unwrap_or(i32::MAX);
    count
        .saturating_mul(ENTRY_LINE_HEIGHT)
        .saturating_add(ENTRY_BLOCK_PADDING)
        .max(MIN_ROW_HEIGHT)
}

/// 구역 순서(매장 → 워크인)대로 행 묶음을 만든다. 항목이 없는 품목 행은 빠진다.
fn layout(sheet: &Sheet) -> Vec<Block<'_>> {
    Location::ALL
        .into_iter()
        .map(|location| {
            let rows: Vec<(Row<'_>, i32)> = Product::ALL
                .into_iter()
                .filter_map(|product| {
                    let entries = sheet.entries(location, product);
                    if entries.is_empty() {
                        return None;
                    }
                    let height = product_row_height(entries.len());
                    Some((Row::Product { product, entries }, height))
                })
                .collect();

            let rows = if rows.is_empty() {
                vec![(Row::Empty, EMPTY_ROW_HEIGHT)]
            } else {
                rows
            };

            Block { location, rows }
        })
        .collect()
}

/// 조사표를 RGBA 이미지로 그린다. 높이는 내용에 따라 결정된다.
pub fn render_rgba(sheet: &Sheet) -> Result<RgbaImage> {
    let fonts = Fonts::bundled()?;
    let blocks = layout(sheet);

    let table_height: i32 = blocks.iter().map(Block::height).sum();
    let total_height = TABLE_TOP + table_height + MARGIN;
    let height = u32::try_from(total_height)
        .map_err(|_| Error::Render(format!("이미지 높이가 잘못되었습니다: {total_height}")))?;

    let mut image = RgbaImage::from_pixel(IMAGE_WIDTH, height, WHITE);
    draw_header(&mut image, sheet.created_at, &fonts);
    draw_table(&mut image, &blocks, table_height, &fonts);
    Ok(image)
}

/// 조사표를 PNG 바이트로 인코딩한다.
pub fn render_png(sheet: &Sheet) -> Result<Vec<u8>> {
    let image = render_rgba(sheet)?;
    let mut buffer = Vec::new();
    PngEncoder::new(&mut buffer)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|error| Error::Render(format!("PNG 인코딩에 실패했습니다: {error}")))?;
    Ok(buffer)
}

/// 헤더 오른쪽의 작성 시각: `2026.08.30 (토) 오전 8:02`.
fn header_timestamp(at: NaiveDateTime) -> String {
    let period = if at.hour() < 12 { "오전" } else { "오후" };
    let hour12 = match at.hour() % 12 {
        0 => 12,
        other => other,
    };
    format!(
        "{}.{:02}.{:02} ({}) {} {}:{:02}",
        at.year(),
        at.month(),
        at.day(),
        slots::weekday_label(at.date()),
        period,
        hour12,
        at.minute()
    )
}

fn draw_header(image: &mut RgbaImage, created_at: NaiveDateTime, fonts: &Fonts<'_>) {
    let title_scale = PxScale::from(TITLE_SIZE);
    let (_, title_height) = text_size(title_scale, &fonts.bold, TITLE);
    draw_text_mut(
        image,
        INK,
        COL_X0,
        MARGIN + centered_offset(HEADER_HEIGHT, title_height),
        title_scale,
        &fonts.bold,
        TITLE,
    );

    let meta = header_timestamp(created_at);
    let meta_scale = PxScale::from(META_SIZE);
    let (meta_width, meta_height) = text_size(meta_scale, &fonts.regular, &meta);
    draw_text_mut(
        image,
        INK,
        COL_X3 - to_i32(meta_width),
        MARGIN + centered_offset(HEADER_HEIGHT, meta_height),
        meta_scale,
        &fonts.regular,
        &meta,
    );

    fill_rect(image, COL_X0, MARGIN + HEADER_HEIGHT, TABLE_WIDTH, RULE);
}

fn draw_table(image: &mut RgbaImage, blocks: &[Block<'_>], table_height: i32, fonts: &Fonts<'_>) {
    // 표 전체를 가로지르는 세로선: 왼쪽 테두리, 구역/품목 경계, 오른쪽 테두리.
    fill_rect(image, COL_X0, TABLE_TOP, LINE, table_height);
    fill_rect(image, COL_X1, TABLE_TOP, LINE, table_height);
    fill_rect(image, COL_X3 - LINE, TABLE_TOP, LINE, table_height);
    // 위·아래 테두리.
    fill_rect(image, COL_X0, TABLE_TOP, TABLE_WIDTH, LINE);
    fill_rect(
        image,
        COL_X0,
        TABLE_TOP + table_height - LINE,
        TABLE_WIDTH,
        LINE,
    );

    let mut block_top = TABLE_TOP;
    for (block_index, block) in blocks.iter().enumerate() {
        if block_index > 0 {
            // 구역 경계선은 구역 열까지 포함해 표 전체를 가로지른다.
            fill_rect(image, COL_X0, block_top, TABLE_WIDTH, LINE);
        }
        draw_location_cell(image, block.location, block_top, block.height(), fonts);

        let mut row_top = block_top;
        for (row_index, (row, height)) in block.rows.iter().enumerate() {
            if row_index > 0 {
                // 같은 구역 안의 행 경계선은 세로 병합된 구역 셀을 건드리지 않는다.
                fill_rect(image, COL_X1, row_top, COL_X3 - COL_X1, LINE);
            }
            match row {
                Row::Product { product, entries } => {
                    fill_rect(image, COL_X2, row_top, LINE, *height);
                    draw_product_row(image, fonts, *product, entries, row_top, *height);
                }
                Row::Empty => draw_empty_row(image, row_top, *height),
            }
            row_top += *height;
        }
        block_top += block.height();
    }
}

/// 세로 병합된 구역 셀에 라벨을 가로·세로 가운데로 그린다.
fn draw_location_cell(
    image: &mut RgbaImage,
    location: Location,
    top: i32,
    height: i32,
    fonts: &Fonts<'_>,
) {
    let scale = PxScale::from(LABEL_SIZE);
    let label = location.label();
    let (width, text_height) = text_size(scale, &fonts.bold, label);
    draw_text_mut(
        image,
        INK,
        COL_X0 + centered_offset(COL_LOCATION, width),
        top + centered_offset(height, text_height),
        scale,
        &fonts.bold,
        label,
    );
}

/// 품목 라벨(세로 가운데)과 항목 줄들을 그린다.
fn draw_product_row(
    image: &mut RgbaImage,
    fonts: &Fonts<'_>,
    product: Product,
    entries: &[Entry],
    top: i32,
    height: i32,
) {
    let label_scale = PxScale::from(LABEL_SIZE);
    let label = product.label();
    let (_, label_height) = text_size(label_scale, &fonts.bold, label);
    draw_text_mut(
        image,
        INK,
        COL_X1 + CELL_PADDING,
        top + centered_offset(height, label_height),
        label_scale,
        &fonts.bold,
        label,
    );

    let entry_scale = PxScale::from(ENTRY_SIZE);
    for (index, entry) in entries.iter().enumerate() {
        let text = format!("{} {}개", slots::entry_label(entry.at), entry.quantity);
        let (_, text_height) = text_size(entry_scale, &fonts.regular, &text);
        let line_top =
            top + CELL_PADDING + i32::try_from(index).unwrap_or(i32::MAX) * ENTRY_LINE_HEIGHT;
        draw_text_mut(
            image,
            INK,
            COL_X2 + CELL_PADDING,
            line_top + centered_offset(ENTRY_LINE_HEIGHT, text_height),
            entry_scale,
            &fonts.regular,
            &text,
        );
    }
}

/// 품목·항목 열을 가로 병합한 칸을 두 대각선 X표로 채운다.
fn draw_empty_row(image: &mut RgbaImage, top: i32, height: i32) {
    let left = COL_X1 as f32;
    let right = COL_X3 as f32;
    let upper = top as f32;
    let lower = (top + height) as f32;
    draw_thick_line(image, (left, upper), (right, lower));
    draw_thick_line(image, (right, upper), (left, lower));
}

/// 선에 수직인 방향으로 평행 이동한 선을 겹쳐 [`RULE`] 굵기를 만든다.
fn draw_thick_line(image: &mut RgbaImage, start: (f32, f32), end: (f32, f32)) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length = dx.hypot(dy);
    if length <= f32::EPSILON {
        return;
    }
    let (normal_x, normal_y) = (-dy / length, dx / length);
    let half = (RULE - 1) as f32 / 2.0;
    for step in 0..RULE {
        let offset = step as f32 - half;
        draw_line_segment_mut(
            image,
            (start.0 + normal_x * offset, start.1 + normal_y * offset),
            (end.0 + normal_x * offset, end.1 + normal_y * offset),
            INK,
        );
    }
}

/// 이미지 밖으로 나가지 않는 채워진 사각형.
fn fill_rect(image: &mut RgbaImage, x: i32, y: i32, width: i32, height: i32) {
    let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
        return;
    };
    if width == 0 || height == 0 {
        return;
    }
    draw_filled_rect_mut(image, Rect::at(x, y).of_size(width, height), INK);
}

/// 높이 `outer` 안에서 높이 `inner`를 가운데 두기 위한 위쪽 여백.
fn centered_offset(outer: i32, inner: u32) -> i32 {
    (outer - to_i32(inner)) / 2
}

fn to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("test date must be valid")
    }

    fn is_dark(image: &RgbaImage, x: u32, y: u32) -> bool {
        image.get_pixel(x, y).0[0] < 128
    }

    /// Bresenham 대각선은 한두 픽셀 흔들리므로 세로로 조금 훑는다.
    fn any_dark_near(image: &RgbaImage, x: u32, y: u32, radius: u32) -> bool {
        (y.saturating_sub(radius)..=y + radius).any(|row| is_dark(image, x, row))
    }

    /// 테스트용 조사표 명세: 어느 구역·품목에 (슬롯, 수량)들을 넣을지.
    type Fixture<'a> = (Location, Product, &'a [(NaiveDateTime, u32)]);

    fn sheet_with(entries: &[Fixture<'_>]) -> Sheet {
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        for (location, product, items) in entries {
            let list = items
                .iter()
                .map(|(when, quantity)| Entry {
                    at: *when,
                    quantity: *quantity,
                })
                .collect();
            sheet
                .sections
                .get_mut(location)
                .expect("location key exists")
                .insert(*product, list);
        }
        sheet
    }

    // (a) 빈 조사표: 폭 1200, 높이 700, 두 구역 모두 X표.
    #[test]
    fn empty_sheet_has_contract_size_and_two_crosses() {
        let sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        let image = render_rgba(&sheet).expect("render");

        assert_eq!(image.width(), 1200);
        assert_eq!(
            image.height(),
            48 + 96 + 4 + 24 + (240 * 2) + 48,
            "총 높이 = 여백 + 헤더 + 밑줄 + 간격 + 빈 구역 두 개 + 여백"
        );
        assert_eq!(image.height(), 700);

        let center_x = (COL_X1 + COL_X3) as u32 / 2;
        for block_index in 0..2 {
            let top = TABLE_TOP + block_index * EMPTY_ROW_HEIGHT;
            let center_y = (top + EMPTY_ROW_HEIGHT / 2) as u32;
            assert!(
                any_dark_near(&image, center_x, center_y, 3),
                "구역 {block_index}: 두 대각선이 만나는 칸 중앙이 어두워야 한다"
            );

            // 대각선에서 벗어난 점(칸 위쪽 가운데)은 흰색이어야 한다.
            let off_diagonal_y = (top + 28) as u32;
            assert!(
                !is_dark(&image, center_x, off_diagonal_y),
                "구역 {block_index}: 대각선 밖은 흰색이어야 한다"
            );
        }
    }

    // (b) 항목 3개짜리 품목 하나 → 행 높이 208이 총 높이에 반영된다.
    #[test]
    fn three_entry_row_is_two_hundred_eight_tall() {
        assert_eq!(product_row_height(3), 208);
        assert_eq!(product_row_height(1), 96);
        assert_eq!(product_row_height(2), 152);
        assert_eq!(product_row_height(0), 96);

        let sheet = sheet_with(&[(
            Location::Store,
            Product::Onigiri,
            &[
                (at(2026, 8, 30, 14, 0), 12),
                (at(2026, 8, 31, 2, 0), 6),
                (at(2026, 8, 31, 14, 0), 3),
            ],
        )]);
        let image = render_rgba(&sheet).expect("render");

        // 매장 = 208(품목 행 하나), 워크인 = 240(빈 구역).
        assert_eq!(image.width(), 1200);
        assert_eq!(image.height(), (TABLE_TOP + 208 + 240 + MARGIN) as u32);
        assert_eq!(image.height(), 668);
    }

    // (c) render_png는 디코드되고 크기가 render_rgba와 같다.
    #[test]
    fn png_decodes_to_the_same_size() {
        let sheet = sheet_with(&[
            (
                Location::Store,
                Product::Onigiri,
                &[(at(2026, 8, 30, 14, 0), 12)],
            ),
            (
                Location::WalkIn,
                Product::Sandwich,
                &[(at(2026, 8, 30, 22, 0), 4)],
            ),
        ]);
        let expected = render_rgba(&sheet).expect("render");
        let bytes = render_png(&sheet).expect("encode");

        let decoded = image::load_from_memory(&bytes).expect("decode png");
        assert_eq!(decoded.width(), expected.width());
        assert_eq!(decoded.height(), expected.height());
        assert_eq!(decoded.to_rgba8(), expected);
    }

    #[test]
    fn columns_sum_to_the_table_width() {
        assert_eq!(COL_LOCATION + COL_PRODUCT + COL_ENTRIES, TABLE_WIDTH);
        assert_eq!(TABLE_WIDTH, 1104);
        assert_eq!(COL_X3 - COL_X0, TABLE_WIDTH);
        assert_eq!(TABLE_TOP, 172);
    }

    #[test]
    fn rows_without_entries_are_omitted() {
        let sheet = sheet_with(&[
            (
                Location::Store,
                Product::Onigiri,
                &[(at(2026, 8, 30, 14, 0), 12)],
            ),
            (
                Location::Store,
                Product::Burger,
                &[(at(2026, 8, 30, 22, 0), 2)],
            ),
        ]);
        let blocks = layout(&sheet);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].rows.len(), 2, "김밥·도시락·샌드위치 행은 빠진다");
        assert!(matches!(
            blocks[0].rows[0].0,
            Row::Product {
                product: Product::Onigiri,
                ..
            }
        ));
        assert!(matches!(
            blocks[0].rows[1].0,
            Row::Product {
                product: Product::Burger,
                ..
            }
        ));
        assert_eq!(blocks[0].height(), 96 * 2);
        assert!(matches!(blocks[1].rows[0].0, Row::Empty));
        assert_eq!(blocks[1].height(), EMPTY_ROW_HEIGHT);
    }

    #[test]
    fn header_timestamp_format() {
        // 계약서 예시는 `(토)`지만 2026-08-30은 실제로 일요일이다. 고정된 것은 포맷이다.
        assert_eq!(
            header_timestamp(at(2026, 8, 29, 8, 2)),
            "2026.08.29 (토) 오전 8:02"
        );
        assert_eq!(
            header_timestamp(at(2026, 8, 30, 8, 2)),
            "2026.08.30 (일) 오전 8:02"
        );
        assert_eq!(
            header_timestamp(at(2026, 12, 1, 14, 30)),
            "2026.12.01 (화) 오후 2:30"
        );
        assert_eq!(
            header_timestamp(at(2026, 1, 5, 0, 5)),
            "2026.01.05 (월) 오전 12:05"
        );
        assert_eq!(
            header_timestamp(at(2026, 1, 5, 12, 0)),
            "2026.01.05 (월) 오후 12:00"
        );
    }

    #[test]
    fn table_borders_are_drawn() {
        let sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        let image = render_rgba(&sheet).expect("render");

        // 헤더 밑줄.
        assert!(is_dark(&image, 600, (MARGIN + HEADER_HEIGHT) as u32));
        // 표 위·아래 테두리와 좌우 테두리.
        assert!(is_dark(&image, 600, TABLE_TOP as u32));
        assert!(is_dark(
            &image,
            600,
            (image.height() as i32 - MARGIN - 1) as u32
        ));
        assert!(is_dark(&image, COL_X0 as u32, (TABLE_TOP + 100) as u32));
        assert!(is_dark(
            &image,
            (COL_X3 - 1) as u32,
            (TABLE_TOP + 100) as u32
        ));
        // 구역/품목 경계선.
        assert!(is_dark(&image, COL_X1 as u32, (TABLE_TOP + 100) as u32));
        // 여백은 흰색.
        assert!(!is_dark(&image, 4, 4));
    }

    #[test]
    fn empty_section_merges_the_last_two_columns() {
        let sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        let image = render_rgba(&sheet).expect("render");

        // 빈 구역 행에는 품목/항목 경계선이 없다. 대각선을 피해 위쪽에서 확인한다.
        let probe_y = (TABLE_TOP + 20) as u32;
        assert!(!is_dark(&image, COL_X2 as u32, probe_y));
        assert!(!is_dark(&image, (COL_X2 + 1) as u32, probe_y));
    }

    #[test]
    fn product_section_keeps_the_product_entry_divider() {
        let sheet = sheet_with(&[(
            Location::Store,
            Product::Onigiri,
            &[(at(2026, 8, 30, 14, 0), 12)],
        )]);
        let image = render_rgba(&sheet).expect("render");

        assert!(is_dark(&image, COL_X2 as u32, (TABLE_TOP + 48) as u32));
    }
}
