use dioxus_native::prelude::Key;

/// キー→意図の対応。機構名でなく意図名(裁定174)。
#[derive(Clone, Copy)]
pub enum Intent {
    Split,
    StepFrame(i64),
    Home,
    End,
    Deselect,
    SelectAll,
    PlayPause,
}

#[derive(Clone, Copy, PartialEq)]
enum KeySpec {
    Char(char),
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    Escape,
}

struct Binding {
    key: KeySpec,
    cmd: bool,
    shift: bool,
    intent: Intent,
}

const BINDINGS: &[Binding] = &[
    Binding { key: KeySpec::Char('k'), cmd: true, shift: false, intent: Intent::Split },
    Binding { key: KeySpec::Char('a'), cmd: true, shift: false, intent: Intent::SelectAll },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: false, intent: Intent::StepFrame(-1) },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: true, intent: Intent::StepFrame(-10) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: false, intent: Intent::StepFrame(1) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: true, intent: Intent::StepFrame(10) },
    Binding { key: KeySpec::Home, cmd: false, shift: false, intent: Intent::Home },
    Binding { key: KeySpec::End, cmd: false, shift: false, intent: Intent::End },
    Binding { key: KeySpec::Escape, cmd: false, shift: false, intent: Intent::Deselect },
    Binding { key: KeySpec::Char(' '), cmd: false, shift: false, intent: Intent::PlayPause },
];

/// 表を引いて意図を返す。テキスト編集中に呼ぶかどうかは呼び出し側の責任。
pub fn lookup(key: &Key, cmd: bool, shift: bool) -> Option<Intent> {
    let spec = match key {
        Key::Character(c) if c.len() == 1 => KeySpec::Char(c.chars().next()?.to_ascii_lowercase()),
        Key::ArrowLeft => KeySpec::ArrowLeft,
        Key::ArrowRight => KeySpec::ArrowRight,
        Key::Home => KeySpec::Home,
        Key::End => KeySpec::End,
        Key::Escape => KeySpec::Escape,
        _ => return None,
    };
    BINDINGS
        .iter()
        .find(|b| b.key == spec && b.cmd == cmd && b.shift == shift)
        .map(|b| b.intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_a_is_select_all() {
        assert!(matches!(
            lookup(&Key::Character("a".into()), true, false),
            Some(Intent::SelectAll)
        ));
    }

    #[test]
    fn plain_a_is_not_bound() {
        assert!(lookup(&Key::Character("a".into()), false, false).is_none());
    }

    #[test]
    fn space_is_play_pause() {
        assert!(matches!(
            lookup(&Key::Character(" ".into()), false, false),
            Some(Intent::PlayPause)
        ));
    }

    #[test]
    fn cmd_k_is_split() {
        assert!(matches!(
            lookup(&Key::Character("k".into()), true, false),
            Some(Intent::Split)
        ));
    }

    #[test]
    fn plain_k_is_not_split() {
        assert!(lookup(&Key::Character("k".into()), false, false).is_none());
    }
}
