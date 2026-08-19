//! 窓ぜんたいの近道キーの正本 — **表と、そこから外れた理由**を1箇所に置く。
//!
//! 移植元は `crates/motolii-ui/src/timeline_editor/mod.rs` の `egui::Key::` 各所
//! (2026-08-19 iced 近道キー移植レーン)。**この殻の実装は3箇所に分かれている**
//! — 表(この file)は3つともまとめて説明する:
//!
//! - [`additional_window_shortcut`] … 窓ぜんたいが受ける近道のうち、この
//!   移植レーンが `window_input.rs` へ足した1本(Cmd+A = 全選択)。
//!   New/Open/Save は `window_input.rs` 自身の表のまま(このレーンの持ち物
//!   ではない)。
//! - [`playback_shortcut`] … `Space`(play/pause)・`L`(loop)。同じ
//!   `window_input.rs` の、command 修飾**の無い**キーを受ける別アーム
//!   (2026-08-19 再生機構移植レーンで追加。`motolii_ui::blitz_shell::
//!   ShellGateway::toggle_playing` / `toggle_loop` — `UiIntent` を通らない、
//!   `intent.rs` module doc の「ここに無いもの」参照)。
//! - `timeline/canvas.rs` の `Program::update`(既存, M-3) … Undo/Redo・
//!   Esc(gesture 取消)・Delete/Backspace・← / → は**この移植レーンの前から
//!   既に居た**(`drive_timeline.rs` の既存テストが審判している)。canvas
//!   widget はカーソル位置に関わらず窓ぜんたいの keyboard 事象を受けるので
//!   (`iced_test::Simulator` に hover を要らない既存テストが証拠)、実質もう
//!   window-level の近道として機能している。ここでは新しく書かない —
//!   上書きすると並走中の別レーン(Timeline のキー編集 / 構造操作)の編集と
//!   衝突する。
//!
//! [`SHORTCUTS`] が唯一の正本の表。status 帯の legend([`legend_line`])も
//! ここから作るので、表を更新すれば legend も揃って動く。
//!
//! ## ここに置かなかったキー(Q0: 機構が無いものは近道として書かない)
//!
//! - **Cmd+K**(split)・**`M`**(playhead へ locator): `motolii_doc::Command::
//!   SplitClip` / `AddLocator` は D2(`crates/motolii-doc`)に既に在るが、
//!   iced 側の製品状態への唯一の口は `ShellGateway::dispatch(UiIntent)`
//!   (`crates/motolii-ui/src/blitz_shell/intent.rs`)で、`UiIntent` に
//!   `SplitClip` / `AddLocator` は無い。新しい intent を作らない指示なので、
//!   口が無いまま報告する(egui 版 `TimelineEditor::split_selected` /
//!   `tap_locator` は D2 を直接叩けるが、iced 版にその道は無い)。
//! - **Cmd+G / Cmd+Shift+G**(group/ungroup)・**Cmd+D**(複製)・**Enter**
//!   (rename 確定): 担当外(構造操作レーン)。
//! - Delete/Backspace で**キー菱形が選ばれている時**の分岐: 担当外
//!   (キー編集レーン)。この殻はまだキー菱形の選択自体を持たない。

use crate::message::Message;
use crate::timeline::TimelineMsg;

/// 表の1行。`implemented` が `false` の行は [`legend_line`] に出さない
/// (Q0: 効かないキーを提示に書かない)。
pub struct ShortcutRow {
    pub keys: &'static str,
    pub meaning: &'static str,
    pub implemented: bool,
}

/// 近道キーの表そのもの。egui 版 `timeline_editor/mod.rs` の `egui::Key::`
/// 全数に対応する行を持つ(移植した分は `implemented: true`、機構 / 口が
/// 無い分は `false` — 上のモジュール doc に理由がある)。
pub const SHORTCUTS: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Space",
        meaning: "play / pause",
        implemented: true,
    },
    ShortcutRow {
        keys: "L",
        meaning: "loop",
        implemented: true,
    },
    ShortcutRow {
        keys: "Cmd+Z",
        meaning: "undo",
        implemented: true,
    },
    ShortcutRow {
        keys: "Shift+Cmd+Z",
        meaning: "redo",
        implemented: true,
    },
    ShortcutRow {
        keys: "Esc",
        meaning: "cancel gesture",
        implemented: true,
    },
    ShortcutRow {
        keys: "Delete",
        meaning: "delete selection",
        implemented: true,
    },
    ShortcutRow {
        keys: "\u{2190}/\u{2192}",
        meaning: "step 1 frame (Shift = 10)",
        implemented: true,
    },
    ShortcutRow {
        keys: "Cmd+A",
        meaning: "select all",
        implemented: true,
    },
    ShortcutRow {
        keys: "Cmd+K",
        meaning: "split",
        implemented: false,
    },
    ShortcutRow {
        keys: "M",
        meaning: "add marker",
        implemented: false,
    },
];

/// status 帯に出す一行。**実際に効くキーだけ**(Q0: 触れそうで触れない物を
/// 提示に書かない)。
pub fn legend_line() -> String {
    SHORTCUTS
        .iter()
        .filter(|row| row.implemented)
        .map(|row| format!("{}={}", row.keys, row.meaning))
        .collect::<Vec<_>>()
        .join("   ")
}

/// 窓ぜんたいが受ける近道のうち、このレーンが `window_input.rs` へ足した
/// 1本。呼び手(`window_input.rs`)は既に `modifiers.command()` を確かめた
/// 後なので、ここは文字だけを見る(`shortcut()` と同じ流儀)。
pub fn additional_window_shortcut(character: &str) -> Option<Message> {
    if character.eq_ignore_ascii_case("a") {
        Some(Message::Timeline(TimelineMsg::SelectAllPressed))
    } else {
        None
    }
}

/// テスト専用: [`additional_window_shortcut`] を `Key` から直接叩く。
/// `window_input.rs` の呼び出しと同じ形(command 済み前提)で確かめる。
#[cfg(test)]
fn shortcut_for_key(key: &iced::keyboard::Key) -> Option<Message> {
    let iced::keyboard::Key::Character(character) = key else {
        return None;
    };
    additional_window_shortcut(character)
}

/// `Space`(play/pause)・`L`(loop)の生 key → `Message`。
///
/// 呼び手(`window_input.rs`)のアームは command 修飾が付いた事象を先に取るので、
/// ここへ来る時点で Cmd は押されていない — 改めて `modifiers` を見る必要は無い
/// (`additional_window_shortcut` が `shortcut()` の後にだけ呼ばれるのと同じ分担)。
///
/// **`L` に `!command` の除外は要らない** — 上のアームが既に弾いている。egui 版
/// (`timeline_editor/mod.rs`)の `i.key_pressed(egui::Key::L) && !i.modifiers.command`
/// と同じ効きを、iced 側は「2つのアームの順」で作っている。
pub fn playback_shortcut(key: &iced::keyboard::Key) -> Option<Message> {
    match key {
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) => {
            Some(Message::TogglePlayPressed)
        }
        iced::keyboard::Key::Character(character) if character.eq_ignore_ascii_case("l") => {
            Some(Message::ToggleLoopPressed)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::Key;

    #[test]
    fn legend_lists_only_implemented_keys() {
        let line = legend_line();
        assert!(
            line.contains("Cmd+A"),
            "実装済みの Cmd+A が legend に無い: {line}"
        );
        assert!(
            line.contains("Cmd+Z"),
            "実装済みの Cmd+Z が legend に無い: {line}"
        );
        assert!(
            line.contains("Space"),
            "実装済みの Space(play/pause)が legend に無い: {line}"
        );
        assert!(
            line.contains("L=loop"),
            "実装済みの L(loop)が legend に無い: {line}"
        );
        assert!(
            !line.contains("Cmd+K"),
            "口の無い Cmd+K が legend に出た: {line}"
        );
        assert!(
            !line.contains("marker"),
            "口の無い marker が legend に出た: {line}"
        );
    }

    #[test]
    fn select_all_maps_the_a_character() {
        let key = Key::Character("a".into());
        assert_eq!(
            shortcut_for_key(&key),
            Some(Message::Timeline(TimelineMsg::SelectAllPressed))
        );
        // 大文字(Shift+a)でも同じ意味(既存 `shortcut()` の n/o/s と同じ流儀)。
        let key = Key::Character("A".into());
        assert_eq!(
            shortcut_for_key(&key),
            Some(Message::Timeline(TimelineMsg::SelectAllPressed))
        );
    }

    #[test]
    fn unrelated_characters_stay_none() {
        let key = Key::Character("z".into());
        assert_eq!(shortcut_for_key(&key), None);
    }

    #[test]
    fn space_toggles_play() {
        let key = Key::Named(iced::keyboard::key::Named::Space);
        assert_eq!(playback_shortcut(&key), Some(Message::TogglePlayPressed));
    }

    #[test]
    fn l_toggles_loop_case_insensitively() {
        let key = Key::Character("l".into());
        assert_eq!(playback_shortcut(&key), Some(Message::ToggleLoopPressed));
        let key = Key::Character("L".into());
        assert_eq!(playback_shortcut(&key), Some(Message::ToggleLoopPressed));
    }

    #[test]
    fn playback_shortcut_ignores_unrelated_keys() {
        let key = Key::Character("k".into());
        assert_eq!(playback_shortcut(&key), None);
        let key = Key::Named(iced::keyboard::key::Named::Enter);
        assert_eq!(playback_shortcut(&key), None);
    }
}
