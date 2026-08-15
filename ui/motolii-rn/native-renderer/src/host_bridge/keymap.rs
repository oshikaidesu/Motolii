use motolii_ui::{
    AsciiKey, EffectiveTrigger, InputPhase, KeyToken, Modifier, Modifiers,
    PlatformCommandModifier, ProductAction, builtin_command_registry,
    default_user_keymap_override_path, load_user_keymap_override, product_action_host_kind,
    product_builtin_keymap, resolve_product_action,
};

use super::dispatch::try_dispatch_keymap;

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `kind_utf8` must point to `kind_len` UTF-8 bytes naming undo/redo/delete_layer/toggle_playback.
pub unsafe extern "C" fn motolii_rnapp_host_keymap(kind_utf8: *const u8, kind_len: usize) -> bool {
    if kind_utf8.is_null() || kind_len == 0 || kind_len > 64 {
        return false;
    }
    let Ok(kind) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(kind_utf8, kind_len) })
    else {
        return false;
    };
    try_dispatch_keymap(kind).is_some_and(|result| result.accepted)
}

pub(super) const MOD_SHIFT: u32 = 1;
pub(super) const MOD_CONTROL: u32 = 2;
pub(super) const MOD_ALT: u32 = 4;
pub(super) const MOD_META: u32 = 8;

fn mac_key_token(key_code: u16, chars: &str) -> Option<KeyToken> {
    match key_code {
        49 => Some(KeyToken::Space),
        36 => Some(KeyToken::Enter),
        53 => Some(KeyToken::Escape),
        117 => Some(KeyToken::Delete),
        51 => Some(KeyToken::Backspace),
        48 => Some(KeyToken::Tab),
        126 => Some(KeyToken::ArrowUp),
        125 => Some(KeyToken::ArrowDown),
        123 => Some(KeyToken::ArrowLeft),
        124 => Some(KeyToken::ArrowRight),
        115 => Some(KeyToken::Home),
        119 => Some(KeyToken::End),
        116 => Some(KeyToken::PageUp),
        121 => Some(KeyToken::PageDown),
        _ => {
            let value = chars.chars().next()?.to_ascii_lowercase();
            AsciiKey::try_new(value).ok().map(KeyToken::Ascii)
        }
    }
}

fn mac_modifiers(bits: u32) -> Option<Modifiers> {
    let mut modifiers = Vec::new();
    if bits & MOD_SHIFT != 0 {
        modifiers.push(Modifier::Shift);
    }
    if bits & MOD_CONTROL != 0 {
        modifiers.push(Modifier::Control);
    }
    if bits & MOD_ALT != 0 {
        modifiers.push(Modifier::Alt);
    }
    if bits & MOD_META != 0 {
        modifiers.push(Modifier::Meta);
    }
    Modifiers::try_new(modifiers).ok()
}

pub(super) fn resolve_mac_key_action(key_code: u16, modifier_bits: u32, chars: &str) -> Option<ProductAction> {
    let key = mac_key_token(key_code, chars)?;
    let modifiers = mac_modifiers(modifier_bits)?;
    let trigger = EffectiveTrigger::Keyboard {
        key,
        modifiers,
        phase: InputPhase::Press,
    };
    let registry = builtin_command_registry().ok()?;
    let base = product_builtin_keymap();
    let delta = load_user_keymap_override(default_user_keymap_override_path().as_deref(), &base);
    resolve_product_action(&trigger, &registry, &delta, PlatformCommandModifier::Meta)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `chars_utf8` may be null when `chars_len` is 0.
/// 戻り: 0=未束縛, 1=消費, 2=timeline既存deleteへ。
pub unsafe extern "C" fn motolii_rnapp_host_key_event(
    key_code: u16,
    modifier_bits: u32,
    chars_utf8: *const u8,
    chars_len: usize,
    is_repeat: bool,
    timeline_focused: bool,
) -> i32 {
    let chars = if chars_utf8.is_null() || chars_len == 0 {
        ""
    } else if chars_len > 16 {
        return 0;
    } else {
        match std::str::from_utf8(unsafe { std::slice::from_raw_parts(chars_utf8, chars_len) }) {
            Ok(value) => value,
            Err(_) => return 0,
        }
    };
    let Some(action) = resolve_mac_key_action(key_code, modifier_bits, chars) else {
        return 0;
    };
    let Some(kind) = product_action_host_kind(&action) else {
        return i32::from(matches!(action, ProductAction::Unwired(_)));
    };
    if is_repeat
        && matches!(
            kind,
            "toggle_playback"
                | "shuttle_forward"
                | "shuttle_stop"
                | "trim_clip_in"
                | "trim_clip_out"
        )
    {
        return 1;
    }
    if kind == "delete_layer" && timeline_focused {
        return 2;
    }
    i32::from(try_dispatch_keymap(kind).is_some_and(|result| result.accepted))
}
