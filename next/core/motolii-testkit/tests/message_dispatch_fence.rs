//! `Message` の全 variant が、どこかの `dispatch_*` に届いているか。
//!
//! 2026-08-23(SP-1): `Shell::update` は**一つの網羅 match** から8領域の
//! chain-of-responsibility へ分かれた。読みやすさは上がったが、代わりに
//! **コンパイラの網羅検査を失っている** — 腕を書き忘れた variant は
//! `dispatch_message` 末尾の `Err(_unhandled) => Task::none()` へ落ち、
//! **押しても何も起きない**枝として黙って増える(M13「無反応ゼロ」に反する)。
//!
//! コンパイル時には戻せない(どの dispatcher も「他が消費した」を証明できない)。
//! 失った検査をここで**文字列として**買い戻す。
//!
//! 落ちたら: 出力の variant を扱う腕を、対応する領域の `dispatch_*` へ足す。

use std::process::Command;

#[test]
fn every_message_variant_reaches_a_dispatcher() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_message_dispatch.py"))
        .arg(&root)
        .output()
        .expect("check_message_dispatch.py を起動できない");
    assert!(
        out.status.success(),
        "どの dispatch_* にも届かない Message variant が在る:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}
