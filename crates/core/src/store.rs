//! 조사표 파일 저장소: 디렉터리 하나에 `<id>.json` 하나씩(ADR-0004).

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::domain::Sheet;
use crate::error::{Error, Result};

/// 이 일수보다 오래된 조사표는 앱 시작 시 지운다.
pub const RETENTION_DAYS: i64 = 30;

/// 목록 화면이 쓰는 요약. 조사표 본문 없이 한 줄을 그리기 위한 최소 정보.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSummary {
    /// 조사표 id.
    pub id: String,
    /// 작성 시각.
    pub created_at: NaiveDateTime,
    /// 마지막 저장 시각.
    pub updated_at: NaiveDateTime,
    /// 모든 구역·품목의 항목 개수.
    pub entry_count: u32,
    /// 모든 항목의 수량 합.
    pub total_quantity: u32,
}

impl SheetSummary {
    /// 조사표에서 요약을 뽑는다.
    fn of(sheet: &Sheet) -> SheetSummary {
        SheetSummary {
            id: sheet.id.clone(),
            created_at: sheet.created_at,
            updated_at: sheet.updated_at,
            entry_count: sheet.entry_count(),
            total_quantity: sheet.total_quantity(),
        }
    }
}

/// 디렉터리 하나를 조사표 저장소로 다룬다.
#[derive(Debug, Clone)]
pub struct SheetStore {
    dir: PathBuf,
    write_lock: Arc<Mutex<()>>,
}

impl SheetStore {
    /// 저장 디렉터리를 만들고(이미 있으면 그대로) 저장소를 연다.
    pub fn open(dir: impl Into<PathBuf>) -> Result<SheetStore> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(SheetStore {
            dir,
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    /// 저장 디렉터리.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// id에 해당하는 파일 경로. 파일명으로 안전하지 않은 id는 거부한다.
    pub fn path_of(&self, id: &str) -> Result<PathBuf> {
        if !Sheet::is_valid_id(id) {
            return Err(Error::InvalidId(id.to_string()));
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    /// 임시 파일에 쓴 뒤 rename해 원자적으로 저장한다(같은 이름은 덮어쓴다).
    pub fn save(&self, sheet: &Sheet) -> Result<()> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| Error::Io(std::io::Error::other("조사표 저장 잠금이 손상되었습니다")))?;
        let path = self.path_of(&sheet.id)?;
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(sheet)?;
        fs::write(&tmp, &bytes)?;

        // Unix rename replaces an existing destination atomically. Windows does not, so remove
        // the old file before the second rename when the first attempt reports that conflict.
        match fs::rename(&tmp, &path) {
            Ok(()) => Ok(()),
            Err(error)
                if path.exists()
                    && matches!(
                        error.kind(),
                        ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
                    ) =>
            {
                fs::remove_file(&path)?;
                fs::rename(&tmp, &path)?;
                Ok(())
            }
            Err(error) => Err(Error::Io(error)),
        }
    }

    /// 조사표를 읽는다. 파일이 없으면 [`Error::NotFound`].
    pub fn load(&self, id: &str) -> Result<Sheet> {
        let path = self.path_of(id)?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(Error::NotFound(id.to_string()));
            }
            Err(error) => return Err(Error::Io(error)),
        };
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// 모든 조사표 요약을 작성 시각 내림차순으로.
    ///
    /// 읽거나 파싱할 수 없는 파일은 건너뛴다. 파일 하나가 깨졌다고 목록 전체가
    /// 실패하면 사용자가 앱을 쓸 수 없게 된다.
    pub fn list(&self) -> Result<Vec<SheetSummary>> {
        let mut summaries: Vec<SheetSummary> =
            self.read_sheets()?.iter().map(SheetSummary::of).collect();
        summaries.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        Ok(summaries)
    }

    /// 조사표를 지운다. 없으면 그냥 성공(멱등).
    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.path_of(id)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Error::Io(error)),
        }
    }

    /// `now - updated_at > max_age`인 조사표를 지우고 지운 id를 돌려준다.
    pub fn purge_older_than(&self, now: NaiveDateTime, max_age: Duration) -> Result<Vec<String>> {
        let mut purged = Vec::new();
        for sheet in self.read_sheets()? {
            if now.signed_duration_since(sheet.updated_at) > max_age {
                self.delete(&sheet.id)?;
                purged.push(sheet.id);
            }
        }
        Ok(purged)
    }

    /// 디렉터리의 `*.json`을 전부 읽어 파싱한다. 실패한 파일은 조용히 건너뛴다.
    fn read_sheets(&self) -> Result<Vec<Sheet>> {
        let mut sheets = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let Ok(sheet) = serde_json::from_slice::<Sheet>(&bytes) else {
                continue;
            };
            sheets.push(sheet);
        }
        Ok(sheets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Entry, Location, Product};
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("test date must be valid")
    }

    fn store() -> (TempDir, SheetStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = SheetStore::open(dir.path().join("sheets")).expect("open store");
        (dir, store)
    }

    fn sheet_with_entry(created: NaiveDateTime, slot: NaiveDateTime, quantity: u32) -> Sheet {
        let mut sheet = Sheet::new(created);
        sheet
            .sections
            .get_mut(&Location::Store)
            .expect("store section")
            .insert(Product::Onigiri, vec![Entry { at: slot, quantity }]);
        sheet
    }

    #[test]
    fn open_creates_the_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b").join("sheets");
        let store = SheetStore::open(&nested).expect("open store");
        assert!(nested.is_dir());
        assert_eq!(store.dir(), nested.as_path());
    }

    #[test]
    fn path_of_rejects_unsafe_ids() {
        let (_dir, store) = store();
        assert!(matches!(store.path_of("../evil"), Err(Error::InvalidId(_))));
        assert!(matches!(store.path_of(""), Err(Error::InvalidId(_))));
        let path = store.path_of("20260830-080215").expect("valid id");
        assert_eq!(path, store.dir().join("20260830-080215.json"));
    }

    #[test]
    fn save_and_load_round_trip() {
        let (_dir, store) = store();
        let sheet = sheet_with_entry(at(2026, 8, 30, 8, 2), at(2026, 8, 30, 14, 0), 12);

        store.save(&sheet).expect("save");
        let loaded = store.load(&sheet.id).expect("load");
        assert_eq!(loaded, sheet);

        // 저장은 바뀐 내용도 덮어쓰며 임시 파일을 남기지 않는다.
        let mut updated = sheet.clone();
        updated.updated_at = at(2026, 8, 30, 8, 10);
        store.save(&updated).expect("save again");
        assert_eq!(store.load(&updated.id).expect("load overwritten"), updated);
        let leftovers: Vec<PathBuf> = fs::read_dir(store.dir())
            .expect("read_dir")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );
    }

    #[test]
    fn load_missing_is_not_found() {
        let (_dir, store) = store();
        match store.load("20260830-080215") {
            Err(Error::NotFound(id)) => assert_eq!(id, "20260830-080215"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn list_is_newest_first_and_skips_broken_files() {
        let (_dir, store) = store();
        let old = sheet_with_entry(at(2026, 8, 28, 9, 0), at(2026, 8, 29, 14, 0), 3);
        let middle = Sheet::new(at(2026, 8, 29, 9, 0));
        let new = sheet_with_entry(at(2026, 8, 30, 9, 0), at(2026, 8, 30, 14, 0), 12);
        store.save(&old).expect("save");
        store.save(&middle).expect("save");
        store.save(&new).expect("save");

        // 깨진 JSON 하나와 json이 아닌 파일 하나를 섞는다.
        fs::write(store.dir().join("broken.json"), b"{ not json").expect("write");
        fs::write(store.dir().join("notes.txt"), b"ignore me").expect("write");

        let listed = store.list().expect("list");
        let ids: Vec<&str> = listed.iter().map(|summary| summary.id.as_str()).collect();
        assert_eq!(ids, [new.id.as_str(), middle.id.as_str(), old.id.as_str()]);

        let newest = &listed[0];
        assert_eq!(newest.created_at, new.created_at);
        assert_eq!(newest.updated_at, new.updated_at);
        assert_eq!(newest.entry_count, 1);
        assert_eq!(newest.total_quantity, 12);
        assert_eq!(listed[1].entry_count, 0);
        assert_eq!(listed[1].total_quantity, 0);
    }

    #[test]
    fn delete_is_idempotent() {
        let (_dir, store) = store();
        let sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        store.save(&sheet).expect("save");

        store.delete(&sheet.id).expect("first delete");
        store.delete(&sheet.id).expect("second delete is a no-op");
        assert!(matches!(store.load(&sheet.id), Err(Error::NotFound(_))));
        assert!(store.list().expect("list").is_empty());
    }

    #[test]
    fn purge_removes_only_sheets_older_than_the_window() {
        let (_dir, store) = store();
        let now = at(2026, 8, 30, 9, 0);

        // updated_at이 정확히 30일 전 → 남는다(초과가 아니라 같음).
        let mut boundary = Sheet::new(at(2026, 7, 31, 9, 0));
        boundary.updated_at = at(2026, 7, 31, 9, 0);
        // 30일 + 1분 전 → 지운다.
        let mut stale = Sheet::new(at(2026, 7, 31, 8, 59));
        stale.updated_at = at(2026, 7, 31, 8, 59);
        // 최근 → 남는다.
        let fresh = Sheet::new(at(2026, 8, 29, 9, 0));

        store.save(&boundary).expect("save");
        store.save(&stale).expect("save");
        store.save(&fresh).expect("save");

        let purged = store
            .purge_older_than(now, Duration::days(RETENTION_DAYS))
            .expect("purge");
        assert_eq!(purged, vec![stale.id.clone()]);

        let remaining: Vec<String> = store
            .list()
            .expect("list")
            .into_iter()
            .map(|summary| summary.id)
            .collect();
        assert_eq!(remaining, vec![fresh.id.clone(), boundary.id.clone()]);
        assert!(matches!(store.load(&stale.id), Err(Error::NotFound(_))));
    }

    #[test]
    fn purge_uses_updated_at_not_created_at() {
        let (_dir, store) = store();
        let now = at(2026, 8, 30, 9, 0);

        // 오래 전에 만들었지만 어제 고친 조사표는 남아야 한다.
        let mut touched = Sheet::new(at(2026, 1, 1, 9, 0));
        touched.updated_at = at(2026, 8, 29, 9, 0);
        store.save(&touched).expect("save");

        let purged = store
            .purge_older_than(now, Duration::days(RETENTION_DAYS))
            .expect("purge");
        assert!(purged.is_empty());
        assert!(store.load(&touched.id).is_ok());
    }
}
