//! 右クリック(コンテキストメニュー)基盤+文脈別項目正本(発注 2026-08-22)。
//!
//! ## 1. overlay 実測: 任意座標に開く口は無い
//!
//! [`vendored::menu`] の `MenuBar`/`Menu`/`Item`(iced_aw vendoring)は、開いた
//! menu の位置を **root item(バー項目)自身の layout 上の bounds** から算出する
//! — `menu_bar_overlay.rs` 実測: `MenuBarOverlay::layout` が `bar_bounds`/
//! `parent_bounds`(`roots_layout` = `MenuBar::layout` が `flex::resolve` で
//! 水平配置した root 列の bounds)を起点に子 menu を上下左右へ展開する。
//! `MenuBar`/`Menu` の public builder(`new`/`width`/`height`/`spacing`/
//! `padding`/`offset`/`style`/`draw_path`/`scroll_speed`/`close_on_*`)には
//! `Point`/座標を受け取る口が1つも無い — 開く位置は「どの root item に
//! 属するか」でしか決まらず、任意のスクリーン座標(右クリック位置)を渡す
//! public API は存在しない(実測、`grep -n "pub fn" menu_bar.rs menu_tree.rs`)。
//!
//! 結論: **無い**。vendored の menu 機構(トリガー hover/click → 開閉状態 →
//! 隣接展開)は「メニューバーの1本」という前提に強く結びついており、右クリック
//! の「その場に出す」用途には転用できない。
//!
//! ### 代替案: `stack`+`opaque`
//!
//! 発注書は「Settings モーダルの手本」と書いていたが、実装を確認すると
//! Settings は `stack`+`opaque` ではなく**別の OS window**(`Shell::settings_window:
//! Option<iced::window::Id>`、`next/shell/motolii-shell/src/lib.rs:791`)で
//! 実現されていた — 発注書の前提はこの1点で外れている(RETURN の逸脱)。
//! とはいえ `stack`+`opaque` は iced 標準の modal 手口(`advanced` feature、
//! workspace の `iced` 依存で有効済み)であり、この crate 単体で完結できる
//! ([`open_at`] 参照) — 呼び手(shell)が「今どこで右クリックされたか」
//! (`Point`)を state に持ち、`Some(point)` の間だけ `open_at(content, point,
//! items, dims, colors)` で包む(`None` なら `content` をそのまま返す)、
//! という条件分岐だけで足りる。vendored の開閉状態機械は使わない —
//! 開閉は呼び手の `Option<Point>` state がそのまま表現する(Q0: 閉じていれば
//! menu の要素は widget木に存在しない、という oracle と同じ形)。
//!
//! ## 2. 文脈別の項目正本
//!
//! 発注書が挙げた4文脈それぞれについて、**既存 `Message` が実在し、かつ
//! S6(メニューバー or shortcut に同じ動詞が既にある)を満たす項目だけ**を
//! 関数化した。満たさない項目は見送り — 存在しない redundancy を偽装しない
//! (`menus.rs` の 見送り表と同じ規律)。判定根拠は [`SHORTCUT_ONLY_REGISTRY`]
//! と `crate::menus::{edit_menu, layer_menu}` の実項目(この crate 内で
//! 直接呼べる分は関数呼び出しで実クロスチェック、shell 側にしか無い分は
//! 転記+出典行で追跡可能にした)。
//!
//! ### クリップ右クリック(発注書の8項目 → 裁定195で7項目採用・1項目見送り)
//!
//! | label | message 出典 | S6 根拠 | 判定 |
//! |---|---|---|---|
//! | Copy | 既存(shell `Message::CopyLayer`) | Edit メニュー(`crate::menus::edit_menu`)+ Cmd+C | 採用 |
//! | Paste | 既存(`Message::PasteLayer`) | 同上 + Cmd+V | 採用 |
//! | Duplicate | 既存(`Message::DuplicateLayer`) | 同上 + Cmd+D | 採用 |
//! | Cut | 既存(`Message::CutLayer`) | 同上 + Cmd+X | 採用(発注書の「Delete」枠 — 下記注記) |
//! | Group | 既存(`Message::GroupLayers`) | Layer メニュー(`crate::menus::layer_menu`)+ Cmd+G | 採用 |
//! | Freeze | 既存(`Message::FreezeGroups`) | 同上(shortcut 無しだが menu bar 存在で足りる、OR 条件) | 採用 |
//! | Split | **存在しない** — `motolii_timeline_pane::split::Message::SplitAtPlayhead` は宣言のみで `write::Message`/shell へ未統合(`split.rs` 冒頭 doc「統合手順(次波)」1〜3) | — | 見送り(次波の統合を待つ) |
//! | ラベル色(`Label Color`) | 既存(`inspector_pane::Message::CycleLabelColor`) | **裁定195で解禁**: `crate::menus::layer_menu` に `Label Color` 項目を追加した(Inspector の色 swatch button 1本だけだった単一入口を穴埋め) — 今は menu bar が2本目の入口 | 採用 |
//!
//! **「Delete」についての注記**: normal-map/shell の実装に「クリップボードを
//! 汚さない削除」は存在しない — 唯一の削除動詞は `Message::CutLayer`
//! (Copy+`Intent::RemoveLayer` 1回、`next/shell/motolii-shell/src/lib.rs:1566`
//! 冒頭 doc)。存在しない「Delete」を発明せず、実在する動詞のラベル
//! (`Cut`)をそのまま出す — 意図優先の原則(裁定174)は「機構を語らない」であって
//! 「存在しない動詞を語る」ことは許さない。
//!
//! ### レイヤー行右クリック(発注書の5カテゴリ → 裁定195で全カテゴリ採用)
//!
//! | label | message 出典 | S6 根拠 | 判定 |
//! |---|---|---|---|
//! | Rename | 既存(`timeline_pane::Message::RenameBegin(layer)`) | shortcut Enter(`next/shell/motolii-shell/src/lib.rs:5391`) | 採用 |
//! | Bring Forward / Send Backward / Bring to Front / Send to Back | 既存(`timeline_pane::Message::RestackLayer(StackDirection::_)`) | shortcut Cmd+Opt(+Shift)+Up/Down(`lib.rs:5399-5417`) | 採用(4項目) |
//! | Hide(`ToggleMute`) | 既存 | **裁定195で解禁**: `crate::menus::layer_menu` に `Hide` 項目を追加した(`rail.rs:337` の mute glyph button だけだった単一入口を穴埋め) | 採用 |
//! | Solo(`ToggleSolo`) | 既存 | 同上(`layer_menu` `Solo` 項目、`rail.rs:338`) | 採用 |
//! | Lock(`ToggleLock`) | 既存 | 同上(`layer_menu` `Lock` 項目、`rail.rs:339`) | 採用 |
//!
//! 前レーンが見送った理由(rail の glyph button のみ = 単一入口、Ableton
//! 可視性原理・S6 違反)は裁定195の発注そのもの — `menus.rs` の
//! `layer_menu` に Hide/Solo/Lock を追加したことで S6 が成立し、この波で
//! 右クリックへ復帰させた。並びは rail の描画順(M/S/L)を踏襲し、
//! restack 系(並び替え)の後ろへ置く。
//!
//! ### キャンバス右クリック(発注書の3項目、全採用)
//!
//! | label | message 出典 | S6 根拠 |
//! |---|---|---|
//! | Select All | 既存(`Message::SelectAllLayers`) | Edit メニュー + Cmd+A |
//! | Deselect All | 既存(`Message::DeselectAllLayers`) | Edit メニュー + Cmd+Shift+A |
//! | Group | 既存(`Message::GroupLayers`) | Layer メニュー + Cmd+G |
//!
//! ### キーフレーム右クリック(発注書の3カテゴリ → 6項目、全採用)
//!
//! | label | message 出典 | S6 根拠 |
//! |---|---|---|
//! | Interpolation: Hold/Linear/Easy Ease/Easy Ease In/Easy Ease Out | 既存(`timeline_pane::Message::SetKeyInterp`) | shell `menu.rs` Edit メニュー末尾(`next/shell/motolii-shell/src/menu.rs:112-142`、この crate の `edit_menu` にはまだ未統合 — shell 側で足された分) |
//! | Delete | 既存(`timeline_pane::Message::DeleteSelectedKeys`) | shortcut Backspace/Delete(`next/shell/motolii-shell/src/lib.rs:5146-5152`) |
//!
//! ## 型の形
//!
//! `menus.rs` と同じジェネリック message-bag(`XxxMessages<M>` フィールド +
//! `xxx_context_items<M>(items) -> Vec<Item<M>>`)。トップレベルの `Menu`
//! (バー表示ラベル)は持たない — 文脈メニューに「バーの1本」という概念は
//! 無いため、`Vec<Item<M>>` のまま返し、呼び手が [`open_at`] へ渡す。

use crate::{leaf, Item};
use iced::widget::{column, container, opaque, stack};
use iced::{Border, Element, Length, Padding, Point};
use motolii_tokens_rs::{Colors, Dimensions};

/// shell の shortcut 実装(menu bar に出典が無い項目)の転記。この crate は
/// shell に依存しない層なので直接参照はできない — S6 監査はラベル文字列の
/// 一致で行う。転記元は各行のコメントに出典(ファイル:行)を明記した。
/// 出典が崩れたら(shell 側のキー割当が変わったら)この表を追随修正すること。
pub const SHORTCUT_ONLY_REGISTRY: &[(&str, &str)] = &[
    // next/shell/motolii-shell/src/lib.rs:5391
    // `Key::Named(Named::Enter) => Some(Message::RenameSelectedLayer)`
    ("Rename", "Enter"),
    // next/shell/motolii-shell/src/lib.rs:5399-5417
    // Cmd+Alt(+Shift)+Up/Down → `timeline_pane::Message::RestackLayer(StackDirection::_)`
    ("Bring Forward", "Cmd+Opt+Up"),
    ("Send Backward", "Cmd+Opt+Down"),
    ("Bring to Front", "Cmd+Opt+Shift+Up"),
    ("Send to Back", "Cmd+Opt+Shift+Down"),
    // next/shell/motolii-shell/src/lib.rs:5146-5152
    // Backspace/Delete(Mac は両方 `Named::Backspace` で届く実測注記あり)
    // → `timeline_pane::Message::DeleteSelectedKeys`
    ("Delete", "Backspace"),
];

/// [`clip_context_items`] が必要とする message 一式(モジュール冒頭 doc の
/// 「クリップ右クリック」表と一致させること)。
pub struct ClipContextMessages<M> {
    /// normal-map 429、Edit メニュー Copy と同じ動詞。
    pub copy: M,
    /// normal-map 430、Edit メニュー Paste と同じ動詞。
    pub paste: M,
    /// normal-map 434、Edit メニュー Duplicate と同じ動詞。
    pub duplicate: M,
    /// normal-map 432、Edit メニュー Cut と同じ動詞(発注書の「Delete」枠 —
    /// モジュール冒頭 doc「Delete についての注記」参照)。
    pub cut: M,
    /// bundle B34、Layer メニュー Group と同じ動詞。
    pub group: M,
    /// 裁定119、Layer メニュー Freeze と同じ動詞。
    pub freeze: M,
    /// `inspector_pane::Message::CycleLabelColor`。裁定195で解禁 — Layer
    /// メニュー `Label Color` と同じ動詞(モジュール冒頭 doc「クリップ
    /// 右クリック」表参照)。
    pub cycle_label_color: M,
}

/// クリップ右クリックの項目列を組む。並び・ラベルの正本はこの関数
/// (モジュール冒頭 doc の表と一致させること)。
pub fn clip_context_items<M>(items: ClipContextMessages<M>) -> Vec<Item<M>> {
    vec![
        Item { label: "Copy", shortcut: Some("Cmd+C"), message: items.copy },
        Item { label: "Paste", shortcut: Some("Cmd+V"), message: items.paste },
        Item { label: "Duplicate", shortcut: Some("Cmd+D"), message: items.duplicate },
        Item { label: "Cut", shortcut: Some("Cmd+X"), message: items.cut },
        Item { label: "Group", shortcut: Some("Cmd+G"), message: items.group },
        Item { label: "Freeze", shortcut: None, message: items.freeze },
        // 裁定195: 単一入口の穴埋め(Inspector の色 swatch のみだった)。
        Item { label: "Label Color", shortcut: None, message: items.cycle_label_color },
    ]
}

/// [`layer_row_context_items`] が必要とする message 一式(裁定195で
/// lock/hide/solo も採用 — モジュール冒頭 doc「レイヤー行右クリック」表参照)。
pub struct LayerRowContextMessages<M> {
    /// 正典 §6、shortcut Enter(`SHORTCUT_ONLY_REGISTRY`)。
    pub rename: M,
    /// `StackDirection::Forward`、shortcut Cmd+Opt+Up。
    pub bring_forward: M,
    /// `StackDirection::Backward`、shortcut Cmd+Opt+Down。
    pub send_backward: M,
    /// `StackDirection::ToFront`、shortcut Cmd+Opt+Shift+Up。
    pub bring_to_front: M,
    /// `StackDirection::ToBack`、shortcut Cmd+Opt+Shift+Down。
    pub send_to_back: M,
    /// `timeline_pane::Message::ToggleMute`。裁定195で解禁 — Layer メニュー
    /// `Hide` と同じ動詞。
    pub toggle_hide: M,
    /// `timeline_pane::Message::ToggleSolo`。裁定195で解禁 — Layer メニュー
    /// `Solo` と同じ動詞。
    pub toggle_solo: M,
    /// `timeline_pane::Message::ToggleLock`。裁定195で解禁 — Layer メニュー
    /// `Lock` と同じ動詞。
    pub toggle_lock: M,
}

/// レイヤー行右クリックの項目列を組む。並びは rename → restack(4項目)→
/// 可視性/ロック(rail の M/S/L 描画順、裁定195で追加)。
pub fn layer_row_context_items<M>(items: LayerRowContextMessages<M>) -> Vec<Item<M>> {
    vec![
        Item { label: "Rename", shortcut: Some("Enter"), message: items.rename },
        Item {
            label: "Bring Forward",
            shortcut: Some("Cmd+Opt+Up"),
            message: items.bring_forward,
        },
        Item {
            label: "Send Backward",
            shortcut: Some("Cmd+Opt+Down"),
            message: items.send_backward,
        },
        Item {
            label: "Bring to Front",
            shortcut: Some("Cmd+Opt+Shift+Up"),
            message: items.bring_to_front,
        },
        Item {
            label: "Send to Back",
            shortcut: Some("Cmd+Opt+Shift+Down"),
            message: items.send_to_back,
        },
        // 裁定195: 単一入口の穴埋め(rail の glyph button のみだった)。
        Item { label: "Hide", shortcut: None, message: items.toggle_hide },
        Item { label: "Solo", shortcut: None, message: items.toggle_solo },
        Item { label: "Lock", shortcut: None, message: items.toggle_lock },
    ]
}

/// [`canvas_context_items`] が必要とする message 一式。3項目とも S6 を満たす
/// (モジュール冒頭 doc「キャンバス右クリック」表参照)。
pub struct CanvasContextMessages<M> {
    /// normal-map 436、bundle B31。Edit メニュー Select All と同じ動詞。
    pub select_all: M,
    /// normal-map 433、bundle B31。Edit メニュー Deselect All と同じ動詞。
    pub deselect_all: M,
    /// bundle B34。Layer メニュー Group と同じ動詞。
    pub group: M,
}

/// キャンバス右クリックの項目列を組む。
pub fn canvas_context_items<M>(items: CanvasContextMessages<M>) -> Vec<Item<M>> {
    vec![
        Item { label: "Select All", shortcut: Some("Cmd+A"), message: items.select_all },
        Item {
            label: "Deselect All",
            shortcut: Some("Cmd+Shift+A"),
            message: items.deselect_all,
        },
        Item { label: "Group", shortcut: Some("Cmd+G"), message: items.group },
    ]
}

/// [`keyframe_context_items`] が必要とする message 一式。6項目とも S6 を
/// 満たす(モジュール冒頭 doc「キーフレーム右クリック」表参照)。
pub struct KeyframeContextMessages<M> {
    /// `Interp::Hold`。shell `menu.rs` Edit メニュー末尾と同じ動詞。
    pub hold: M,
    /// `Interp::Linear`。同上。
    pub linear: M,
    /// `timeline_pane::EASY_EASE`。同上。
    pub easy_ease: M,
    /// `timeline_pane::EASY_EASE_IN`。同上。
    pub easy_ease_in: M,
    /// `timeline_pane::EASY_EASE_OUT`。同上。
    pub easy_ease_out: M,
    /// `DeleteSelectedKeys`。shortcut Backspace/Delete(`SHORTCUT_ONLY_REGISTRY`)。
    pub delete: M,
}

/// キーフレーム右クリックの項目列を組む。
pub fn keyframe_context_items<M>(items: KeyframeContextMessages<M>) -> Vec<Item<M>> {
    vec![
        Item { label: "Interpolation: Hold", shortcut: None, message: items.hold },
        Item { label: "Interpolation: Linear", shortcut: None, message: items.linear },
        Item {
            label: "Interpolation: Easy Ease",
            shortcut: None,
            message: items.easy_ease,
        },
        Item {
            label: "Interpolation: Easy Ease In",
            shortcut: None,
            message: items.easy_ease_in,
        },
        Item {
            label: "Interpolation: Easy Ease Out",
            shortcut: None,
            message: items.easy_ease_out,
        },
        Item { label: "Delete", shortcut: Some("Backspace"), message: items.delete },
    ]
}

/// `base` の上に文脈メニュー(`items`)を `anchor`(右クリック位置)へ重ねる。
/// モジュール冒頭 doc「1. overlay 実測」— vendored の開閉状態機械は使わない、
/// 呼び手が `Option<Point>` を持ち `Some` の間だけこの関数を通す形(Q0 は
/// 呼び手側の条件分岐が担う、この関数自体は「開いている」前提で常に描く)。
///
/// `stack`+`opaque`(iced 標準、`advanced` feature — workspace の `iced`
/// 依存で有効)。`opaque` は base 側からの pointer/click を完全に奪う
/// (upstream 標準の modal 手口と同じ) — 項目 click 以外(base の裏側)は
/// 反応しない。位置決めは `container` の非対称 `padding`(top/left =
/// `anchor`)で表現する — vendored 側のような layout-bounds 起点ではなく、
/// 「全面 `Length::Fill` の透明層の中で `anchor` だけずらす」単純な方法
/// (`Padding` の right/bottom は 0、他ウィジェットが決める複雑な bounds
/// 計算を持ち込まない)。
pub fn open_at<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    anchor: Point,
    items: Vec<Item<Message>>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let leaves: Vec<Element<'a, Message>> =
        items.into_iter().map(|item| leaf(item, dims, colors)).collect();

    let panel = container(column(leaves).spacing(dims.spacing_xs))
        .padding(dims.spacing_xs)
        .width(Length::Fixed(dims.menubar_menu_width))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            border: Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: dims.menubar_corner_radius.into(),
            },
            ..container::Style::default()
        });

    let positioned = container(panel).width(Length::Fill).height(Length::Fill).padding(Padding {
        top: anchor.y,
        right: 0.0,
        bottom: 0.0,
        left: anchor.x,
    });

    // `Stack` の既定 size は `Length::Fit` — 自身の大きさを base layer(1層目)の
    // 実寸に合わせて決める(upstream `stack.rs` doc「The first Element dictates
    // the intrinsic Size」+ `layout()` 実装)。呼び手の `base` が独自の
    // width/height を持たない(例: 素の `text` 等)場合、stack 自体がその小さい
    // 実寸に潰れ、内側の `Length::Fill` overlay 層もろとも潰れる ——
    // 冒頭 doc の「全面 `Length::Fill` の透明層」を実現するには stack 自身に
    // 明示で Fill を指定する必要がある(base の sizing に依存しない)。
    stack(vec![base.into(), opaque(positioned)])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus::{edit_menu, layer_menu, EditMenuMessages, LayerMenuMessages};
    use iced::widget::text;

    #[derive(Debug, Clone, PartialEq)]
    struct FakeMessage(&'static str);

    fn edit_fake_messages() -> EditMenuMessages<FakeMessage> {
        EditMenuMessages {
            undo: FakeMessage("undo"),
            redo: FakeMessage("redo"),
            cut: FakeMessage("cut"),
            copy: FakeMessage("copy"),
            paste: FakeMessage("paste"),
            duplicate: FakeMessage("duplicate"),
            select_all: FakeMessage("select_all"),
            deselect_all: FakeMessage("deselect_all"),
        }
    }

    fn layer_fake_messages() -> LayerMenuMessages<FakeMessage> {
        LayerMenuMessages {
            new_layer: FakeMessage("new_layer"),
            group: FakeMessage("group"),
            ungroup: FakeMessage("ungroup"),
            freeze: FakeMessage("freeze"),
            unfreeze: FakeMessage("unfreeze"),
            toggle_hide: FakeMessage("toggle_hide"),
            toggle_solo: FakeMessage("toggle_solo"),
            toggle_lock: FakeMessage("toggle_lock"),
            cycle_label_color: FakeMessage("cycle_label_color"),
        }
    }

    /// この crate 内の `edit_menu`/`layer_menu`(実クロスチェック)+ shell
    /// `menu.rs` の Interpolation 5項目(この crate には無い、モジュール
    /// 冒頭 doc「キーフレーム右クリック」表の出典行のとおり転記)。
    fn menu_bar_labels() -> Vec<&'static str> {
        let mut labels: Vec<&'static str> =
            edit_menu(edit_fake_messages()).items.into_iter().map(|item| item.label).collect();
        labels.extend(layer_menu(layer_fake_messages()).items.into_iter().map(|item| item.label));
        labels.extend([
            "Interpolation: Hold",
            "Interpolation: Linear",
            "Interpolation: Easy Ease",
            "Interpolation: Easy Ease In",
            "Interpolation: Easy Ease Out",
        ]);
        labels
    }

    /// S6 併存の機械検査本体: `label` がメニューバー(`menu_bar`)か
    /// [`SHORTCUT_ONLY_REGISTRY`](shortcut)のどちらかに存在すること。
    fn assert_s6_compliant(label: &str, menu_bar: &[&'static str]) {
        let in_menu_bar = menu_bar.contains(&label);
        let in_shortcut_registry =
            SHORTCUT_ONLY_REGISTRY.iter().any(|(registered, _)| *registered == label);
        assert!(
            in_menu_bar || in_shortcut_registry,
            "S6 違反(Ableton 可視性原理): '{label}' がメニューバーにも shortcut \
             registry にも存在しない — 右クリックが唯一の入口になっている"
        );
    }

    fn clip_fake_messages() -> ClipContextMessages<FakeMessage> {
        ClipContextMessages {
            copy: FakeMessage("copy"),
            paste: FakeMessage("paste"),
            duplicate: FakeMessage("duplicate"),
            cut: FakeMessage("cut"),
            group: FakeMessage("group"),
            freeze: FakeMessage("freeze"),
            cycle_label_color: FakeMessage("cycle_label_color"),
        }
    }

    fn layer_row_fake_messages() -> LayerRowContextMessages<FakeMessage> {
        LayerRowContextMessages {
            rename: FakeMessage("rename"),
            bring_forward: FakeMessage("bring_forward"),
            send_backward: FakeMessage("send_backward"),
            bring_to_front: FakeMessage("bring_to_front"),
            send_to_back: FakeMessage("send_to_back"),
            toggle_hide: FakeMessage("toggle_hide"),
            toggle_solo: FakeMessage("toggle_solo"),
            toggle_lock: FakeMessage("toggle_lock"),
        }
    }

    fn canvas_fake_messages() -> CanvasContextMessages<FakeMessage> {
        CanvasContextMessages {
            select_all: FakeMessage("select_all"),
            deselect_all: FakeMessage("deselect_all"),
            group: FakeMessage("group"),
        }
    }

    fn keyframe_fake_messages() -> KeyframeContextMessages<FakeMessage> {
        KeyframeContextMessages {
            hold: FakeMessage("hold"),
            linear: FakeMessage("linear"),
            easy_ease: FakeMessage("easy_ease"),
            easy_ease_in: FakeMessage("easy_ease_in"),
            easy_ease_out: FakeMessage("easy_ease_out"),
            delete: FakeMessage("delete"),
        }
    }

    #[test]
    fn clip_context_has_seven_items_in_declared_order() {
        let labels: Vec<&str> =
            clip_context_items(clip_fake_messages()).into_iter().map(|item| item.label).collect();
        assert_eq!(
            labels,
            vec!["Copy", "Paste", "Duplicate", "Cut", "Group", "Freeze", "Label Color"]
        );
    }

    #[test]
    fn clip_context_items_carry_the_expected_message() {
        let messages: Vec<FakeMessage> = clip_context_items(clip_fake_messages())
            .into_iter()
            .map(|item| item.message)
            .collect();
        assert_eq!(
            messages,
            vec![
                FakeMessage("copy"),
                FakeMessage("paste"),
                FakeMessage("duplicate"),
                FakeMessage("cut"),
                FakeMessage("group"),
                FakeMessage("freeze"),
                FakeMessage("cycle_label_color"),
            ]
        );
    }

    /// S6 機械検査: クリップ右クリックの全項目がメニューバーか shortcut に
    /// 既に存在する(発注書の Split は Message 未統合のためこの関数に現れず
    /// 検査対象に含まれない。ラベル色は裁定195で `layer_menu` に
    /// `Label Color` を足したことで S6 が成立し、この検査に含まれるように
    /// なった — モジュール冒頭 doc の表参照)。
    #[test]
    fn clip_context_items_are_all_s6_compliant() {
        let menu_bar = menu_bar_labels();
        for item in clip_context_items(clip_fake_messages()) {
            assert_s6_compliant(item.label, &menu_bar);
        }
    }

    #[test]
    fn layer_row_context_has_eight_items_in_declared_order() {
        let labels: Vec<&str> = layer_row_context_items(layer_row_fake_messages())
            .into_iter()
            .map(|item| item.label)
            .collect();
        assert_eq!(
            labels,
            vec![
                "Rename",
                "Bring Forward",
                "Send Backward",
                "Bring to Front",
                "Send to Back",
                "Hide",
                "Solo",
                "Lock",
            ]
        );
    }

    #[test]
    fn layer_row_context_items_carry_the_expected_message() {
        let messages: Vec<FakeMessage> = layer_row_context_items(layer_row_fake_messages())
            .into_iter()
            .map(|item| item.message)
            .collect();
        assert_eq!(
            messages,
            vec![
                FakeMessage("rename"),
                FakeMessage("bring_forward"),
                FakeMessage("send_backward"),
                FakeMessage("bring_to_front"),
                FakeMessage("send_to_back"),
                FakeMessage("toggle_hide"),
                FakeMessage("toggle_solo"),
                FakeMessage("toggle_lock"),
            ]
        );
    }

    /// S6 機械検査: レイヤー行右クリック。裁定195で `layer_menu` に
    /// Hide/Solo/Lock を足したことで8項目全てが S6 を満たすようになった
    /// (以前は lock/hide/solo が関数に現れず検査対象外だった — モジュール
    /// 冒頭 doc の表参照)。
    #[test]
    fn layer_row_context_items_are_all_s6_compliant() {
        let menu_bar = menu_bar_labels();
        for item in layer_row_context_items(layer_row_fake_messages()) {
            assert_s6_compliant(item.label, &menu_bar);
        }
    }

    #[test]
    fn canvas_context_has_three_items_in_declared_order() {
        let labels: Vec<&str> = canvas_context_items(canvas_fake_messages())
            .into_iter()
            .map(|item| item.label)
            .collect();
        assert_eq!(labels, vec!["Select All", "Deselect All", "Group"]);
    }

    #[test]
    fn canvas_context_items_carry_the_expected_message() {
        let messages: Vec<FakeMessage> = canvas_context_items(canvas_fake_messages())
            .into_iter()
            .map(|item| item.message)
            .collect();
        assert_eq!(
            messages,
            vec![FakeMessage("select_all"), FakeMessage("deselect_all"), FakeMessage("group")]
        );
    }

    #[test]
    fn canvas_context_items_are_all_s6_compliant() {
        let menu_bar = menu_bar_labels();
        for item in canvas_context_items(canvas_fake_messages()) {
            assert_s6_compliant(item.label, &menu_bar);
        }
    }

    #[test]
    fn keyframe_context_has_six_items_in_declared_order() {
        let labels: Vec<&str> = keyframe_context_items(keyframe_fake_messages())
            .into_iter()
            .map(|item| item.label)
            .collect();
        assert_eq!(
            labels,
            vec![
                "Interpolation: Hold",
                "Interpolation: Linear",
                "Interpolation: Easy Ease",
                "Interpolation: Easy Ease In",
                "Interpolation: Easy Ease Out",
                "Delete",
            ]
        );
    }

    #[test]
    fn keyframe_context_items_carry_the_expected_message() {
        let messages: Vec<FakeMessage> = keyframe_context_items(keyframe_fake_messages())
            .into_iter()
            .map(|item| item.message)
            .collect();
        assert_eq!(
            messages,
            vec![
                FakeMessage("hold"),
                FakeMessage("linear"),
                FakeMessage("easy_ease"),
                FakeMessage("easy_ease_in"),
                FakeMessage("easy_ease_out"),
                FakeMessage("delete"),
            ]
        );
    }

    #[test]
    fn keyframe_context_items_are_all_s6_compliant() {
        let menu_bar = menu_bar_labels();
        for item in keyframe_context_items(keyframe_fake_messages()) {
            assert_s6_compliant(item.label, &menu_bar);
        }
    }

    /// `open_at` oracle: base の上に項目が重なり、項目 click で `message` が
    /// publish される(`tests/menubar_oracle.rs` と同じ `iced_test::Simulator`
    /// 手口)。この項目は S6 監査の対象外(vendored oracle と同様、plumbing の
    /// 実証だけが目的 — 実在しない仮のラベルを使う)。
    #[test]
    fn open_at_places_items_over_the_base_and_clicking_one_publishes_its_message() {
        let tokens = motolii_tokens_rs::Tokens::default();
        let items = vec![Item {
            label: "Duplicate",
            shortcut: Some("Cmd+D"),
            message: FakeMessage("duplicate"),
        }];
        let element =
            open_at(text("base"), Point::new(40.0, 60.0), items, tokens.dims, tokens.colors);
        let mut ui = iced_test::simulator(element);
        ui.click("Duplicate").expect("open_at の項目へ selector が届かない");

        let messages: Vec<_> = ui.into_messages().collect();
        assert_eq!(messages, vec![FakeMessage("duplicate")]);
    }
}
