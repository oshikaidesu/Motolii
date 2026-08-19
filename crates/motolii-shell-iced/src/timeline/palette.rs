//! Timeline canvas の色を1箇所へ集める — 2026-08-19 UIトンマナ統一 campaign
//! レーンD。ここに置いてよい値は3種類だけ(**新しい raw hex の発明は0件**):
//!
//! 1. supervisor 指定のクリップ8色([`CLIP_COLORS`])
//! 2. `crate::theme::Tokens` の既存 role([`param_chip_color`] など)
//! 3. `canvas.rs` から移した、Tokens の21 role に完全一致する値が無い
//!    raw 定数(値は変えず、出所コメントを付けてここへ置くだけ)
//!
//! ## なぜ集約するか
//!
//! 実測(2026-08-19): `timeline/keys.rs` の `param_chip_color`(Position=青系
//! `(0.55, 0.72, 0.86)`)と `timeline/structure.rs` の `param_color`
//! (Position=赤系 `0xcf7d6d` 系)が、同じ `ParamRef::Position` に対して別々の
//! 色を返していた。前者は `canvas.rs` の描画ループから実際に呼ばれる経路
//! (`RowKind::Property` の分岐で `continue` する前に呼ばれる)、後者は
//! **呼ばれない**(同じ分岐がその前に無条件 `continue` するので
//! `structure::draw_property_row` へは到達しない — 2026-08-19 実測、
//! `canvas.rs` 内の旧コードは削除した)。到達可能かどうかに関わらず、
//! 同じ意味の色が2箇所に別実装で存在すること自体が「場所によってチップ色が
//! 変わる」不整合の温床なので、ここへ一本化する。
//!
//! ## クリップ色の永続化経路(EVIDENCE_GAP)
//!
//! egui 版 `crates/motolii-ui/src/timeline_editor/mod.rs:5585` の
//! `layer_color` は `TrackItem::Clip/Group.envelope.color`(`Option<u32>`、
//! ユーザーが右クリック `Colour ▸` メニューで選ぶと `SetClipColor` 相当の
//! command で Document へ書かれる、`crates/motolii-doc/src/command/apply.rs:428`
//! `find_envelope_mut(doc, *target)?.color = *new`)を最優先で読み、無ければ
//! `LayerId` から `LAYER_COLORS[id % 8]` を導く。
//!
//! **この iced 側の canvas は `envelope.color` を1箇所も読まない** —
//! [`clip_color`] は常に `LayerId` からの導出だけである。つまり:
//! - この関数の8色を差し替えれば、iced 側の絵は**必ず**変わる(Document 由来
//!   の色で上書きされる経路が無いので、置換が絵に反映されない心配は無い)。
//! - 逆に、egui 側の `Colour ▸` メニューでユーザーが個別クリップに色を
//!   選んでいても、iced 側はそれを無視して常に id 由来の色を描く
//!   (2026-08-19 時点の既知の乖離。Document/command の意味は変えない柵の
//!   範囲内なので、ここでは直さず報告だけする)。

use iced::Color;

use motolii_doc::LayerId;
use motolii_ui::timeline_rows::ParamRef;

use crate::theme::Tokens;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// ── クリップ8色 ─────────────────────────────────────────────────────────
//
// supervisor 指定・Browser モックの設計済み8色
// (`docs/mocks-ui/public/browser-library.css:252-261` の `.thumb-*` 実値を
// そのまま写す — Browser のサムネイル色と Timeline のクリップ色を揃える)。
// egui 版 `LAYER_COLORS`(`crates/motolii-ui/src/timeline_editor/mod.rs:108`)
// は自己申告で「仮のパレット・当て馬・ここで発明した色を正本にしない」
// (同ファイル 106-108 行)としており、この8色が後継の正本になる。
//
// | index | thumb class | hex |
// |---|---|---|
// | 0 | `.thumb-blue`    | `#5d7899` |
// | 1 | `.thumb-purple`  | `#746398` |
// | 2 | `.thumb-ochre`   | `#88704e` |
// | 3 | `.thumb-green`   | `#557f6d` |
// | 4 | `.thumb-red`     | `#875459` |
// | 5 | `.thumb-slate`   | `#546a75` |
// | 6 | `.thumb-magenta` | `#8a5b83` |
// | 7 | `.thumb-cyan`    | `#477d81` |
pub const CLIP_COLORS: [Color; 8] = [
    rgb(0x5d, 0x78, 0x99),
    rgb(0x74, 0x63, 0x98),
    rgb(0x88, 0x70, 0x4e),
    rgb(0x55, 0x7f, 0x6d),
    rgb(0x87, 0x54, 0x59),
    rgb(0x54, 0x6a, 0x75),
    rgb(0x8a, 0x5b, 0x83),
    rgb(0x47, 0x7d, 0x81),
];

/// `LayerId` からクリップ/行スウォッチの色を選ぶ。**Document の
/// `envelope.color` は読まない**(モジュール doc の EVIDENCE_GAP 参照) —
/// 採番後に変わらず再利用もされない `LayerId` からの導出だけを行う
/// (egui 版 `layer_color` の `None` 分岐と同じ導出則)。
pub fn clip_color(layer: LayerId) -> Color {
    CLIP_COLORS[(layer.get() % CLIP_COLORS.len() as u64) as usize]
}

// ── param チップ色 ──────────────────────────────────────────────────────

/// property 行のチップ色。**全 param 同色**(虹色を正当化する根拠が無い —
/// 上記モジュール doc の実測どおり、`ParamRef::Position` に対して keys.rs と
/// structure.rs が別々の色を返していた)。
///
/// 色の選び方は Inspector 側の key ボタン語彙(2026-08-13 裁定、
/// `crates/motolii-shell-iced/src/widgets/key_button.rs` の
/// `KeyState`/`look`)をそのまま写す: `Unkeyed => text_secondary` /
/// `Animated`・`Current => action_active`。この2状態(idle/keyed)へ潰した
/// のはこの関数の仕事 — この行の param が(playhead に限らず)どこかに
/// キーを持てば `Animated` 相当として accent にする。Inspector 側の別の
/// 語彙である `row_band_color`(`inspector_pane.rs:723` — Position=data /
/// Rotation=action_active / Scale=shape / Opacity=text_secondary という
/// **param 別の固定色**)とは別物なので混同しない — あちらは param の
/// 識別色、こちらは「キーの有無」の状態色。
pub fn param_chip_color(tokens: &Tokens, keyed: bool) -> Color {
    if keyed {
        tokens.action_active
    } else {
        tokens.text_secondary
    }
}

/// property 行のラベル文字列。keys.rs / structure.rs で完全一致していた
/// 重複(`param_label`)をここへ統一。
pub fn param_label(param: ParamRef) -> &'static str {
    match param {
        ParamRef::Position => "Position",
        ParamRef::Anchor => "Anchor",
        ParamRef::Scale => "Scale",
        ParamRef::Rotation => "Rotation",
        ParamRef::Opacity => "Opacity",
    }
}

// ── canvas.rs から移した raw 定数 ────────────────────────────────────────
//
// 出所: egui 版 `timeline_editor/mod.rs` の mock_tokens 採用値をそのまま写した
// もの(見た目の正本は mock_tokens — canvas.rs はここで色を発明していない、
// という元コメントの経緯ごと移した)。`theme::Tokens` の21 role と1つずつ
// 突き合わせたところ、**完全一致したのは無い**(`WAVE_BG` だけは
// `#222222` == `Tokens::surface_raised` と厳密一致したので、そちらは定数を
// 削除して呼び出し側で `tokens.surface_raised` を直接使う形に置き換えた —
// `canvas.rs` 参照)。残りは値を変えずここへ再配置するだけ。
pub const BG: Color = rgb(0x29, 0x29, 0x29);
pub const HEAD_BG: Color = rgb(0x38, 0x38, 0x38);
pub const CELL: Color = rgb(0x36, 0x36, 0x36);
pub const TRACK_A: Color = rgb(0x37, 0x37, 0x37);
pub const TRACK_B: Color = rgb(0x25, 0x25, 0x25);
pub const RULE: Color = rgb(0x11, 0x11, 0x11);
pub const INK: Color = rgb(0xd4, 0xd4, 0xd4);
pub const DIM: Color = rgb(0x8d, 0x8d, 0x8d);
/// 元は playhead に使っていた gold(#e9cf72)。timeline-library.css:93,96 では
/// この色は `.timeGuide`(drag 中の snap 案内線)のもので、`.playhead` は白
/// (2026-08-18 訂正、`SELECTED` へ差し替え済み)。
pub const SELECTED: Color = rgb(0xf2, 0xf2, 0xf2);
pub const WAVE: Color = rgb(0x7f, 0x92, 0x8c);
pub const TRIM_BAND: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.28);
/// ARRANGEMENT 俯瞰帯の下地。`timeline-library.css:3` `.overview{background:#242424}`
/// (`TRACK_B` の `#252525` とほぼ同値だが、帯とトラック縞は別要素なので
/// 取り違えないよう別定数にする)。
pub const OVERVIEW_BG: Color = rgb(0x24, 0x24, 0x24);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_colors_are_the_supervisor_specified_eight() {
        // browser-library.css:252-261 の .thumb-* 実値と1対1で一致するはず。
        let expected = [
            Color::from_rgb8(0x5d, 0x78, 0x99),
            Color::from_rgb8(0x74, 0x63, 0x98),
            Color::from_rgb8(0x88, 0x70, 0x4e),
            Color::from_rgb8(0x55, 0x7f, 0x6d),
            Color::from_rgb8(0x87, 0x54, 0x59),
            Color::from_rgb8(0x54, 0x6a, 0x75),
            Color::from_rgb8(0x8a, 0x5b, 0x83),
            Color::from_rgb8(0x47, 0x7d, 0x81),
        ];
        assert_eq!(CLIP_COLORS, expected);
    }

    #[test]
    fn clip_color_wraps_around_eight_by_layer_id() {
        assert_eq!(clip_color(LayerId::from_raw(0)), CLIP_COLORS[0]);
        assert_eq!(clip_color(LayerId::from_raw(7)), CLIP_COLORS[7]);
        assert_eq!(clip_color(LayerId::from_raw(8)), CLIP_COLORS[0]);
        assert_eq!(clip_color(LayerId::from_raw(15)), CLIP_COLORS[7]);
    }

    #[test]
    fn idle_param_chip_is_text_secondary_regardless_of_param() {
        let t = &Tokens::DARK;
        assert_eq!(param_chip_color(t, false), t.text_secondary);
        // 「全 param 同色」を字面どおりに固定する: 呼び出し側で param を
        // 変えても keyed が false である限り同じ色が返る(このAPIは param
        // を引数に取らないので、シグネチャ自体がそれを保証するが、将来
        // 引数が増えた事故に備えて明示しておく)。
        assert_ne!(t.text_secondary, t.action_active, "対比が無いと次のテストが空証明になる");
    }

    #[test]
    fn keyed_param_chip_is_accent() {
        let t = &Tokens::DARK;
        assert_eq!(param_chip_color(t, true), t.action_active);
    }

    #[test]
    fn param_label_covers_all_five_params() {
        assert_eq!(param_label(ParamRef::Position), "Position");
        assert_eq!(param_label(ParamRef::Anchor), "Anchor");
        assert_eq!(param_label(ParamRef::Scale), "Scale");
        assert_eq!(param_label(ParamRef::Rotation), "Rotation");
        assert_eq!(param_label(ParamRef::Opacity), "Opacity");
    }
}
