//! 조사표를 기기 공용 사진 폴더에 PNG로 저장한다(ADR-0003).
//!
//! Android 11(API 30)+에서는 앱이 공용 미디어 폴더에 **자기 파일을 새로 만드는 것**이
//! 권한 없이 허용되고, FUSE 계층이 MediaStore에 자동 등록한다. 그래서 경로만 제대로
//! 유도하면 `std::fs`로 바로 쓸 수 있다. 경로 유도는 순수 함수라 여기서 테스트한다.

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::Sheet;
use crate::error::Result;
use crate::render;

/// 공용 사진 폴더 아래 이 앱이 쓰는 하위 폴더 이름.
pub const EXPORT_SUBDIR: &str = "소비기한";

/// 앱 전용 외부 저장소 경로에서 공용 저장소 루트를 가리키는 구성요소.
const ANDROID_DATA_COMPONENT: &str = "Android";
/// 공용 저장소 루트 아래의 사진 폴더 이름.
const PICTURES_COMPONENT: &str = "Pictures";

/// 내보내기 결과. 프론트엔드가 토스트로 경로를 보여준다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportResult {
    /// 저장된 파일의 전체 경로.
    pub path: String,
    /// 파일 이름만.
    pub file_name: String,
    /// 저장된 파일 크기(바이트).
    pub bytes: u64,
    /// 기기 미디어 인덱스(MediaStore) uri. shell이 Android에서 채우며, 데스크톱이나 등록 전에는 `None`.
    #[serde(default)]
    pub media_uri: Option<String>,
}

/// 앱 전용 Pictures 경로에서 기기 공용 Pictures 경로를 유도한다.
///
/// Android: `.../Android/data/<pkg>/files/Pictures` → `.../Pictures`
/// (첫 `Android` 구성요소의 부모 + `Pictures`).
/// `Android` 구성요소가 없으면(데스크톱) 입력을 그대로 돌려준다.
pub fn public_pictures_dir(app_pictures_dir: &Path) -> PathBuf {
    let mut root = PathBuf::new();
    for component in app_pictures_dir.components() {
        if matches!(component, Component::Normal(name) if name == OsStr::new(ANDROID_DATA_COMPONENT))
        {
            root.push(PICTURES_COMPONENT);
            return root;
        }
        root.push(component);
    }
    app_pictures_dir.to_path_buf()
}

/// 내보내기 파일 이름: `소비기한_2026-08-30_0802.png` (작성 시각 기준).
pub fn export_file_name(sheet: &Sheet) -> String {
    format!(
        "{}_{}.png",
        EXPORT_SUBDIR,
        sheet.created_at.format("%Y-%m-%d_%H%M")
    )
}

/// `pictures_dir/소비기한/`을 만들고 렌더링한 PNG를 저장한다. 같은 이름은 덮어쓴다.
pub fn export_png(sheet: &Sheet, pictures_dir: &Path) -> Result<ExportResult> {
    let dir = pictures_dir.join(EXPORT_SUBDIR);
    fs::create_dir_all(&dir)?;

    let file_name = export_file_name(sheet);
    let path = dir.join(&file_name);
    let png = render::render_png(sheet)?;
    fs::write(&path, &png)?;
    let bytes = fs::metadata(&path)?.len();

    Ok(ExportResult {
        path: path.to_string_lossy().into_owned(),
        file_name,
        bytes,
        media_uri: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Entry, Location, Product};
    use chrono::{NaiveDate, NaiveDateTime};

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("test date must be valid")
    }

    #[test]
    fn android_app_pictures_maps_to_the_public_folder() {
        let app =
            Path::new("/storage/emulated/0/Android/data/dev.dkk115.cubestbefore/files/Pictures");
        assert_eq!(
            public_pictures_dir(app),
            PathBuf::from("/storage/emulated/0/Pictures")
        );
    }

    #[test]
    fn android_mapping_uses_the_first_android_component() {
        let app = Path::new("/storage/emulated/0/Android/data/pkg/files/Android/Pictures");
        assert_eq!(
            public_pictures_dir(app),
            PathBuf::from("/storage/emulated/0/Pictures")
        );
    }

    #[test]
    fn desktop_path_is_returned_unchanged() {
        let desktop = Path::new("/home/user/Pictures");
        assert_eq!(public_pictures_dir(desktop), PathBuf::from(desktop));

        let windows = Path::new(r"C:\Users\user\Pictures");
        assert_eq!(public_pictures_dir(windows), PathBuf::from(windows));

        let relative = Path::new("pictures");
        assert_eq!(public_pictures_dir(relative), PathBuf::from("pictures"));
    }

    #[test]
    fn export_file_name_uses_created_at() {
        let sheet = Sheet::new(at(2026, 8, 30, 8, 2));
        assert_eq!(export_file_name(&sheet), "소비기한_2026-08-30_0802.png");

        let mut later = Sheet::new(at(2026, 12, 1, 22, 45));
        later.updated_at = at(2026, 12, 5, 9, 0);
        assert_eq!(export_file_name(&later), "소비기한_2026-12-01_2245.png");
    }

    #[test]
    fn export_png_writes_the_file_and_reports_its_size() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut sheet = Sheet::new(at(2026, 8, 30, 8, 2));
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

        let result = export_png(&sheet, dir.path()).expect("export");

        let expected = dir
            .path()
            .join(EXPORT_SUBDIR)
            .join("소비기한_2026-08-30_0802.png");
        assert_eq!(result.file_name, "소비기한_2026-08-30_0802.png");
        assert_eq!(Path::new(&result.path), expected);
        assert!(expected.is_file());

        let written = fs::read(&expected).expect("read back");
        assert_eq!(result.bytes, written.len() as u64);
        assert_eq!(written, render::render_png(&sheet).expect("render"));

        // 같은 이름으로 다시 내보내면 덮어쓴다(파일이 하나만 남는다).
        export_png(&sheet, dir.path()).expect("export again");
        let count = fs::read_dir(dir.path().join(EXPORT_SUBDIR))
            .expect("read_dir")
            .count();
        assert_eq!(count, 1);
    }
}
