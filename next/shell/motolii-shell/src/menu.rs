//! header のメニューバー(M-menu 設計調査 MB-0+Edit、MB-1、
//! `docs/reviews/2026-08-22-menubar-foundation-survey.md` 準拠)。
//!
//! ## MB-0(基盤 widget)── 実装時に経路を変えた記録
//!
//! 調査 §3.3 案A は `overlay::menu::Menu`(pick_list/combo_box が内部で使う
//! primitive、`iced_widget::overlay::menu`)の wraps を推奨した。実装に入って
//! 確認したところ、`overlay::menu::Overlay`(同モジュール、pick_list/
//! combo_box が共有する overlay 実体)は `iced_core::overlay::Overlay` trait
//! の `operate()` を**オーバーライドしない**(`core/src/overlay.rs` の既定
//! 空実装のまま、実測)── これは `widget::Operation` 走査(`iced_test::
//! Simulator::find`/`collect_targets` が使う唯一の発見経路、
//! `tests/suite/target_walk.rs` 冒頭 doc「`operate()` で明示的に登録した物
//! しか見えない」)がドロップダウンの各項目へ一切届かないことを意味する。
//! 本レーンの ORACLE(「Edit クリック→dropdown 項目が Target で見える→
//! Undo 項目クリックで `Message::Undo` が publish」)がまさにこの発見経路を
//! 要求するため、`overlay::menu::Menu` をそのまま使うと項目がテストから
//! 一切見えない(red のまま詰む)── 調査は cargo build 不可の read-only
//! レーンだったため、この一点(pick_list 系 primitive の `operate()` 欠落)を
//! 検証できていなかった(EVIDENCE_GAP の実地確認、FINDING として作業報告へ
//! 記載済み)。
//!
//! 代わりに、**開いている間だけ普通の `column`(中身は通常の `button`)を
//! `Shell::view` の縦積みへ push する**形にした — `settings_panel_open` が
//! 既に使っている「表示だけの分岐、開いていなければ木に一切現れない」
//! (Q0)と同型。`button` は `operate()` で `Container` candidate を自己登録
//! 済み(`widget/src/button.rs:250` 実測)なので、既存の
//! `ident_band_drive.rs`/`settings_drive.rs` と同じ Simulator の手口(find→
//! bounds→click)でそのまま検分できる。**ボタン直下への絶対配置(真の
//! フローティング overlay)は次波へ持ち越し** — 生の `Overlay` trait を
//! 自前実装するスクラッチ(調査 §3.3 案C、非推奨)は避け、保守最低限
//! (「wraps>移植>スクラッチ」)の範囲に留めた。
//!
//! ## S6 併存
//!
//! 全8項目は既に header ボタンまたは Cmd+キー(`resolve_navigation_key`)を
//! 持つ動詞(調査 §4「メニューは純粋な初回発見用の可視化」)── このメニューは
//! 「唯一の入口」ではなく第3の入口。**新しい `Message` は追加しない** —
//! `Message::Undo`/`Redo`/`CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/
//! `SelectAllLayers`/`DeselectAllLayers` の8本を、右寄せのショートカット表記
//! つきでそのまま `on_press` へぶら下げるだけ(意味の複製ゼロ)。
//!
//! ## MB-1(File 束、裁定176)── File トップレベルが2番目に着地
//!
//! MB-0 が構造だけ予約しないまま見送った File(New Project/Save As/Save a
//! Copy/Quit、当時は rfd 裁定待ちの §6 EVIDENCE_GAP 1)を、裁定176(rfd 採用)
//! を受けてここへ実配線する。**Edit と全く同じ形**(トリガー+開いている間
//! だけ木へ現れる column dropdown)── 新しい見た目・新しい widget 構成を
//! 発明していない。File/Edit の2トップレベルは中身(`Item` 列)以外を共有する
//! ため、`trigger`/`dropdown` を汎用化した(下記)。
//!
//! File の4項目は本切片で**同時に**キーボード shortcut も持つ
//! (`resolve_navigation_key` の Cmd+N/Cmd+Shift+S/Cmd+Q 腕、`lib.rs` 参照)──
//! 発注書「メニューと shortcut を同切片で併設する義務」(M-menu 調査 S6
//! 併存表: 4動詞は着手前は入口ゼロだった)を満たすため、メニュー項目だけを
//! 先に出して shortcut を後回しにしない。
//!
//! ## G1(裁定174「意図優先の原則」、2026-08-22 追加)
//!
//! `GroupLayers`/`UngroupLayers`(⌘G/⌘⇧G)を同じ形でこの Edit メニューへ足した
//! — 独立した "Layer" トップレベル(調査が触れる MB-2)はまだ無いので、Cut/
//! Copy/Duplicate と同じく layer 束の動詞として Edit に同居させる。上と同じ
//! S6 併存(shortcut が正の入口、メニューは第3の発見用入口)、新しい
//! `Message` はやはり増やさない(`Document::group_layers`/`ungroup_layers`
//! doc 参照)。

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};

use crate::chrome::button_style;
use crate::Message;
use motolii_tokens_rs::{Colors, Dimensions};

/// メニュー1項目。**`message` は必須**(Option ではない)── captured→
/// publish 必須(q0 柵、silent 項目ゼロ)を型で保証する。ショートカット表記は
/// プレーン ASCII(`Cmd+Z` 等)── `header()` の歯車ボタン doc が既に避けている
/// 「絵文字/unicode グリフのフォント欠け」(`../reference/KNOWN.md` の
/// letter-spacing 欠けと同種、iced 0.14 系の未確認リスク)を同じ理由で踏まない。
/// `shortcut: None` は「shortcut 出典ゼロ」(normal-map の entries 列)を
/// そのまま表す ── Save a Copy(id 1227)がこれ(下記 `file_items`)。
struct Item {
    label: &'static str,
    shortcut: Option<&'static str>,
    message: Message,
}

/// §1 Edit 構造案(normal-map id 437/435/432/429/430/434/436/433)のうち、
/// S6 併存表(調査 §4)で「既に別入口がある」と判定済みの8項目だけを実配線
/// する。Paste Attributes/Find 等(入口ゼロ)は §6 EVIDENCE_GAP 1 のため出さ
/// ない — Q0「効かない chrome を並べない」に従い、未実装分はメニュー項目
/// 自体を出さない側を採る。
fn edit_items() -> Vec<Item> {
    vec![
        Item { label: "Undo", shortcut: Some("Cmd+Z"), message: Message::Undo }, // id 437
        Item { label: "Redo", shortcut: Some("Cmd+Shift+Z"), message: Message::Redo }, // id 435
        Item { label: "Cut", shortcut: Some("Cmd+X"), message: Message::CutLayer }, // id 432
        Item { label: "Copy", shortcut: Some("Cmd+C"), message: Message::CopyLayer }, // id 429
        Item { label: "Paste", shortcut: Some("Cmd+V"), message: Message::PasteLayer }, // id 430
        Item {
            label: "Duplicate",
            shortcut: Some("Cmd+D"),
            message: Message::DuplicateLayer,
        }, // id 434
        Item {
            label: "Select All",
            shortcut: Some("Cmd+A"),
            message: Message::SelectAllLayers,
        }, // id 436
        Item {
            label: "Deselect All",
            shortcut: Some("Cmd+Shift+A"),
            message: Message::DeselectAllLayers,
        }, // id 433
        // G1(裁定174、normal-map id 無し — H3「Parent column」の置換動詞)。
        Item { label: "Group", shortcut: Some("Cmd+G"), message: Message::GroupLayers },
        Item {
            label: "Ungroup",
            shortcut: Some("Cmd+Shift+G"),
            message: Message::UngroupLayers,
        },
    ]
}

/// MB-1(裁定176)。map 1221/1225/1227/1223 の4項目全部 ── Open は本切片の
/// NON-GOALS(発注書「read 経路の検証が重い」)なので出さない。
fn file_items() -> Vec<Item> {
    vec![
        Item {
            label: "New Project",
            shortcut: Some("Cmd+N"),
            message: Message::NewProjectRequested,
        }, // id 1221
        Item {
            label: "Save As…",
            shortcut: Some("Cmd+Shift+S"),
            message: Message::SaveAsRequested,
        }, // id 1225
        // normal-map entries(menu:shortcut:panel:pref)= 2:0:0:0 ── shortcut
        // 出典ゼロ。他3項目と違い shortcut を発明しない(`Item::shortcut` が
        // `None` を運べる理由そのもの)。
        Item {
            label: "Save a Copy…",
            shortcut: None,
            message: Message::SaveACopyRequested,
        }, // id 1227
        Item { label: "Quit", shortcut: Some("Cmd+Q"), message: Message::QuitRequested }, // id 1223
    ]
}

/// header のトリガーボタン(File/Edit 共通の骨格)。他の header ボタン
/// (Undo/Redo/+Layer/Settings)と同じ `chrome::button_style` — 独自の見た目を
/// 新設しない。
fn trigger(label: &'static str, message: Message, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    button(text(label).size(dims.body_text))
        .style(move |_theme, status| button_style(dims, colors, status))
        .on_press(message)
        .into()
}

/// header の "File" トリガーボタン(MB-1)。
pub fn file_trigger(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    trigger("File", Message::ToggleFileMenu, dims, colors)
}

/// header の "Edit" トリガーボタン。
pub fn edit_trigger(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    trigger("Edit", Message::ToggleEditMenu, dims, colors)
}

/// ドロップダウン本体(File/Edit 共通の骨格。開いている間だけ呼び出し側が
/// 木へ push する、crate 冒頭 doc 参照)。項目=動詞名+ショートカット表記
/// (右寄せ・`text_muted`)── shortcut を持つ項目は全部併記されていること
/// 自体が S6「唯一の入口ゼロ」の構造証明(crate 冒頭 doc)。
fn dropdown(items: Vec<Item>, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    // 算術合成のみ(新トークン新設なし、裁定142)── `inspector_value_width`
    // (64px級)の3倍 = "Deselect All" + "Cmd+Shift+A" が1行に収まる幅。
    // transport_slider_style の `dims.spacing_m * 2.0` と同型の「既存 token
    // への係数」。
    let item_width = dims.inspector_value_width * 3.0;

    let rows: Vec<Element<'static, Message>> = items
        .into_iter()
        .map(|item| {
            let shortcut: Element<'static, Message> = match item.shortcut {
                Some(shortcut) => {
                    text(shortcut).size(dims.caption_text).color(colors.text_muted).into()
                }
                // shortcut 出典ゼロの項目(Save a Copy)は右側を空けるだけ ──
                // 存在しない shortcut を発明しない(`file_items` doc 参照)。
                None => Space::new().into(),
            };
            button(
                row![
                    text(item.label).size(dims.body_text),
                    Space::new().width(Length::Fill),
                    shortcut,
                ]
                .spacing(dims.spacing_m)
                .align_y(iced::alignment::Vertical::Center),
            )
            .width(Length::Fixed(item_width))
            .style(move |_theme, status| button_style(dims, colors, status))
            .on_press(item.message)
            .into()
        })
        .collect();

    container(column(rows))
        .padding(dims.spacing_xs)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// File ドロップダウン本体(MB-1)。
pub fn file_dropdown(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    dropdown(file_items(), dims, colors)
}

/// Edit ドロップダウン本体。
pub fn edit_dropdown(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    dropdown(edit_items(), dims, colors)
}
