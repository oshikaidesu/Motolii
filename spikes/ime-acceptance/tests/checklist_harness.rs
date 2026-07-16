//! チェックリスト4項目のテストハーネス雛形。
//! 実機IME審判は `#[ignore]` — 開発主機で `cargo test -- --ignored` を手動実行。

use ime_acceptance::{AcceptanceManifest, ChecklistEntry, ChecklistId, PlatformRun, Verdict};

#[test]
fn checklist_has_four_items() {
    assert_eq!(ChecklistId::ALL.len(), 4);
    let specs: Vec<_> = ChecklistId::ALL.iter().map(|id| id.spec_text()).collect();
    assert!(specs.iter().all(|s| !s.is_empty()));
}

#[test]
fn skeleton_manifest_all_pending() {
    let m = AcceptanceManifest::skeleton_template();
    assert_eq!(m.overall, Verdict::Pending);
    assert_eq!(m.ticket, "M3-GUARD-1");
    for platform in &m.platforms {
        assert_eq!(platform.overall, Verdict::Pending);
        for entry in &platform.entries {
            assert_eq!(entry.verdict, Verdict::Pending);
        }
    }
}

#[test]
fn manifest_serializes_to_json() {
    let json = serde_json::to_string(&AcceptanceManifest::skeleton_template()).unwrap();
    assert!(json.contains("preedit_underline"));
    assert!(json.contains("pending"));
}

/// 実機審判用テンプレ — 人手で各項目を記録してから manifest を更新する。
#[test]
#[ignore = "GUI+IME実機のみ。開発主機で cargo test -- --ignored record_manual_template"]
fn record_manual_template() {
    let run = PlatformRun {
        platform: std::env::var("IME_PLATFORM").unwrap_or_else(|_| "(記入)".into()),
        ime_backend: std::env::var("IME_BACKEND").unwrap_or_else(|_| "(記入)".into()),
        display_server: std::env::var("IME_DISPLAY").unwrap_or_else(|_| "(記入)".into()),
        entries: ChecklistId::ALL
            .map(|id| ChecklistEntry {
                id,
                spec: id.spec_text().into(),
                verdict: Verdict::Pending,
                notes: id.manual_steps().into(),
            })
            .to_vec(),
        overall: Verdict::Pending,
    };

    eprintln!("=== IME手動審判テンプレ ===");
    for entry in &run.entries {
        eprintln!("[{}] {}", format!("{:?}", entry.id), entry.spec);
        eprintln!("  手順: {}", entry.notes);
        eprintln!("  判定: pending → 実機で pass/fail を記入");
    }
    eprintln!("記録先: docs/spikes/ime-acceptance.md");
}
