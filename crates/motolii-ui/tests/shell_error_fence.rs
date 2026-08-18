//! 黙殺フェンス: 面の失敗は stderr でも `let _` でも panic でもなく transcript へ言う。
//!
//! 2026-08-18 の実測で、Stage 構築失敗・composition 失敗・document 読めない等が
//! `eprintln!` だけで消え、窓は**黙って空白**になることが分かった
//! (pane.rs:573-575, 586-588, 595-598, 655-657, 664-670, 772, 821)。
//! CLI から GUI を検証する運転席では「失敗が言われない」ことが構造的に許されないため、
//! pane の失敗は全て `ShellTranscript` を通す(帯に出て、`--status-log` に残る)。
//!
//! 外部診断(`docs/reviews/2026-08-18-external-ux-diagnosis.md`)の D 類で、
//! 同じ黙殺が pane の外に4件残っていることが分かった。この fence はそこまで広げる:
//!
//! | 件 | 場所 | 黙殺の形 |
//! |----|------|----------|
//! | F-07 | `timeline_editor` | 拒否が `status` にだけ出て shell へ返らない |
//! | F-08 | `rerun_stage` | `copy_gpu_image` 失敗の無言 return と `let _ =` |
//! | F-09 | `browser_panel` / `browser_blitz::thumbnail` | `eprintln!` と `.ok()?` |
//! | F-10 | `export_seat` | spawn 失敗で `expect` = panic |
//!
//! 走査型のフェンスは tests/diagnostic.rs の source-text assert と同型。
//!
//! **`let _ =` の全面禁止はしない。** 「受け手が居ないだけ」の破棄
//! (`sender.send` / `handle.join`)は黙殺ではない。禁じるのは
//! *失敗を運ぶ戻り値の破棄* だけで、走査で書けるものは名指しし、
//! 書けないものはここに理由を書く。

use std::path::{Path, PathBuf};

use motolii_ui::browser_panel::BrowserPanel;

/// 黙殺を数える対象。`(表示名, crate 相対 path, source)` で、
/// どの面が何箇所残しているかが落ちた時に読める。
const SILENCED_MODULES: &[(&str, &str, &str)] = &[
    (
        "blitz_shell/pane.rs",
        "src/blitz_shell/pane.rs",
        include_str!("../src/blitz_shell/pane.rs"),
    ),
    (
        "browser_panel/mod.rs (F-09)",
        "src/browser_panel/mod.rs",
        include_str!("../src/browser_panel/mod.rs"),
    ),
    (
        "browser_blitz/thumbnail.rs (F-09)",
        "src/browser_blitz/thumbnail.rs",
        include_str!("../src/browser_blitz/thumbnail.rs"),
    ),
    (
        "rerun_stage/adapter.rs (F-08)",
        "src/rerun_stage/adapter.rs",
        include_str!("../src/rerun_stage/adapter.rs"),
    ),
    (
        "rerun_stage/host_mesh.rs (F-08)",
        "src/rerun_stage/host_mesh.rs",
        include_str!("../src/rerun_stage/host_mesh.rs"),
    ),
    (
        "export_seat.rs (F-10)",
        "src/export_seat.rs",
        include_str!("../src/export_seat.rs"),
    ),
    (
        "timeline_editor/mod.rs (F-07)",
        "src/timeline_editor/mod.rs",
        include_str!("../src/timeline_editor/mod.rs"),
    ),
];

/// 製品側だけを見る(テストの中の `expect` は運転席の合格条件と関係ない)。
fn product_half(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

/// 行 comment を落とした source。**禁じたいのは呼び出しであって、
/// 「なぜ禁じたか」を書いた注記ではない** — 注記まで数えると、この規則の
/// 由来を source に書き残せなくなる。
fn code_only(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn pane_failures_do_not_go_to_stderr_only() {
    let source = code_only(include_str!("../src/blitz_shell/pane.rs"));
    let count = source.matches("eprintln!").count();
    assert_eq!(
        count, 0,
        "blitz_shell/pane.rs に eprintln! が {count} 箇所残っている — \
         pane の失敗は ShellTranscript へ言う(status帯と --status-log に出す)。\
         stderr 専用の黙殺は運転席の合格条件に反する"
    );
}

/// D 類の4件が居る面まで同じ規則を広げる。
#[test]
fn the_silenced_four_do_not_go_to_stderr_only() {
    let offenders: Vec<String> = SILENCED_MODULES
        .iter()
        .filter_map(|(label, _, source)| {
            let count = code_only(source).matches("eprintln!").count();
            (count > 0).then(|| format!("{label}: {count} 箇所"))
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "eprintln! が残っている面がある({}) — \
         失敗は呼び手(shell)へ返し、ShellTranscript が言う。\
         窓しか無い運転席では stderr は誰も読まない",
        offenders.join(" / ")
    );
}

/// F-08: 合成フレームを載せる失敗と、その後の幾何再適用の失敗を捨てない。
///
/// Stage は shell を知らないので `Result` / outbox で**返す**のが仕事で、
/// 帯に出すのは pane 側(latch つき)。
#[test]
fn stage_present_failures_are_not_dropped_on_the_floor() {
    let adapter = code_only(include_str!("../src/rerun_stage/adapter.rs"));
    assert!(
        !adapter.contains("let _ = self.apply_host_stage_geometry"),
        "rerun_stage/adapter.rs が幾何の再適用失敗を `let _ =` で捨てている(F-08) — \
         絵は載ったのに fill が隠れないまま黙って残る"
    );
    assert!(
        adapter.contains("fn take_failures"),
        "rerun_stage/adapter.rs に失敗の受け渡し口(take_failures)が無い(F-08) — \
         copy_gpu_image の失敗が無言 return で消える"
    );
    let pane = code_only(include_str!("../src/blitz_shell/pane.rs"));
    assert!(
        pane.contains("take_failures()"),
        "blitz_shell/pane.rs が Stage の失敗を引き取っていない(F-08) — \
         返す口を作っても、読む側が居なければ黙殺のまま"
    );
}

/// F-10: 書き出し thread が起きない時に panic しない。
///
/// panic すると帯に何も出ないまま窓ごと落ちる。返事(`ExportFinish::Failed`)なら
/// `poll_export` が「export failed: …」を帯に出す。
#[test]
fn export_start_does_not_panic_when_the_thread_will_not_start() {
    let source = code_only(product_half(include_str!("../src/export_seat.rs")));
    assert!(
        !source.contains(".expect("),
        "export_seat.rs の製品側に .expect( が残っている(F-10) — \
         spawn 失敗は panic ではなく ExportFinish::Failed で帯へ言う"
    );
    assert!(
        !source.contains("panic!"),
        "export_seat.rs の製品側に panic! が残っている(F-10)"
    );
}

/// F-07: 落ちた編集の理由を timeline が**返す**。transcript 型は持ち込まない。
#[test]
fn the_timeline_hands_its_refusals_back_instead_of_only_painting_them() {
    let editor = code_only(include_str!("../src/timeline_editor/mod.rs"));
    assert!(
        !editor.contains("ShellTranscript"),
        "timeline_editor が shell の台帳型を知ってしまっている(F-07 の非目標) — \
         editor は理由を返すだけで、言うのは pane / shell 側"
    );
    assert!(
        editor.contains("fn take_rejections"),
        "timeline_editor に拒否の受け渡し口(take_rejections)が無い(F-07) — \
         親 lock 等の拒否が Timeline 内描画だけで終わり、帯にも log にも残らない"
    );
    let pane = code_only(include_str!("../src/blitz_shell/pane.rs"));
    assert!(
        pane.contains("take_rejections()"),
        "blitz_shell/pane.rs が timeline の拒否を引き取っていない(F-07)"
    );
}

/// F-09: 縮小実体を作れない/読めない理由を、`None` cache と一緒に**返す**。
///
/// 再試行の方針は変えない(`None` を覚えて再試行しないまま)。変えるのは
/// 「理由が1度は外へ出る」ことだけである。
#[test]
fn a_thumbnail_that_cannot_be_read_says_why() {
    let dir = std::env::temp_dir().join(format!(
        "motolii-fence-thumb-{}-{}",
        std::process::id(),
        line!()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the fixture folder");
    // 拡張子は画像、中身は画像ではない。縮小実体を作る所で必ず落ちる。
    let broken = dir.join("broken.png");
    std::fs::write(&broken, b"this is not a png").expect("write the broken image");

    let mut panel = BrowserPanel::with_root(dir.clone());
    let notices = panel.take_notices();

    assert!(
        notices.iter().any(|notice| notice.contains("broken.png")),
        "縮小実体を作れない理由が返ってこない(F-09) — 出たのは {notices:?}。\
         card は glyph のまま出てよいが、なぜ画像が無いのかは1度は言う"
    );
    assert!(
        panel.take_notices().is_empty(),
        "take_notices は引き取ったら空になる(同じ理由を毎フレーム帯へ流さない)"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// フェンスが実在の path を見ていること。`include_str!` は消えた file で
/// コンパイルが落ちるが、path を書き換えて空 file を指す事故は防げない。
#[test]
fn the_fenced_modules_are_real_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (label, path, source) in SILENCED_MODULES {
        let full = root.join(path);
        assert!(
            Path::new(&full).is_file(),
            "{label}: {} が無い — フェンスの見張り先が消えている",
            full.display()
        );
        assert!(!source.is_empty(), "{label}: 中身が空");
    }
}
