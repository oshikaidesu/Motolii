//! 運転席 — 窓を開けずに shell を動かす。
//!
//! 見るのは背骨1(書き口が1箇所)と M13(拒否が必ず出る)と、
//! **描画キャッシュが `revision()` で正しく落ちること**。

use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

#[test]
fn adding_a_layer_shows_up_and_undo_takes_it_back() {
    let mut shell = shell();
    assert_eq!(shell.layer_count(), 0);

    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1, "layer を足しても増えない");

    shell.update(Message::Undo);
    // AddLayer は AddLayer + SetMeta の2 intent なので、1回の Undo では meta だけ戻る。
    // **これが M10「1 gesture = 1 Undo」がまだ未達である証拠**(GOALS 参照)。
    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "2回戻しても layer が消えない");
}

#[test]
fn rejection_reaches_the_status_band() {
    let mut shell = shell();
    shell.update(Message::Undo);
    assert_eq!(
        shell.status(),
        Some("これ以上戻せない"),
        "戻せない時に何も言わないのは M13 違反(無反応ゼロ)"
    );

    // 次の操作で理由が消えること(古い理由が居座らない)。
    shell.update(Message::AddLayer);
    assert_eq!(shell.status(), None);
}

#[test]
fn frame_cache_follows_revision_and_playhead() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let first = shell.frame_token().expect("frame");

    // 同じ入力なら描き直さない。
    shell.update(Message::Select(motolii_store::LayerId(1)));
    assert_eq!(shell.frame_token(), Some(first.clone()), "選択だけで描き直している");

    // 再生位置が動いたら描き直す。
    shell.update(Message::ScrubTo(10));
    assert_ne!(shell.frame_token(), Some(first.clone()), "scrub で描き直していない");

    // undo で Document が戻ったら描き直す(store 世代は変わらないので
    // `revision()` が edit 位置も見ていないとここが落ちる)。
    let scrubbed = shell.frame_token().expect("frame");
    shell.update(Message::Undo);
    assert_ne!(shell.frame_token(), Some(scrubbed), "undo で描き直していない");
}
