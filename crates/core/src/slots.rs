//! 후보 슬롯 계산과 라벨 포맷(ADR-0006).
//!
//! 후보는 **엄격히 미래**(`at > now`)인 슬롯만 포함한다. 과거 슬롯 상품은 폐기 대상이다.

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};

use crate::domain::Product;

/// 후보 슬롯을 오늘부터 며칠 뒤까지 보여줄 것인가.
pub const DEFAULT_HORIZON_DAYS: u32 = 14;

/// 한 품목의 후보 슬롯 전체.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotOptions {
    /// 이 후보 목록의 품목.
    pub product: Product,
    /// 날짜별 묶음(오름차순). 후보가 하나도 없는 날짜는 빠진다.
    pub dates: Vec<DateOption>,
}

/// 날짜 휠의 한 칸.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateOption {
    /// 날짜.
    pub date: NaiveDate,
    /// 날짜 휠에 표시할 라벨, 예: `8/30 (일)`.
    pub label: String,
    /// 이미 지난 슬롯만 있는 날짜인가. 편집 중인 항목을 위해 끼워 넣은 날짜에서만 `true`.
    #[serde(default)]
    pub past: bool,
    /// 이 날짜의 후보 시각(오름차순). 항상 하나 이상.
    pub times: Vec<TimeOption>,
}

/// 시각 휠의 한 칸.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeOption {
    /// 이 칸이 가리키는 구체 슬롯.
    pub at: NaiveDateTime,
    /// 24시간제 시.
    pub hour: u32,
    /// 시각 휠에 표시할 라벨, 예: `오후 2시`.
    pub label: String,
}

/// `now`부터 `horizon_days`일 뒤까지의 후보 슬롯.
///
/// 각 날짜의 [`Product::slot_hours`] 중 `at > now`인 것만 담고, 후보가 없는 날짜는 뺀다.
/// `dates`와 `times`는 모두 오름차순이다.
pub fn slot_options(product: Product, now: NaiveDateTime, horizon_days: u32) -> SlotOptions {
    slot_options_with(product, now, horizon_days, None)
}

/// 기본 범위([`DEFAULT_HORIZON_DAYS`])의 후보 슬롯. shell이 쓰는 진입점이다.
pub fn default_slot_options(
    product: Product,
    now: NaiveDateTime,
    include: Option<NaiveDateTime>,
) -> SlotOptions {
    slot_options_with(product, now, DEFAULT_HORIZON_DAYS, include)
}

/// [`slot_options`]에 더해, `include`가 이 품목의 슬롯이면 후보에 없더라도 끼워 넣는다.
///
/// 이미 지난 항목을 편집할 때 그 슬롯을 휠에서 고를 수 있게 하기 위한 것이다.
/// 끼워 넣은 날짜가 `now` 이전이면 [`DateOption::past`]가 `true`다. 품목의 슬롯이 아닌
/// 시각은 무시한다.
pub fn slot_options_with(
    product: Product,
    now: NaiveDateTime,
    horizon_days: u32,
    include: Option<NaiveDateTime>,
) -> SlotOptions {
    let start = now.date();
    let mut dates = Vec::new();

    for offset in 0..=horizon_days {
        let Some(date) = start.checked_add_days(Days::new(u64::from(offset))) else {
            break;
        };

        let times: Vec<TimeOption> = product
            .slot_hours()
            .into_iter()
            .filter_map(|hour| {
                let time = NaiveTime::from_hms_opt(hour, 0, 0)?;
                let at = NaiveDateTime::new(date, time);
                (at > now).then(|| TimeOption {
                    at,
                    hour,
                    label: time_label(hour),
                })
            })
            .collect();

        if !times.is_empty() {
            dates.push(DateOption {
                date,
                label: date_label(date),
                past: false,
                times,
            });
        }
    }

    if let Some(at) = include.filter(|at| product.is_slot(*at)) {
        let option = TimeOption {
            at,
            hour: at.hour(),
            label: time_label(at.hour()),
        };
        match dates.iter_mut().find(|day| day.date == at.date()) {
            Some(day) => {
                if !day.times.iter().any(|time| time.at == at) {
                    day.times.push(option);
                    day.times.sort_by_key(|time| time.at);
                }
            }
            None => {
                let index = dates.partition_point(|day| day.date < at.date());
                dates.insert(
                    index,
                    DateOption {
                        date: at.date(),
                        label: date_label(at.date()),
                        past: at <= now,
                        times: vec![option],
                    },
                );
            }
        }
    }

    SlotOptions { product, dates }
}

/// 날짜 휠 라벨: `8/30 (일)`. 월·일은 앞자리 0 없음.
pub fn date_label(date: NaiveDate) -> String {
    format!("{}/{} ({})", date.month(), date.day(), weekday_label(date))
}

/// 24시간제 시를 (`오전`/`오후`, 12시간제 시)로. 0→(오전, 12), 12→(오후, 12).
pub fn meridiem(hour: u32) -> (&'static str, u32) {
    let period = if hour < 12 { "오전" } else { "오후" };
    let hour12 = match hour % 12 {
        0 => 12,
        other => other,
    };
    (period, hour12)
}

/// 시각 휠 라벨: `오전 2시`, `오후 12시`. 12시간제이며 0시는 `오전 12시`.
pub fn time_label(hour: u32) -> String {
    let (period, hour12) = meridiem(hour);
    format!("{period} {hour12}시")
}

/// 조사표 작성 시각 라벨(화면용): `8/30 (일) 오전 8:02`.
pub fn sheet_label(at: NaiveDateTime) -> String {
    let (period, hour12) = meridiem(at.hour());
    format!(
        "{}/{} ({}) {} {}:{:02}",
        at.month(),
        at.day(),
        weekday_label(at.date()),
        period,
        hour12,
        at.minute()
    )
}

/// 항목 라벨: `8/30 14시`. 월·일은 앞자리 0 없이, 시는 24시간제 두 자리.
pub fn entry_label(at: NaiveDateTime) -> String {
    format!("{}/{} {:02}시", at.month(), at.day(), at.hour())
}

/// 요일 한글 한 글자. 헤더 시각 포맷도 이것을 쓴다.
pub(crate) fn weekday_label(date: NaiveDate) -> &'static str {
    const NAMES: [&str; 7] = ["월", "화", "수", "목", "금", "토", "일"];
    let index = date.weekday().num_days_from_monday() as usize;
    NAMES.get(index).copied().unwrap_or("월")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("test date must be valid")
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("test date must be valid")
    }

    fn first_slot(product: Product, now: NaiveDateTime) -> NaiveDateTime {
        slot_options(product, now, DEFAULT_HORIZON_DAYS)
            .dates
            .first()
            .and_then(|day| day.times.first())
            .map(|time| time.at)
            .expect("at least one candidate within the horizon")
    }

    // ADR-0006 예시 1: 8/1 01:00 → 도시락 첫 후보 8/1 02:00.
    #[test]
    fn adr_example_before_first_slot() {
        assert_eq!(
            first_slot(Product::Lunchbox, at(2026, 8, 1, 1, 0)),
            at(2026, 8, 1, 2, 0)
        );
    }

    // ADR-0006 예시 2: 8/1 03:00 → 첫 후보 8/1 14:00.
    #[test]
    fn adr_example_between_slots() {
        assert_eq!(
            first_slot(Product::Lunchbox, at(2026, 8, 1, 3, 0)),
            at(2026, 8, 1, 14, 0)
        );
    }

    // ADR-0006 예시 3: 8/1 02:00 정각 → 02:00 제외, 첫 후보 14:00.
    #[test]
    fn adr_example_exactly_on_the_slot_excludes_it() {
        let options = slot_options(
            Product::Lunchbox,
            at(2026, 8, 1, 2, 0),
            DEFAULT_HORIZON_DAYS,
        );
        let today = options.dates.first().expect("today still has 14:00");
        assert_eq!(today.date, date(2026, 8, 1));
        assert_eq!(today.times.len(), 1);
        assert_eq!(today.times[0].at, at(2026, 8, 1, 14, 0));
    }

    // ADR-0006 예시 4: 8/1 23:30 → 오늘은 후보가 없어 첫 날짜가 8/2.
    #[test]
    fn adr_example_late_night_drops_today() {
        let options = slot_options(
            Product::Lunchbox,
            at(2026, 8, 1, 23, 30),
            DEFAULT_HORIZON_DAYS,
        );
        let first = options.dates.first().expect("tomorrow is still in range");
        assert_eq!(first.date, date(2026, 8, 2));
        assert_eq!(first.times.len(), 2);
        assert_eq!(first.times[0].at, at(2026, 8, 2, 2, 0));
        assert_eq!(first.times[1].at, at(2026, 8, 2, 14, 0));
    }

    // ADR-0006 예시 5: 샌드위치 8/1 09:59 → 8/1 10:00.
    #[test]
    fn adr_example_sandwich_just_before_ten() {
        assert_eq!(
            first_slot(Product::Sandwich, at(2026, 8, 1, 9, 59)),
            at(2026, 8, 1, 10, 0)
        );
    }

    #[test]
    fn horizon_boundary_is_today_plus_fourteen() {
        let now = at(2026, 8, 1, 1, 0);
        let options = slot_options(Product::Lunchbox, now, DEFAULT_HORIZON_DAYS);

        assert_eq!(options.product, Product::Lunchbox);
        // 오늘 02:00이 아직 남아 있으므로 오늘 포함 15일.
        assert_eq!(options.dates.len(), 15);
        assert_eq!(
            options.dates.last().expect("last date").date,
            date(2026, 8, 15)
        );
        // 오늘이 빠지는 경우에는 14일.
        let late = slot_options(
            Product::Lunchbox,
            at(2026, 8, 1, 23, 30),
            DEFAULT_HORIZON_DAYS,
        );
        assert_eq!(late.dates.len(), 14);
        assert_eq!(
            late.dates.last().expect("last date").date,
            date(2026, 8, 15)
        );
    }

    #[test]
    fn dates_and_times_are_ascending_and_never_empty() {
        let options = slot_options(Product::Burger, at(2026, 8, 1, 12, 0), DEFAULT_HORIZON_DAYS);
        for window in options.dates.windows(2) {
            assert!(window[0].date < window[1].date);
        }
        for day in &options.dates {
            assert!(!day.times.is_empty());
            for window in day.times.windows(2) {
                assert!(window[0].at < window[1].at);
            }
            for time in &day.times {
                assert!(Product::Burger.is_slot(time.at));
                assert_eq!(time.label, time_label(time.hour));
            }
            assert_eq!(day.label, date_label(day.date));
        }
    }

    #[test]
    fn horizon_zero_only_covers_today() {
        let options = slot_options(Product::Lunchbox, at(2026, 8, 1, 1, 0), 0);
        assert_eq!(options.dates.len(), 1);
        assert_eq!(options.dates[0].date, date(2026, 8, 1));
        assert_eq!(options.dates[0].times.len(), 2);
    }

    #[test]
    fn include_inserts_a_past_slot_as_its_own_past_date() {
        let now = at(2026, 8, 30, 8, 2);
        let past = at(2026, 8, 29, 14, 0);
        let options = slot_options_with(Product::Lunchbox, now, DEFAULT_HORIZON_DAYS, Some(past));

        let first = options.dates.first().expect("inserted date comes first");
        assert_eq!(first.date, date(2026, 8, 29));
        assert!(first.past);
        assert_eq!(first.times.len(), 1);
        assert_eq!(first.times[0].at, past);
        assert_eq!(first.times[0].label, "오후 2시");
        assert!(options.dates[1..].iter().all(|day| !day.past));
        assert_eq!(options.dates.len(), 16);
    }

    #[test]
    fn include_merges_into_an_existing_date_without_duplicates() {
        let now = at(2026, 8, 30, 8, 2);
        // 오늘 02:00은 지났지만 오늘 14:00은 후보다 → 오늘 날짜에 02:00을 끼워 넣는다.
        let earlier_today = at(2026, 8, 30, 2, 0);
        let options = slot_options_with(
            Product::Lunchbox,
            now,
            DEFAULT_HORIZON_DAYS,
            Some(earlier_today),
        );
        let today = options.dates.first().expect("today");
        assert_eq!(today.date, date(2026, 8, 30));
        assert!(!today.past, "날짜 자체는 아직 후보를 가진다");
        assert_eq!(
            today.times.iter().map(|t| t.at).collect::<Vec<_>>(),
            [earlier_today, at(2026, 8, 30, 14, 0)]
        );

        // 이미 후보인 슬롯을 끼워 넣어도 중복되지 않는다.
        let dup = slot_options_with(
            Product::Lunchbox,
            now,
            DEFAULT_HORIZON_DAYS,
            Some(at(2026, 8, 30, 14, 0)),
        );
        assert_eq!(
            dup,
            slot_options(Product::Lunchbox, now, DEFAULT_HORIZON_DAYS)
        );
    }

    #[test]
    fn include_ignores_non_slots_and_default_entry_point_matches() {
        let now = at(2026, 8, 30, 8, 2);
        let bogus = at(2026, 8, 29, 13, 0);
        assert_eq!(
            slot_options_with(Product::Lunchbox, now, DEFAULT_HORIZON_DAYS, Some(bogus)),
            slot_options(Product::Lunchbox, now, DEFAULT_HORIZON_DAYS)
        );
        assert_eq!(
            default_slot_options(Product::Burger, now, None),
            slot_options(Product::Burger, now, DEFAULT_HORIZON_DAYS)
        );
    }

    #[test]
    fn sheet_label_format() {
        assert_eq!(sheet_label(at(2026, 8, 30, 8, 2)), "8/30 (일) 오전 8:02");
        assert_eq!(sheet_label(at(2026, 12, 1, 14, 30)), "12/1 (화) 오후 2:30");
        assert_eq!(sheet_label(at(2026, 1, 5, 0, 5)), "1/5 (월) 오전 12:05");
        assert_eq!(meridiem(12), ("오후", 12));
    }

    #[test]
    fn date_label_format() {
        // 계약서 예시는 `8/30 (토)`지만 2026-08-30은 실제로 일요일이다.
        // 고정된 것은 포맷(월/일 앞자리 0 없음 + 한글 한 글자 요일)이지 요일 글자가 아니다.
        assert_eq!(date_label(date(2026, 8, 29)), "8/29 (토)");
        assert_eq!(date_label(date(2026, 8, 30)), "8/30 (일)");
        assert_eq!(date_label(date(2026, 8, 31)), "8/31 (월)");
        assert_eq!(date_label(date(2026, 12, 1)), "12/1 (화)");
    }

    #[test]
    fn time_label_format() {
        assert_eq!(time_label(0), "오전 12시");
        assert_eq!(time_label(2), "오전 2시");
        assert_eq!(time_label(10), "오전 10시");
        assert_eq!(time_label(12), "오후 12시");
        assert_eq!(time_label(14), "오후 2시");
        assert_eq!(time_label(22), "오후 10시");
    }

    #[test]
    fn entry_label_format() {
        assert_eq!(entry_label(at(2026, 8, 30, 14, 0)), "8/30 14시");
        assert_eq!(entry_label(at(2026, 8, 31, 2, 0)), "8/31 02시");
        assert_eq!(entry_label(at(2026, 12, 1, 22, 0)), "12/1 22시");
    }

    #[test]
    fn weekday_labels_cover_the_week() {
        // 2026-08-31은 월요일.
        let labels: Vec<&str> = (0..7)
            .filter_map(|offset| date(2026, 8, 31).checked_add_days(Days::new(offset)))
            .map(weekday_label)
            .collect();
        assert_eq!(labels, ["월", "화", "수", "목", "금", "토", "일"]);
    }
}
