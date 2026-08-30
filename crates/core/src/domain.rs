//! 도메인 타입: 품목(Product), 구역(Location), 항목(Entry), 구역표(Section), 조사표(Sheet).
//!
//! 이 모듈은 시각을 만들지 않는다. `now`는 항상 호출자가 넘긴다(ADR-0001).

use std::collections::BTreeMap;
use std::fmt;

use chrono::{NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// 조사 대상 상품 종류. 선언 순서가 곧 화면·이미지 표시 순서이며 `Ord`가 그것을 따른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Product {
    /// 삼각김밥
    Onigiri,
    /// 김밥
    Gimbap,
    /// 도시락
    Lunchbox,
    /// 샌드위치
    Sandwich,
    /// 햄버거
    Burger,
}

impl Product {
    /// 모든 품목을 표시 순서대로.
    pub const ALL: [Product; 5] = [
        Product::Onigiri,
        Product::Gimbap,
        Product::Lunchbox,
        Product::Sandwich,
        Product::Burger,
    ];

    /// 화면·이미지에 쓰는 한글 이름.
    pub fn label(self) -> &'static str {
        match self {
            Product::Onigiri => "삼각김밥",
            Product::Gimbap => "김밥",
            Product::Lunchbox => "도시락",
            Product::Sandwich => "샌드위치",
            Product::Burger => "햄버거",
        }
    }

    /// 이 품목의 소비기한이 만료되는 하루 두 시각(오름차순).
    pub fn slot_hours(self) -> [u32; 2] {
        match self {
            Product::Onigiri | Product::Gimbap | Product::Lunchbox => [2, 14],
            Product::Sandwich | Product::Burger => [10, 22],
        }
    }

    /// `at`이 이 품목의 기한 슬롯인가(분·초·나노초가 0이고 시가 `slot_hours`에 있는가).
    pub fn is_slot(self, at: NaiveDateTime) -> bool {
        at.minute() == 0
            && at.second() == 0
            && at.nanosecond() == 0
            && self.slot_hours().contains(&at.hour())
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// 상품이 놓인 물리적 공간. 정확히 둘.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    /// 매장 — 매대에 진열된 것.
    Store,
    /// 워크인 — 냉장 창고에 들어 있는 것.
    WalkIn,
}

impl Location {
    /// 모든 구역을 표시 순서대로.
    pub const ALL: [Location; 2] = [Location::Store, Location::WalkIn];

    /// 화면·이미지에 쓰는 한글 이름.
    pub fn label(self) -> &'static str {
        match self {
            Location::Store => "매장",
            Location::WalkIn => "워크인",
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// "이 슬롯에 만료되는 상품이 N개 있다"는 기록.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// 기한 슬롯(기기 로컬 시각).
    pub at: NaiveDateTime,
    /// 수량. 정규화 후에는 항상 1 이상.
    pub quantity: u32,
}

/// 한 구역 안의 품목별 항목 목록.
///
/// 품목 키 5개가 항상 모두 존재한다(빈 `Vec` 허용). [`Sheet::normalized`]가 보장한다.
pub type Section = BTreeMap<Product, Vec<Entry>>;

/// 한 번의 소비기한 조사 결과 전체.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    /// `created_at`을 `%Y%m%d-%H%M%S`로 포맷한 파일명 안전 식별자.
    pub id: String,
    /// 작성 시각.
    pub created_at: NaiveDateTime,
    /// 마지막 저장 시각.
    pub updated_at: NaiveDateTime,
    /// 구역별 구역표. 키 2개가 항상 존재한다.
    pub sections: BTreeMap<Location, Section>,
}

impl Sheet {
    /// 모든 구역·품목 키를 빈 목록으로 채운 새 조사표.
    pub fn new(now: NaiveDateTime) -> Sheet {
        Sheet {
            id: Sheet::sheet_id(now),
            created_at: now,
            updated_at: now,
            sections: empty_sections(),
        }
    }

    /// 시각에서 조사표 id를 만든다.
    pub fn sheet_id(at: NaiveDateTime) -> String {
        at.format("%Y%m%d-%H%M%S").to_string()
    }

    /// 파일명으로 쓸 수 있는 id인가(비어있지 않고 `[0-9A-Za-z_-]`만).
    pub fn is_valid_id(id: &str) -> bool {
        !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    /// 어느 구역·품목의 항목 목록. 키가 없으면 빈 슬라이스.
    pub fn entries(&self, location: Location, product: Product) -> &[Entry] {
        self.sections
            .get(&location)
            .and_then(|section| section.get(&product))
            .map_or(&[], Vec::as_slice)
    }

    /// 이 구역의 다섯 품목이 모두 항목 0개인가.
    pub fn is_section_empty(&self, location: Location) -> bool {
        self.sections
            .get(&location)
            .is_none_or(|section| section.values().all(Vec::is_empty))
    }

    /// 모든 구역·품목의 항목 개수.
    pub fn entry_count(&self) -> u32 {
        let count: usize = self
            .sections
            .values()
            .flat_map(BTreeMap::values)
            .map(Vec::len)
            .sum();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// 모든 항목의 수량 합(포화 덧셈).
    pub fn total_quantity(&self) -> u32 {
        self.sections
            .values()
            .flat_map(BTreeMap::values)
            .flatten()
            .fold(0u32, |sum, entry| sum.saturating_add(entry.quantity))
    }

    /// 저장 전 정규화.
    ///
    /// 빠진 키를 채우고, 각 항목을 검증하고([`Error::InvalidQuantity`],
    /// [`Error::InvalidSlot`]), 슬롯 오름차순으로 정렬한 뒤 같은 슬롯끼리 수량을 합치고,
    /// id를 검증하고([`Error::InvalidId`]), `updated_at`을 `now`로 바꾼다.
    /// `created_at`은 유지한다.
    pub fn normalized(self, now: NaiveDateTime) -> Result<Sheet> {
        let Sheet {
            id,
            created_at,
            updated_at: _,
            sections,
        } = self;

        let mut normalized = BTreeMap::new();
        for location in Location::ALL {
            let mut section = Section::new();
            for product in Product::ALL {
                let mut entries = sections
                    .get(&location)
                    .and_then(|section| section.get(&product))
                    .cloned()
                    .unwrap_or_default();

                for entry in &entries {
                    if entry.quantity < 1 {
                        return Err(Error::InvalidQuantity);
                    }
                    if !product.is_slot(entry.at) {
                        return Err(Error::InvalidSlot {
                            product,
                            at: entry.at,
                        });
                    }
                }

                entries.sort_by_key(|entry| entry.at);
                section.insert(product, merge_same_slot(entries));
            }
            normalized.insert(location, section);
        }

        if !Sheet::is_valid_id(&id) {
            return Err(Error::InvalidId(id));
        }

        Ok(Sheet {
            id,
            created_at,
            updated_at: now,
            sections: normalized,
        })
    }
}

/// 구역·품목 키를 모두 갖춘 빈 구역표 묶음.
fn empty_sections() -> BTreeMap<Location, Section> {
    Location::ALL
        .into_iter()
        .map(|location| {
            let section = Product::ALL
                .into_iter()
                .map(|product| (product, Vec::new()))
                .collect();
            (location, section)
        })
        .collect()
}

/// 슬롯 오름차순으로 정렬된 항목에서 같은 슬롯을 하나로 합친다(수량 포화 합).
fn merge_same_slot(entries: Vec<Entry>) -> Vec<Entry> {
    let mut merged: Vec<Entry> = Vec::with_capacity(entries.len());
    for entry in entries {
        match merged.last_mut() {
            Some(last) if last.at == entry.at => {
                last.quantity = last.quantity.saturating_add(entry.quantity);
            }
            _ => merged.push(entry),
        }
    }
    merged
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
    fn product_labels_and_slot_hours() {
        assert_eq!(Product::Onigiri.label(), "삼각김밥");
        assert_eq!(Product::Gimbap.label(), "김밥");
        assert_eq!(Product::Lunchbox.label(), "도시락");
        assert_eq!(Product::Sandwich.label(), "샌드위치");
        assert_eq!(Product::Burger.label(), "햄버거");
        assert_eq!(Product::Onigiri.slot_hours(), [2, 14]);
        assert_eq!(Product::Gimbap.slot_hours(), [2, 14]);
        assert_eq!(Product::Lunchbox.slot_hours(), [2, 14]);
        assert_eq!(Product::Sandwich.slot_hours(), [10, 22]);
        assert_eq!(Product::Burger.slot_hours(), [10, 22]);
        assert_eq!(Product::Lunchbox.to_string(), "도시락");
        assert_eq!(Location::Store.to_string(), "매장");
        assert_eq!(Location::WalkIn.to_string(), "워크인");
    }

    #[test]
    fn product_ord_matches_display_order() {
        let mut sorted = Product::ALL;
        sorted.sort();
        assert_eq!(sorted, Product::ALL);
        assert!(Product::Onigiri < Product::Burger);
        assert!(Location::Store < Location::WalkIn);
    }

    #[test]
    fn is_slot_requires_exact_hour() {
        assert!(Product::Lunchbox.is_slot(at(2026, 8, 30, 14, 0)));
        assert!(Product::Lunchbox.is_slot(at(2026, 8, 30, 2, 0)));
        assert!(!Product::Lunchbox.is_slot(at(2026, 8, 30, 14, 1)));
        assert!(!Product::Lunchbox.is_slot(at(2026, 8, 30, 10, 0)));
        assert!(Product::Sandwich.is_slot(at(2026, 8, 30, 10, 0)));
        assert!(Product::Sandwich.is_slot(at(2026, 8, 30, 22, 0)));
        assert!(!Product::Sandwich.is_slot(at(2026, 8, 30, 2, 0)));

        let with_seconds = NaiveDate::from_ymd_opt(2026, 8, 30)
            .and_then(|date| date.and_hms_opt(14, 0, 1))
            .expect("test date must be valid");
        assert!(!Product::Lunchbox.is_slot(with_seconds));

        let with_nanos = NaiveDate::from_ymd_opt(2026, 8, 30)
            .and_then(|date| date.and_hms_nano_opt(14, 0, 0, 1))
            .expect("test date must be valid");
        assert!(!Product::Lunchbox.is_slot(with_nanos));
    }

    #[test]
    fn new_sheet_has_every_key() {
        let now = at(2026, 8, 30, 8, 2);
        let sheet = Sheet::new(now);

        assert_eq!(sheet.id, "20260830-080200");
        assert_eq!(sheet.created_at, now);
        assert_eq!(sheet.updated_at, now);
        assert_eq!(sheet.sections.len(), 2);
        for location in Location::ALL {
            let section = sheet
                .sections
                .get(&location)
                .expect("every location key must exist");
            assert_eq!(section.len(), 5);
            for product in Product::ALL {
                assert_eq!(sheet.entries(location, product), &[]);
            }
            assert!(sheet.is_section_empty(location));
        }
        assert_eq!(sheet.entry_count(), 0);
        assert_eq!(sheet.total_quantity(), 0);
    }

    #[test]
    fn sheet_id_and_validity() {
        assert_eq!(
            Sheet::sheet_id(at(2026, 8, 30, 8, 2)),
            "20260830-080200".to_string()
        );
        assert!(Sheet::is_valid_id("20260830-080215"));
        assert!(Sheet::is_valid_id("a_B-9"));
        assert!(!Sheet::is_valid_id(""));
        assert!(!Sheet::is_valid_id("../escape"));
        assert!(!Sheet::is_valid_id("has space"));
        assert!(!Sheet::is_valid_id("한글"));
    }

    #[test]
    fn normalized_fills_keys_sorts_and_merges() {
        let now = at(2026, 8, 30, 8, 10);
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        // 일부러 키 하나를 지워 정규화가 되살리는지 본다.
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .remove(&Product::Gimbap);
        sheet.sections.remove(&Location::WalkIn);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Onigiri,
                vec![
                    Entry {
                        at: at(2026, 8, 31, 2, 0),
                        quantity: 6,
                    },
                    Entry {
                        at: at(2026, 8, 30, 14, 0),
                        quantity: 5,
                    },
                    Entry {
                        at: at(2026, 8, 30, 14, 0),
                        quantity: 7,
                    },
                ],
            );

        let normalized = sheet.normalized(now).expect("valid sheet");

        assert_eq!(normalized.created_at, at(2026, 8, 30, 8, 2));
        assert_eq!(normalized.updated_at, now);
        assert_eq!(normalized.sections.len(), 2);
        for location in Location::ALL {
            assert_eq!(
                normalized
                    .sections
                    .get(&location)
                    .expect("restored location")
                    .len(),
                5
            );
        }
        assert_eq!(
            normalized.entries(Location::Store, Product::Onigiri),
            &[
                Entry {
                    at: at(2026, 8, 30, 14, 0),
                    quantity: 12,
                },
                Entry {
                    at: at(2026, 8, 31, 2, 0),
                    quantity: 6,
                },
            ]
        );
        assert_eq!(normalized.entry_count(), 2);
        assert_eq!(normalized.total_quantity(), 18);
        assert!(!normalized.is_section_empty(Location::Store));
        assert!(normalized.is_section_empty(Location::WalkIn));
    }

    #[test]
    fn normalized_rejects_bad_slot() {
        let now = at(2026, 8, 30, 8, 10);
        let mut sheet = Sheet::new(now);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Onigiri,
                vec![Entry {
                    at: at(2026, 8, 30, 10, 0),
                    quantity: 1,
                }],
            );

        match sheet.normalized(now) {
            Err(Error::InvalidSlot { product, at: when }) => {
                assert_eq!(product, Product::Onigiri);
                assert_eq!(when, at(2026, 8, 30, 10, 0));
            }
            other => panic!("expected InvalidSlot, got {other:?}"),
        }
    }

    #[test]
    fn normalized_rejects_zero_quantity() {
        let now = at(2026, 8, 30, 8, 10);
        let mut sheet = Sheet::new(now);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Onigiri,
                vec![Entry {
                    at: at(2026, 8, 30, 14, 0),
                    quantity: 0,
                }],
            );

        assert!(matches!(sheet.normalized(now), Err(Error::InvalidQuantity)));
    }

    #[test]
    fn normalized_rejects_bad_id() {
        let now = at(2026, 8, 30, 8, 10);
        let mut sheet = Sheet::new(now);
        sheet.id = "../evil".to_string();

        match sheet.normalized(now) {
            Err(Error::InvalidId(id)) => assert_eq!(id, "../evil"),
            other => panic!("expected InvalidId, got {other:?}"),
        }
    }

    #[test]
    fn serde_round_trip_matches_contract_shape() {
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        sheet.updated_at = at(2026, 8, 30, 8, 10);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Onigiri,
                vec![Entry {
                    at: at(2026, 8, 30, 14, 0),
                    quantity: 12,
                }],
            );

        let expected = json!({
            "id": "20260830-080215",
            "created_at": "2026-08-30T08:02:15",
            "updated_at": "2026-08-30T08:10:00",
            "sections": {
                "store": {
                    "onigiri": [{ "at": "2026-08-30T14:00:00", "quantity": 12 }],
                    "gimbap": [],
                    "lunchbox": [],
                    "sandwich": [],
                    "burger": []
                },
                "walk_in": {
                    "onigiri": [],
                    "gimbap": [],
                    "lunchbox": [],
                    "sandwich": [],
                    "burger": []
                }
            }
        });

        // 계약서의 JSON 예와 같은 시각을 쓰도록 초까지 맞춘 조사표를 따로 만든다.
        let contract_sheet = Sheet {
            id: "20260830-080215".to_string(),
            created_at: NaiveDate::from_ymd_opt(2026, 8, 30)
                .and_then(|date| date.and_hms_opt(8, 2, 15))
                .expect("valid"),
            updated_at: at(2026, 8, 30, 8, 10),
            sections: sheet.sections.clone(),
        };

        assert_eq!(
            serde_json::to_value(&contract_sheet).expect("serialize"),
            expected
        );
        let decoded: Sheet = serde_json::from_value(expected).expect("deserialize");
        assert_eq!(decoded, contract_sheet);
    }

    #[test]
    fn counts_saturate_rather_than_overflow() {
        let now = at(2026, 8, 30, 8, 2);
        let mut sheet = Sheet::new(now);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Onigiri,
                vec![
                    Entry {
                        at: at(2026, 8, 30, 14, 0),
                        quantity: u32::MAX,
                    },
                    Entry {
                        at: at(2026, 8, 31, 14, 0),
                        quantity: 5,
                    },
                ],
            );
        assert_eq!(sheet.total_quantity(), u32::MAX);

        // 같은 슬롯 병합도 포화 덧셈이다.
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(
                Product::Gimbap,
                vec![
                    Entry {
                        at: at(2026, 8, 30, 14, 0),
                        quantity: u32::MAX,
                    },
                    Entry {
                        at: at(2026, 8, 30, 14, 0),
                        quantity: 3,
                    },
                ],
            );
        let normalized = sheet.normalized(now).expect("valid");
        assert_eq!(
            normalized.entries(Location::Store, Product::Gimbap),
            &[Entry {
                at: at(2026, 8, 30, 14, 0),
                quantity: u32::MAX,
            }]
        );
    }
}
