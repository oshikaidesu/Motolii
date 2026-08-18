//! 「最後に開いていた project」の置き場と形 — red 先行。
//!
//! 外部診断 F-01(`docs/reviews/2026-08-18-external-ux-diagnosis.md`)の受け皿。
//! 台本(`docs/ux-check-first-ten-minutes.md` P5「保存→再起動→続きがそのまま開く」)が
//! 成立するには、**窓が閉じたあとも残る場所**に最後の project が要る。
//!
//! 置き場と保存方式は palette settings と同じ user 設定層
//! (`~/Library/Application Support/Motolii/`、version 付き JSON、temp→rename)で、
//! 新しい保存方式は発明しない。このテストが固定するのはその1点である。

use std::path::{Path, PathBuf};

use motolii_ui::{
    default_last_project_path, default_user_palette_settings_path, load_last_project,
    remember_last_project, LastProjectError, LAST_PROJECT_FILE_NAME, LAST_PROJECT_VERSION,
};

fn store(name: &str) -> PathBuf {
    motolii_testkit::tmp_dir(name).join(LAST_PROJECT_FILE_NAME)
}

/// 覚えた path がそのまま戻る。**次の起動が読むのはこの1本だけ**である。
#[test]
fn a_remembered_project_comes_back_on_the_next_read() {
    let path = store("last_project_roundtrip");
    let project = PathBuf::from("/tmp/motolii/last-one.json");

    remember_last_project(&path, &project).expect("覚えられる");
    assert_eq!(
        load_last_project(&path).expect("読める"),
        Some(project.clone()),
        "覚えた project がそのまま戻る"
    );

    // 上書きは最後の1本だけを残す(recent files の一覧ではない)。
    let next = PathBuf::from("/tmp/motolii/another.json");
    remember_last_project(&path, &next).expect("覚え直せる");
    assert_eq!(load_last_project(&path).expect("読める"), Some(next));
}

/// 置き場が無い(初回起動)は**エラーではない** — 覚えていないだけ。
#[test]
fn a_missing_store_is_simply_nothing_remembered() {
    let path = store("last_project_missing");
    assert_eq!(
        load_last_project(&path).expect("初回起動は失敗ではない"),
        None
    );
}

/// 壊れた・知らない version は**黙って捨てない**。呼び手が帯へ理由を出せるよう
/// Err で返る(palette settings と同じ扱い)。
#[test]
fn a_corrupt_or_unknown_store_is_an_error_not_a_silent_none() {
    let path = store("last_project_corrupt");
    std::fs::write(&path, b"not json").expect("write");
    assert!(matches!(
        load_last_project(&path),
        Err(LastProjectError::Decode(_))
    ));

    std::fs::write(&path, br#"{"version":99,"path":"/tmp/x.json"}"#).expect("write");
    assert!(matches!(
        load_last_project(&path),
        Err(LastProjectError::UnsupportedVersion(99))
    ));
}

/// 置き場は palette settings の**隣**。設定の家を増やさない。
#[test]
fn the_store_lives_beside_the_other_user_settings() {
    let (Some(last), Some(palettes)) = (
        default_last_project_path(),
        default_user_palette_settings_path(),
    ) else {
        return; // HOME の無い環境
    };
    assert_eq!(
        last.parent(),
        palettes.parent(),
        "user 設定層は1つ(新しい置き場を作らない)"
    );
    assert_eq!(last.file_name(), Some(Path::new(LAST_PROJECT_FILE_NAME).as_os_str()));
    assert_eq!(LAST_PROJECT_VERSION, 1);
}
