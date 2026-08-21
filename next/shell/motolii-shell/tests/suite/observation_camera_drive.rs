//! 運転席 — 観測カメラ(裁定157)の Shell 側配線(S1〜S3)。
//!
//! `stage.rs` の純関数(ズーム/パン/フレーム枠の計算)は `motolii-stage-pane`
//! crate 側の `tests`(裁定160 切片10で `motolii-shell/src/stage.rs` から
//! 抽出済み、`next/ui/motolii-stage-pane/src/lib.rs` 内 `#[cfg(test)] mod
//! tests`)が直接縛る — ここは `Shell::update`/`refresh_frame` の配線
//! (発注書 ORACLE (a)〜(d))と screenshot 器具の `--observe` 経路を見る。
//!
//! **canvas は iced_test に見えない**(KNOWN)ので、実 widget を経由せず
//! `Message::Stage(stage::Message::Observe(_))`/
//! `Message::Stage(stage::Message::ResetToRenderCamera)` を直接構成して
//! `Shell::update` へ渡す(`TimelineDragMoved` 等、既存 drive 試験と同じ形 —
//! 「widget が計算した結果」を Message として直接届ける)。

use motolii_engine::ObservationCamera;
use motolii_shell::{stage, Message, Shell};

/// **中身のある** Shell(層1枚)。空の comp は既定背景1色で埋まるだけなので、
/// ズーム/パンしても画素が変わらない(観測カメラがどこを向いても同じ単色) —
/// 「絵が変わる」を確かめる (b) には、位置に応じて画素が変わる中身が要る。
fn shell() -> Shell {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    shell
}

fn zoomed() -> ObservationCamera {
    ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 1.8,
    }
}

/// (a) 既定(`observation` が `None`)の frame は、この機能が無かった頃と同じ
/// `Engine::render_frame` の結果そのまま — 観測カメラに一切触れていない
/// 状態で `frame_rgba()` が空でないことをまず確かめる(以降の (c)(d) の基準点)。
#[test]
fn a_default_observation_is_none_and_frame_is_the_plain_render() {
    let shell = shell();
    assert_eq!(shell.observation(), None, "既定は「カメラを通して見る」のはず");
    assert!(shell.frame_rgba().is_some(), "既定でも Stage に絵が出ているはず(M17)");
    assert!(
        shell.observation_rgba().is_none(),
        "観測カメラが無効なら observation_rgba は無いはず"
    );
}

/// (b) ズーム相当の操作(`Message::Stage(stage::Message::Observe(_))`)を1回出すと、
/// `observation` が `None → Some` になり、Stage の表示 RGBA
/// (`observation_rgba()`)が export 真値(`frame_rgba()`)とは違う絵になる。
#[test]
fn b_observing_transitions_to_some_and_changes_the_displayed_frame() {
    let mut shell = shell();
    let baseline_export = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("既定でも frame がある");

    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));

    assert_eq!(shell.observation(), Some(zoomed()), "observation が Some になっていない");
    let observed = shell
        .observation_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("観測中は observation_rgba があるはず");
    assert_eq!(
        (observed.0, observed.1),
        (baseline_export.0, baseline_export.1),
        "解像度は comp と同じはず"
    );
    assert_ne!(
        observed.2, baseline_export.2,
        "ズームしたのに Stage の見た目(observation_rgba)が変わっていない"
    );
}

/// (c) `Message::Stage(stage::Message::ResetToRenderCamera)` で `None` へ戻ると、export 真値
/// (`frame_rgba()`)はズーム前とバイト一致する(往復して汚染が残らない)。
#[test]
fn c_reset_to_render_camera_returns_to_none_and_the_export_rgba_round_trips() {
    let mut shell = shell();
    let baseline = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("既定でも frame がある");

    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));
    assert!(shell.observation().is_some());

    let _ = shell.update(Message::Stage(stage::Message::ResetToRenderCamera));

    assert_eq!(shell.observation(), None, "カメラへ戻るはずが observation が残っている");
    assert!(
        shell.observation_rgba().is_none(),
        "None に戻ったら observation_rgba も消えるはず(枠 overlay の「自明のため描かない」と対応)"
    );
    let after = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");
    assert_eq!(
        baseline, after,
        "観測カメラを往復したら export 真値はズーム前とバイト一致するはず"
    );
}

/// (d) **汚染ゼロの直接証明**: 観測カメラが有効な間(`Some`)でも、
/// `frame_rgba()`(export/screenshot が読む生値)はレンダリングカメラの絵の
/// まま — ズーム前と一切変わらない。`checkerboard_toggle_never_touches_the_raw_export_rgba`
/// (`drive.rs`)と対の試験。
#[test]
fn d_frame_rgba_stays_the_render_camera_picture_while_observing() {
    let mut shell = shell();
    let before = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));

    let while_observing = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    assert_eq!(
        before, while_observing,
        "観測カメラが有効な間に export 用の生 rgba が変わってしまった\
         (裁定157 の最重要不変: export 経路は observation を一切知らない)"
    );
}

/// 観測カメラは Document を一切触らない — 表示専用状態(`checkerboard` と
/// 同格、`docs/reviews/2026-08-21-camera-seam-survey.md` §3)。undo 履歴にも
/// 乗らない(`can_undo` が変わらない)。
#[test]
fn observing_never_touches_the_document_or_undo_history() {
    let mut shell = shell();
    let can_undo_before = shell.can_undo();
    let layer_count_before = shell.layer_count();

    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));
    let _ = shell.update(Message::Stage(stage::Message::ResetToRenderCamera));

    assert_eq!(shell.can_undo(), can_undo_before, "観測カメラが undo 履歴を汚した");
    assert_eq!(shell.layer_count(), layer_count_before, "観測カメラが Document を書き換えた");
}

/// 同じ観測カメラ値をもう一度送っても、`observation_rgba()` の中身は変わらない
/// (再送でパニックしない・別の値へ勝手にドリフトしない)。**Handle 再生成の
/// 回数そのもの**は `metrics` static を共有する専用バイナリでしか安全に測れない
/// ので、そちらは `render_pipeline_fence.rs::idle_observation_does_not_recreate_the_stage_handle`
/// が縛る(このファイルの冒頭 doc 参照)。
#[test]
fn repeating_the_same_observation_keeps_the_displayed_pixels_stable() {
    let mut shell = shell();
    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));
    let first = shell
        .observation_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("観測中は observation_rgba があるはず");

    let _ = shell.update(Message::Stage(stage::Message::Observe(zoomed())));
    let second = shell
        .observation_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("観測中は observation_rgba があるはず");

    assert_eq!(first, second, "同じ観測カメラを再送したら同じ絵になるはず");
}

/// **構造的隔離の柵、assembler 側の片割れ**(縫い目調査 §3「担保案」、裁定160
/// 切片10で `observation_preview_source` 自体が `motolii-stage-pane` crate へ
/// 移設された)。`Engine::render_frame_with_view_camera` の唯一の呼び手
/// (`motolii-stage-pane::observation_preview_source`)はもう `motolii-shell`
/// の `src/` には無い — この柵はその逆、**`motolii-shell` 自身が直接呼んで
/// いない**ことを縛る(export(`screenshot.rs`・`main.rs`・`motolii-export`)が
/// pane crate を経由せず観測カメラの存在を知ってしまう抜け道を防ぐ)。対になる
/// 「唯一の呼び手が1箇所だけ」の柵は `motolii-stage-pane` crate 側
/// (`next/ui/motolii-stage-pane/src/lib.rs` 内
/// `tests::render_frame_with_view_camera_is_only_called_from_this_crate`)に
/// 移設済み。ソーステキストを grep するだけの軽い柵(`tonmana_token_fence.rs`
/// と同型の「ソースを読んで具体的な行で落とす」手段)。
#[test]
fn motolii_shell_never_calls_render_frame_with_view_camera_directly() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut call_sites: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&src_dir).expect("src/ を読める") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read src file");
        for (line_no, line) in text.lines().enumerate() {
            // `.render_frame_with_view_camera(` = 実際の呼び出し(メソッド呼び出し
            // 構文)。コメント内の言及(バッククォート付きの名指し)は
            // `render_frame_with_view_camera(` の直前が `.` ではないので拾わない。
            if line.contains(".render_frame_with_view_camera(") {
                call_sites.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
            }
        }
    }
    assert!(
        call_sites.is_empty(),
        "motolii-shell の src/ が render_frame_with_view_camera を直接呼んでいる\
         (export 経路が pane crate を経由せず観測カメラを知ってしまった可能性 —\
         裁定157の最重要不変。呼ぶのは motolii-stage-pane::observation_preview_source\
         だけのはず):\n{}",
        call_sites.join("\n")
    );
}
