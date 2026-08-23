//! 「追加できる語彙は、すべてブラウザに札がある」か(裁定2026-08-22
//! 「追加するものは Browser の中に全部入れる」の機械照合)。
//!
//! `message_dispatch_fence.rs`/`entries_fence.rs` と同じ形 — 判定ロジックは
//! `scripts/check_browser_entries.py` 1本に集約し、ここは exit code を
//! Rust の柵として買い戻すだけ(スクリプトの中身は複製しない)。
//!
//! 落ちたら: 出力が挙げる `PathSource`/`OpKind` バリアントに対応する札を
//! `next/ui/motolii-browser-pane/src/model/tabs.rs` の `CREATE_PREVIEW`/
//! `EFFECTS_PREVIEW` へ足す(`CreateKind`/`SelectionAction` へ対応する
//! variant も併せて足す — `every_create_card_declares_its_create_kind`/
//! `effects_action_cards_declare_their_selection_action` の2オラクルが
//! 形を縛っている)。

use std::process::Command;

#[test]
fn every_reachable_vocabulary_has_a_browser_card() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_browser_entries.py"))
        .arg(&root)
        .output()
        .expect("check_browser_entries.py を起動できない");
    assert!(
        out.status.success(),
        "型・描画・書き出しは在るのにブラウザに札が無い語彙が在る:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}
