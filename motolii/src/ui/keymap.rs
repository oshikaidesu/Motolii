use dioxus_native::prelude::{Code, Key, Modifiers};

use crate::ui::contracts::KeySpec;
pub(super) use crate::ui::contracts::{EaseSide, Intent};
use crate::ui::functions::table::bindings;

pub(super) fn primary_modifier(modifiers: Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.intersects(Modifiers::META | Modifiers::SUPER)
    } else {
        modifiers.ctrl()
    }
}

pub(super) fn lookup(key: &Key, cmd: bool, shift: bool, alt: bool) -> Option<Intent> {
    let spec = match key {
        Key::Character(c) if c.len() == 1 => KeySpec::Char(c.chars().next()?.to_ascii_lowercase()),
        Key::ArrowLeft => KeySpec::ArrowLeft,
        Key::ArrowRight => KeySpec::ArrowRight,
        Key::Home => KeySpec::Home,
        Key::End => KeySpec::End,
        Key::Escape => KeySpec::Escape,
        Key::ArrowUp => KeySpec::ArrowUp,
        Key::ArrowDown => KeySpec::ArrowDown,
        Key::Delete | Key::Backspace => KeySpec::Delete,
        Key::F9 => KeySpec::F9,
        Key::Enter => KeySpec::Enter,
        _ => return None,
    };
    bindings()
        .iter()
        .find(|b| b.key == spec && b.cmd == cmd && b.shift == shift && b.alt == alt)
        .map(|b| b.intent)
}

pub(super) fn hint(intent: Intent) -> Option<String> {
    let binding = bindings().into_iter().find(|b| b.intent == intent)?;
    let key = match binding.key {
        KeySpec::Char(' ') => "Space".to_owned(),
        KeySpec::Char(c) => c.to_ascii_uppercase().to_string(),
        KeySpec::Delete => "⌫".to_owned(),
        KeySpec::Enter => "Enter".to_owned(),
        KeySpec::Escape => "Esc".to_owned(),
        KeySpec::Home => "Home".to_owned(),
        KeySpec::End => "End".to_owned(),
        KeySpec::ArrowLeft => "←".to_owned(),
        KeySpec::ArrowRight => "→".to_owned(),
        KeySpec::ArrowUp => "↑".to_owned(),
        KeySpec::ArrowDown => "↓".to_owned(),
        KeySpec::F9 => "F9".to_owned(),
    };
    let primary = if binding.cmd {
        if cfg!(target_os = "macos") {
            "⌘"
        } else {
            "Ctrl+"
        }
    } else {
        ""
    };
    Some(format!(
        "{}{}{primary}{key}",
        if binding.shift { "⇧" } else { "" },
        if binding.alt { "⌥" } else { "" }
    ))
}

/// 修飾キーは、文字のイベントに乗らずに**別のイベントとして**届く。
/// 押されている物を自分で覚えないと、⌘ を伴う打鍵が全部素通りする。
mod held {
    use keyboard_types::Key;
    use std::sync::atomic::{AtomicU8, Ordering};

    static HELD: AtomicU8 = AtomicU8::new(0);

    const CMD: u8 = 1;
    const SHIFT: u8 = 2;
    const ALT: u8 = 4;

    /// mac は ⌘、それ以外は Ctrl が「主」の修飾。
    fn bit(key: &Key) -> Option<u8> {
        let primary = if cfg!(target_os = "macos") {
            Key::Meta
        } else {
            Key::Control
        };
        match key {
            k if *k == primary => Some(CMD),
            Key::Shift => Some(SHIFT),
            Key::Alt => Some(ALT),
            _ => None,
        }
    }

    pub(crate) fn down(key: &Key) {
        if let Some(b) = bit(key) {
            HELD.fetch_or(b, Ordering::Relaxed);
        }
    }

    pub(crate) fn up(key: &Key) {
        if let Some(b) = bit(key) {
            HELD.fetch_and(!b, Ordering::Relaxed);
        }
    }

    /// 窓から離れると押し下げが取り残されるので、そこで一度捨てる。
    pub(crate) fn clear() {
        HELD.store(0, Ordering::Relaxed);
    }

    pub(crate) fn get() -> (bool, bool, bool) {
        let h = HELD.load(Ordering::Relaxed);
        (h & CMD != 0, h & SHIFT != 0, h & ALT != 0)
    }
}

pub(super) use held::{clear as forget_modifiers, down as note_key_down, up as note_key_up};

// 打ち込み中かは**窓の状態**で、窓は1本の糸の上に居る。大域にすると、
// 並べて走る試験が互いの状態を踏む。
thread_local! {
    static TYPING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static ON_CONTROL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// 焦点が button の上に在るか。在れば Enter / Space はその button の物(macOS・Finder)。
pub(super) fn set_on_control(on: bool) {
    ON_CONTROL.with(|t| t.set(on));
}

pub(super) fn is_on_control() -> bool {
    ON_CONTROL.with(std::cell::Cell::get)
}

/// 打鍵が入力欄へ入っているかを窓の側から知らせる。
pub(super) fn set_typing(on: bool) {
    TYPING.with(|t| t.set(on));
}

pub(super) fn is_typing() -> bool {
    TYPING.with(std::cell::Cell::get)
}

/// イベントに乗ってきた修飾と、覚えている押し下げを合わせる。
pub(super) fn lookup_held(
    key: &Key,
    code: Code,
    cmd: bool,
    shift: bool,
    alt: bool,
) -> Option<Intent> {
    // 欄へ打っている間は動詞を引かない。打鍵は入力欄へ入った**あと**根まで
    // 上ってくるので、ここで止めないと名前の空白が再生を始める。
    if TYPING.with(std::cell::Cell::get) {
        return None;
    }
    let (h_cmd, h_shift, h_alt) = held::get();
    let effective_cmd = cmd || h_cmd;
    let effective_shift = shift || h_shift;
    let effective_alt = alt || h_alt;
    // ⌥ を足すと macOS は文字を変える(⌥E = ´、⌥K = ˚)。alt が効いている間は物理 code から文字を引く。
    let physical = match code {
        Code::BracketLeft => Some(Key::Character("[".into())),
        Code::BracketRight => Some(Key::Character("]".into())),
        _ if effective_alt => code_to_char(code).map(|c| Key::Character(c.to_string())),
        _ => None,
    };
    lookup(
        physical.as_ref().unwrap_or(key),
        effective_cmd,
        effective_shift,
        effective_alt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_g_groups_and_shift_primary_g_ungroups() {
        let key = Key::Character("g".into());
        assert!(matches!(
            lookup(&key, true, false, false),
            Some(Intent::Group)
        ));
        assert!(matches!(
            lookup(&key, true, true, false),
            Some(Intent::Ungroup)
        ));
    }

    #[test]
    fn platform_primary_accepts_the_upstream_modifier_bits() {
        if cfg!(target_os = "macos") {
            assert!(primary_modifier(Modifiers::META));
            assert!(primary_modifier(Modifiers::SUPER));
            assert!(!primary_modifier(Modifiers::CONTROL));
        } else {
            assert!(primary_modifier(Modifiers::CONTROL));
            assert!(!primary_modifier(Modifiers::META | Modifiers::SUPER));
        }
    }

    #[test]
    fn mac_option_bracket_uses_physical_code_when_the_logical_character_changes() {
        assert!(matches!(
            lookup_held(
                &Key::Character("“".into()),
                Code::BracketLeft,
                false,
                false,
                true,
            ),
            Some(Intent::TrimToPlayhead(false))
        ));
    }
}

/// 物理 code → 文字(US 配列の素の字)。⌥ 付きの binding を救う為の表。
pub(super) fn code_to_char(code: Code) -> Option<char> {
    Some(match code {
        Code::KeyA => 'a',
        Code::KeyB => 'b',
        Code::KeyC => 'c',
        Code::KeyD => 'd',
        Code::KeyE => 'e',
        Code::KeyF => 'f',
        Code::KeyG => 'g',
        Code::KeyH => 'h',
        Code::KeyI => 'i',
        Code::KeyJ => 'j',
        Code::KeyK => 'k',
        Code::KeyL => 'l',
        Code::KeyM => 'm',
        Code::KeyN => 'n',
        Code::KeyO => 'o',
        Code::KeyP => 'p',
        Code::KeyQ => 'q',
        Code::KeyR => 'r',
        Code::KeyS => 's',
        Code::KeyT => 't',
        Code::KeyU => 'u',
        Code::KeyV => 'v',
        Code::KeyW => 'w',
        Code::KeyX => 'x',
        Code::KeyY => 'y',
        Code::KeyZ => 'z',
        Code::Digit0 => '0',
        Code::Digit1 => '1',
        Code::Digit2 => '2',
        Code::Digit3 => '3',
        Code::Digit4 => '4',
        Code::Digit5 => '5',
        Code::Digit6 => '6',
        Code::Digit7 => '7',
        Code::Digit8 => '8',
        Code::Digit9 => '9',
        Code::Minus => '-',
        Code::Equal => '=',
        _ => return None,
    })
}

/// 今 Shift が押されているか(⇧Tab の判定。winit の KeyEvent は修飾を持たない)。
pub(crate) fn shift_held() -> bool {
    held::get().1
}
