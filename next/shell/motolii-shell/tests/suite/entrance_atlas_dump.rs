//! S 空間スコアの入口 atlas dump(`docs/ui-spatial-score.md` 「器具(実装計画)」1、
//! κ 調査 `docs/reviews/2026-08-21-ui-entrance-atlas-survey.md` 「器具化材料」)。
//!
//! `q0_fence.rs::collect_targets`(`target_walk.rs` へ共有抽出済み)で
//! フル `Shell::view()` を歩き、`id/x/y/w/h/content` の TSV を吐く。
//! **`screenshot.rs` の手計算幾何は使わない**(κ 調査が二重保守と明記)。
//!
//! この TSV が S1(到達コスト = Fitts)・S2(工程動線 = KLM)の座標源になる
//! — `scripts/s-score.py` が入口台帳(κ)・normal-map と突き合わせて表を出す。
//!
//! ## 2本のテスト
//!
//! - [`atlas_includes_known_widgets_within_window_bounds`]: 通常 test。
//!   fixture 状態(`Shell::new_fixture()`)の dump を直接(env var を介さず)
//!   関数呼び出しで検分する — 既知 widget(header の Undo/Redo/+Layer/Settings
//!   ボタンの文言)が content 列に現れ、全行の bounds が既定シミュレータ窓
//!   (`iced_test::simulator` の既定 = 1024×768、upstream `window::Settings::
//!   default()` 実測)の内側に収まることを見る。dump 実装前はこれが red で
//!   落ちる(oracle)。
//! - [`dump_entrance_atlas`]: dev 専用。`#[ignore]` — `MOTOLII_ATLAS_OUT` に
//!   出力先パスを明示した時だけファイルへ書く。env var 未設定なら何もせず
//!   正常終了する(CI を巻き込まない)。

use std::io::Write as _;

use iced_test::selector::Target;
use motolii_shell::Shell;

use crate::target_walk::collect_targets;

/// dump が吐く TSV のヘッダ行(κ 調査「器具化材料」で明示された列順)。
const HEADER: &str = "id\tx\ty\tw\th\tcontent";

/// `widget::Id` を読みやすい文字列へ畳む。`Id` は `Display` を実装しない
/// (upstream `iced_core::widget::id.rs` 実測 — `Debug` のみ)ので、
/// `Id(Custom("foo"))`/`Id(Unique(3))` という Debug 表現を剥がす。
/// 想定外の形が来ても崩れない(Debug 文字列をそのまま出すフォールバック)。
fn readable_id(id: &Option<iced::advanced::widget::Id>) -> String {
    let Some(id) = id else {
        return String::new();
    };
    let debug = format!("{id:?}");
    if let Some(inner) = debug
        .strip_prefix("Id(Custom(\"")
        .and_then(|s| s.strip_suffix("\"))"))
    {
        inner.to_owned()
    } else if let Some(inner) = debug
        .strip_prefix("Id(Unique(")
        .and_then(|s| s.strip_suffix("))"))
    {
        format!("unique:{inner}")
    } else {
        debug
    }
}

/// TSV の1フィールドとして安全な形へ(タブ・改行を潰す)。
fn tsv_field(raw: &str) -> String {
    raw.replace(['\t', '\n', '\r'], " ")
}

/// 1 `Target` を TSV の1行へ(`visible_bounds` が無い/面積ゼロなら `None`
/// — `q0_fence.rs::scan_state` と同じ「実際に画面上でクリックできる領域か」
/// の絞り込みを流用する)。
fn target_row(target: &Target) -> Option<String> {
    let bounds = target.visible_bounds()?;
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return None;
    }

    let (id, content) = match target {
        Target::Container { id, .. }
        | Target::Focusable { id, .. }
        | Target::Scrollable { id, .. }
        | Target::Custom { id, .. } => (readable_id(id), String::new()),
        Target::TextInput { id, content, .. } | Target::Text { id, content, .. } => {
            (readable_id(id), tsv_field(content))
        }
    };

    Some(format!(
        "{id}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{content}",
        bounds.x, bounds.y, bounds.width, bounds.height
    ))
}

/// `Shell::view()` を歩いて TSV を `out` へ書く。行順は `collect_targets` の
/// 収集順(DFS、`target_walk.rs` doc 参照)そのまま — 安定性は保証しない
/// (widget 追加/削除で入れ替わりうる)。S スコア計算側は id/content で
/// 突き合わせる前提。
pub fn write_atlas_tsv(shell: &Shell, out: &mut impl std::io::Write) -> std::io::Result<()> {
    let targets = collect_targets(shell.view());
    writeln!(out, "{HEADER}")?;
    for target in &targets {
        if let Some(row) = target_row(target) {
            writeln!(out, "{row}")?;
        }
    }
    Ok(())
}

/// 既定シミュレータ窓(`iced_test::simulator` → `window::Settings::default()`、
/// upstream 実測で 1024×768)。他の suite ファイルも size 指定なしで
/// `iced_test::simulator(element)` を使っており、この既定を前提にしている。
const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 768.0;

#[test]
fn atlas_includes_known_widgets_within_window_bounds() {
    let shell = Shell::new_fixture().0;
    let mut buffer = Vec::new();
    write_atlas_tsv(&shell, &mut buffer).expect("atlas TSV 書き出しに失敗");
    let text = String::from_utf8(buffer).expect("atlas TSV が UTF-8 でない");

    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some(HEADER),
        "TSV ヘッダが id/x/y/w/h/content でない"
    );

    let rows: Vec<Vec<&str>> = lines
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            assert_eq!(cols.len(), 6, "行の列数が6でない: {line:?}");
            cols
        })
        .collect();
    assert!(
        !rows.is_empty(),
        "atlas が空 — dump 実装前(red)か walker が壊れている"
    );

    // 既知 widget(header ボタンの文言、lib.rs 実測: Edit/Undo/Redo/+ Layer/
    // Browser/Settings)が content 列に現れる — フル Shell::view を歩けている
    // ことの証拠。"Edit" は M-menu MB-0+Edit(2026-08-22)で追加したメニューバー
    // トップレベルの新規 target(発注書 RETURN「atlas に menubar が現れる」)。
    // "Browser" は裁定162 切片 B3(map id980 Project/Media Pool panel の
    // 消化)で追加した header トグル。
    for expected in ["Edit", "Undo", "Redo", "+ Layer", "Browser", "Settings"] {
        assert!(
            rows.iter().any(|row| row[5] == expected),
            "atlas に既知 widget の content={expected:?} が無い — フル Shell::view を歩けていない疑い"
        );
    }

    // bounds が窓内(裁定 — S1 の距離計算は画面座標が前提)。
    for row in &rows {
        let x: f32 = row[1]
            .parse()
            .unwrap_or_else(|e| panic!("x が数値でない: {row:?}: {e}"));
        let y: f32 = row[2]
            .parse()
            .unwrap_or_else(|e| panic!("y が数値でない: {row:?}: {e}"));
        let w: f32 = row[3]
            .parse()
            .unwrap_or_else(|e| panic!("w が数値でない: {row:?}: {e}"));
        let h: f32 = row[4]
            .parse()
            .unwrap_or_else(|e| panic!("h が数値でない: {row:?}: {e}"));
        assert!(x >= -0.5 && y >= -0.5, "負座標へはみ出す行: {row:?}");
        assert!(
            x + w <= WINDOW_WIDTH + 0.5 && y + h <= WINDOW_HEIGHT + 0.5,
            "窓({WINDOW_WIDTH}x{WINDOW_HEIGHT})外にはみ出す行: {row:?}"
        );
    }
}

/// dev 専用の atlas dump 器具。明示実行の型: `MOTOLII_ATLAS_OUT=/path/to/atlas.tsv
/// cargo test -p motolii-shell --test suite_main -- --ignored dump_entrance_atlas`。
/// env var 未設定なら何もしない(CI を巻き込まない、κ 調査「器具化材料」の
/// 「CLI フラグ化は境界違反、tests/ 内 instrument が筋」を踏襲)。
#[test]
#[ignore = "dev器具: MOTOLII_ATLAS_OUT を明示指定した時だけ実行する"]
fn dump_entrance_atlas() {
    let Ok(out_path) = std::env::var("MOTOLII_ATLAS_OUT") else {
        eprintln!("MOTOLII_ATLAS_OUT 未設定 — atlas dump をスキップ");
        return;
    };

    let shell = Shell::new_fixture().0;
    let mut file = std::fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("{out_path} を作成できない: {e}"));
    write_atlas_tsv(&shell, &mut file).expect("atlas TSV 書き出しに失敗");
    file.flush().expect("atlas TSV の flush に失敗");
}
