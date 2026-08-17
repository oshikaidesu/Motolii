//! 黙殺フェンス: blitz_shell の失敗は stderr ではなく transcript(=status帯+ログ)へ言う。
//!
//! 2026-08-18 の実測で、Stage 構築失敗・composition 失敗・document 読めない等が
//! `eprintln!` だけで消え、窓は**黙って空白**になることが分かった
//! (pane.rs:573-575, 586-588, 595-598, 655-657, 664-670, 772, 821)。
//! CLI から GUI を検証する運転席では「失敗が言われない」ことが構造的に許されないため、
//! pane の失敗は全て `ShellTranscript` を通す(帯に出て、`--status-log` に残る)。
//!
//! 走査型のフェンスは tests/diagnostic.rs の source-text assert と同型。

#[test]
fn pane_failures_do_not_go_to_stderr_only() {
    let source = include_str!("../src/blitz_shell/pane.rs");
    let count = source.matches("eprintln!").count();
    assert_eq!(
        count, 0,
        "blitz_shell/pane.rs に eprintln! が {count} 箇所残っている — \
         pane の失敗は ShellTranscript へ言う(status帯と --status-log に出す)。\
         stderr 専用の黙殺は運転席の合格条件に反する"
    );
}
