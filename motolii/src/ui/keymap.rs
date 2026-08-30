use dioxus_native::prelude::Key;

#[derive(Clone, Copy)]
pub(super) enum Intent {
    Split,
    StepFrame(i64),
    Home,
    End,
    Deselect,
    SelectAll,
    PlayPause,
    DeleteLayer,
    Undo,
    Redo,
    /// 重ね順。正なら前へ、負なら後ろへ。
    Reorder(i16),
    /// 層の頭(false)/尻(true)を現在時刻へ動かす。
    SnapEdgeToPlayhead(bool),
    /// 現在時刻で切り落とす。頭(false)/尻(true)。
    TrimToPlayhead(bool),
    /// 選択を1つ上/下の層へ。
    SelectStep(i32),
    /// 現在時刻のマーカーを打つ / 既に在れば外す。
    ToggleMarker,
    /// 前(-1)/次(+1)のマーカーへ跳ぶ。
    JumpMarker(i32),
}

#[derive(Clone, Copy, PartialEq)]
enum KeySpec {
    Char(char),
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    Escape,
    Delete,
}

struct Binding {
    key: KeySpec,
    cmd: bool,
    shift: bool,
    alt: bool,
    intent: Intent,
}

const BINDINGS: &[Binding] = &[
    Binding { key: KeySpec::Char('k'), cmd: true, shift: false, alt: false, intent: Intent::Split },
    Binding { key: KeySpec::Char('a'), cmd: true, shift: false, alt: false, intent: Intent::SelectAll },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: false, alt: false, intent: Intent::StepFrame(-1) },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: true, alt: false, intent: Intent::StepFrame(-10) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: false, alt: false, intent: Intent::StepFrame(1) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: true, alt: false, intent: Intent::StepFrame(10) },
    Binding { key: KeySpec::Home, cmd: false, shift: false, alt: false, intent: Intent::Home },
    Binding { key: KeySpec::End, cmd: false, shift: false, alt: false, intent: Intent::End },
    Binding { key: KeySpec::Escape, cmd: false, shift: false, alt: false, intent: Intent::Deselect },
    Binding { key: KeySpec::Char(' '), cmd: false, shift: false, alt: false, intent: Intent::PlayPause },
    Binding { key: KeySpec::Delete, cmd: false, shift: false, alt: false, intent: Intent::DeleteLayer },
    Binding { key: KeySpec::Char('z'), cmd: true, shift: false, alt: false, intent: Intent::Undo },
    Binding { key: KeySpec::Char('z'), cmd: true, shift: true, alt: false, intent: Intent::Redo },
    Binding { key: KeySpec::Char(']'), cmd: true, shift: false, alt: false, intent: Intent::Reorder(1) },
    Binding { key: KeySpec::Char('['), cmd: true, shift: false, alt: false, intent: Intent::Reorder(-1) },
    Binding { key: KeySpec::Char('['), cmd: false, shift: false, alt: false, intent: Intent::SnapEdgeToPlayhead(false) },
    Binding { key: KeySpec::Char(']'), cmd: false, shift: false, alt: false, intent: Intent::SnapEdgeToPlayhead(true) },
    Binding { key: KeySpec::Char('['), cmd: false, shift: false, alt: true, intent: Intent::TrimToPlayhead(false) },
    Binding { key: KeySpec::Char(']'), cmd: false, shift: false, alt: true, intent: Intent::TrimToPlayhead(true) },
    Binding { key: KeySpec::ArrowUp, cmd: false, shift: false, alt: false, intent: Intent::SelectStep(-1) },
    Binding { key: KeySpec::Char('*'), cmd: false, shift: false, alt: false, intent: Intent::ToggleMarker },
    Binding { key: KeySpec::Char('*'), cmd: false, shift: true, alt: false, intent: Intent::ToggleMarker },
    Binding { key: KeySpec::ArrowLeft, cmd: true, shift: false, alt: false, intent: Intent::JumpMarker(-1) },
    Binding { key: KeySpec::ArrowRight, cmd: true, shift: false, alt: false, intent: Intent::JumpMarker(1) },
    Binding { key: KeySpec::ArrowDown, cmd: false, shift: false, alt: false, intent: Intent::SelectStep(1) },
];

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
        _ => return None,
    };
    BINDINGS
        .iter()
        .find(|b| b.key == spec && b.cmd == cmd && b.shift == shift && b.alt == alt)
        .map(|b| b.intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_a_is_select_all() {
        assert!(matches!(
            lookup(&Key::Character("a".into()), true, false, false),
            Some(Intent::SelectAll)
        ));
    }

    #[test]
    fn plain_a_is_not_bound() {
        assert!(lookup(&Key::Character("a".into()), false, false, false).is_none());
    }

    #[test]
    fn space_is_play_pause() {
        assert!(matches!(
            lookup(&Key::Character(" ".into()), false, false, false),
            Some(Intent::PlayPause)
        ));
    }

    #[test]
    fn cmd_k_is_split() {
        assert!(matches!(
            lookup(&Key::Character("k".into()), true, false, false),
            Some(Intent::Split)
        ));
    }

    #[test]
    fn plain_k_is_not_split() {
        assert!(lookup(&Key::Character("k".into()), false, false, false).is_none());
    }
}
