//! 화면이 그대로 찍는 표시용 뷰(ADR-0008).
//!
//! 저장 형식([`Sheet`])은 시각만 담고, 화면에 보일 문자열은 전부 여기서 만든다. 그래서
//! 프론트엔드는 날짜·요일·오전/오후 계산을 하지 않는다(ADR-0001). 항목의 기한 슬롯이
//! 지났는지도 core가 `now`와 비교해 `past`로 만든다. 뷰 JSON은 저장 형식의 상위 집합이라
//! `save_sheet`에 그대로 되돌려 보내도 된다 — [`Sheet`]는 모르는 필드를 무시한다.

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::{Entry, Location, Product, Sheet};
use crate::slots;

/// 항목 + 화면 라벨(`8/30 14시`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryView {
    /// 기한 슬롯.
    pub at: NaiveDateTime,
    /// 수량.
    pub quantity: u32,
    /// 화면과 PNG가 같은 문자열을 쓰도록 core가 만든 라벨.
    pub label: String,
    /// 기한 슬롯이 이미 지났는가(`at <= now`). 후보 판정(slots.rs)과 같은 기준이다.
    #[serde(default)]
    pub past: bool,
}

impl EntryView {
    /// `now` 기준으로 항목 뷰를 만든다.
    pub fn new(entry: &Entry, now: NaiveDateTime) -> EntryView {
        EntryView {
            at: entry.at,
            quantity: entry.quantity,
            label: slots::entry_label(entry.at),
            past: entry.at <= now,
        }
    }
}

/// 한 구역의 품목별 항목 뷰. 다섯 품목 키가 항상 모두 있다.
pub type SectionView = BTreeMap<Product, Vec<EntryView>>;

/// 조사표 뷰. 구역 2개 × 품목 5개 키가 항상 모두 있다(저장 파일에 키가 빠져 있어도).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetView {
    /// 조사표 id.
    pub id: String,
    /// 작성 시각.
    pub created_at: NaiveDateTime,
    /// 작성 시각 라벨, 예: `8/30 (일) 오전 8:02`.
    pub created_label: String,
    /// 마지막 저장 시각.
    pub updated_at: NaiveDateTime,
    /// 구역별 항목 뷰.
    pub sections: BTreeMap<Location, SectionView>,
}

impl SheetView {
    /// `now` 기준으로 조사표 뷰를 만든다(빠진 키를 빈 Vec으로 채운다).
    pub fn new(sheet: &Sheet, now: NaiveDateTime) -> SheetView {
        let sections = Location::ALL
            .into_iter()
            .map(|location| {
                let section: SectionView = Product::ALL
                    .into_iter()
                    .map(|product| {
                        let entries = sheet
                            .entries(location, product)
                            .iter()
                            .map(|entry| EntryView::new(entry, now))
                            .collect();
                        (product, entries)
                    })
                    .collect();
                (location, section)
            })
            .collect();

        SheetView {
            id: sheet.id.clone(),
            created_at: sheet.created_at,
            created_label: slots::sheet_label(sheet.created_at),
            updated_at: sheet.updated_at,
            sections,
        }
    }
}

/// 품목·구역 하나의 키와 화면 라벨.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogItem<K> {
    /// JSON 키(`onigiri`, `store` …). 조사표 JSON의 키와 같다.
    pub key: K,
    /// 화면 라벨(`삼각김밥`, `매장` …).
    pub label: String,
}

/// 화면이 목록·라벨을 손으로 복사하지 않도록 core가 내려주는 품목·구역 목록(표시 순서).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    /// 품목 다섯, `Product::ALL` 순서.
    pub products: Vec<CatalogItem<Product>>,
    /// 구역 둘, `Location::ALL` 순서.
    pub locations: Vec<CatalogItem<Location>>,
}

impl Catalog {
    /// 현재 도메인의 품목·구역 목록.
    pub fn current() -> Catalog {
        Catalog {
            products: Product::ALL
                .into_iter()
                .map(|product| CatalogItem {
                    key: product,
                    label: product.label().to_string(),
                })
                .collect(),
            locations: Location::ALL
                .into_iter()
                .map(|location| CatalogItem {
                    key: location,
                    label: location.label().to_string(),
                })
                .collect(),
        }
    }
}

impl Default for Catalog {
    fn default() -> Catalog {
        Catalog::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde_json::json;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("test date must be valid")
    }

    #[test]
    fn view_fills_missing_keys_and_labels_entries() {
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        sheet.sections.remove(&Location::WalkIn);
        let store = sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section");
        store.remove(&Product::Gimbap);
        store.insert(
            Product::Onigiri,
            vec![Entry {
                at: at(2026, 8, 31, 2, 0),
                quantity: 6,
            }],
        );

        let view = SheetView::new(&sheet, at(2026, 8, 30, 8, 2));
        assert_eq!(view.created_label, "8/30 (일) 오전 8:02");
        assert_eq!(view.sections.len(), 2);
        for location in Location::ALL {
            assert_eq!(view.sections[&location].len(), 5);
        }
        let onigiri = &view.sections[&Location::Store][&Product::Onigiri];
        assert_eq!(onigiri.len(), 1);
        assert_eq!(onigiri[0].label, "8/31 02시");
        assert_eq!(onigiri[0].quantity, 6);
        assert!(view.sections[&Location::WalkIn][&Product::Burger].is_empty());
    }

    #[test]
    fn entry_past_is_true_at_or_before_now() {
        let now = at(2026, 8, 30, 14, 0);
        let before = Entry {
            at: at(2026, 8, 30, 13, 0),
            quantity: 1,
        };
        let same = Entry {
            at: now,
            quantity: 2,
        };
        let after = Entry {
            at: at(2026, 8, 30, 15, 0),
            quantity: 3,
        };

        assert!(EntryView::new(&before, now).past);
        assert!(EntryView::new(&same, now).past);
        assert!(!EntryView::new(&after, now).past);
    }

    #[test]
    fn view_json_round_trips_into_a_sheet() {
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Sandwich,
                vec![Entry {
                    at: at(2026, 8, 30, 22, 0),
                    quantity: 4,
                }],
            );

        let value = serde_json::to_value(SheetView::new(&sheet, at(2026, 8, 30, 8, 2)))
            .expect("serialize view");
        assert_eq!(value["created_label"], json!("8/30 (일) 오전 8:02"));
        assert_eq!(
            value["sections"]["store"]["sandwich"][0]["label"],
            json!("8/30 22시")
        );

        // 프론트엔드가 뷰를 그대로 되돌려 보내는 경로: 추가 필드는 무시된다.
        let back: Sheet = serde_json::from_value(value).expect("deserialize as sheet");
        assert_eq!(back, sheet);
    }

    #[test]
    fn catalog_lists_products_and_locations_in_display_order() {
        let catalog = Catalog::current();
        assert_eq!(
            catalog.products.iter().map(|p| p.key).collect::<Vec<_>>(),
            Product::ALL
        );
        assert_eq!(
            catalog
                .products
                .iter()
                .map(|p| p.label.as_str())
                .collect::<Vec<_>>(),
            ["삼각김밥", "김밥", "도시락", "샌드위치", "햄버거"]
        );
        assert_eq!(
            catalog
                .locations
                .iter()
                .map(|l| l.label.as_str())
                .collect::<Vec<_>>(),
            ["매장", "워크인"]
        );
        let value = serde_json::to_value(&catalog).expect("serialize");
        assert_eq!(value["products"][0]["key"], json!("onigiri"));
        assert_eq!(value["locations"][1]["key"], json!("walk_in"));
        assert_eq!(Catalog::default(), catalog);
    }
}
