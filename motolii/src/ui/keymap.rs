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
    /// キーを持つ属性だけに絞る / 戻す。
    ToggleKeyedOnly,
    /// 選んだ層をそのまま増やす。
    Duplicate,
    /// 選んだ区間へイージングを当てる。AE の F9 一族。
    EasyEase(EaseSide),
}

/// 区間のどちら側を寝かせるか。
#[derive(Clone, Copy, PartialEq)]
pub(super) enum EaseSide {
    Both,
    In,
    Out,
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
    F9,
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
    Binding { key: KeySpec::Char('d'), cmd: true, shift: false, alt: false, intent: Intent::Duplicate },
    Binding { key: KeySpec::F9, cmd: false, shift: false, alt: false, intent: Intent::EasyEase(EaseSide::Both) },
    Binding { key: KeySpec::F9, cmd: false, shift: true, alt: false, intent: Intent::EasyEase(EaseSide::In) },
    Binding { key: KeySpec::F9, cmd: true, shift: true, alt: false, intent: Intent::EasyEase(EaseSide::Out) },
    Binding { key: KeySpec::Char('u'), cmd: false, shift: false, alt: false, intent: Intent::ToggleKeyedOnly },
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
        Key::F9 => KeySpec::F9,
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
    fn no_two_bindings_claim_the_same_stroke() {
        for (i, a) in BINDINGS.iter().enumerate() {
            for b in &BINDINGS[i + 1..] {
                assert!(
                    !(a.key == b.key && a.cmd == b.cmd && a.shift == b.shift && a.alt == b.alt),
                    "同じ打鍵に2つの動詞が居る(先に並んだ方しか届かない)"
                );
            }
        }
    }

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
        let primary = if cfg!(target_os = "macos") { Key::Meta } else { Key::Control };
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
}

/// 打鍵が入力欄へ入っているかを窓の側から知らせる。
pub(super) fn set_typing(on: bool) {
    TYPING.with(|t| t.set(on));
}

/// イベントに乗ってきた修飾と、覚えている押し下げを合わせる。
pub(super) fn lookup_held(key: &Key, cmd: bool, shift: bool, alt: bool) -> Option<Intent> {
    // 欄へ打っている間は動詞を引かない。打鍵は入力欄へ入った**あと**根まで
    // 上ってくるので、ここで止めないと名前の空白が再生を始める。
    if TYPING.with(std::cell::Cell::get) {
        return None;
    }
    let (h_cmd, h_shift, h_alt) = held::get();
    lookup(key, cmd || h_cmd, shift || h_shift, alt || h_alt)
}

#[cfg(test)]
mod held_tests {
    use super::*;

    /// 押し下げと打ち込み中は窓ごとに1つ。並べて走らせると互いを踏む。
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 実機では ⌘ が別イベントで来て、続く文字に乗らない。覚えていないと全部素通りする。
    #[test]
    fn a_modifier_that_arrived_as_its_own_event_still_counts() {
        let _held = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        forget_modifiers();
        let d = Key::Character("d".into());
        assert!(lookup_held(&d, false, false, false).is_none(), "素の d が拾われている");

        note_key_down(&if cfg!(target_os = "macos") { Key::Meta } else { Key::Control });
        assert!(lookup_held(&d, false, false, false).is_some(), "覚えた ⌘ が効いていない");

        note_key_up(&if cfg!(target_os = "macos") { Key::Meta } else { Key::Control });
        assert!(lookup_held(&d, false, false, false).is_none(), "離した ⌘ が残っている");
    }

    /// 打鍵は入力欄へ入ったあと根まで上ってくる。ここで止めないと、
    /// 名前に打った空白が再生を始める。
    #[test]
    fn a_stroke_typed_into_a_field_does_not_also_run_a_verb() {
        let _held = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        forget_modifiers();
        let space = Key::Character(" ".into());

        set_typing(true);
        assert!(lookup_held(&space, false, false, false).is_none(), "欄に打った空白が動詞を引いた");

        set_typing(false);
        assert!(lookup_held(&space, false, false, false).is_some(), "欄を閉じたのに動詞が戻らない");
    }
}
