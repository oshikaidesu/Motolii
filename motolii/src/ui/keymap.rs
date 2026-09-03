use dioxus_native::prelude::{Code, Key, Modifiers};

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
    /// 選択した層を1つのGroupへ入れる。
    Group,
    /// 選択したGroupを一段だけ開く。
    Ungroup,
    /// 選んだ区間へイージングを当てる。AE の F9 一族。
    EasyEase(EaseSide),
    /// 仕舞う。Cmd+S。
    Save,
    /// 別名で仕舞う。Shift+Cmd+S。
    SaveAs,
    /// 白紙。Cmd+N。
    NewProject,
    /// 開く。Cmd+O。
    OpenProject,
    /// 選んだ層の名前を開く。Enter(AE・Finder)。
    Rename,
    /// 視点(⌘0 = Fit、⌘1 = 100%、⌘= / ⌘− = 段階)。AE・Figma・Nuke の指。
    View(crate::ui::session::ViewRequest),
    /// 選んだ層を 1px(Shift で 10px)動かす。Alt+矢印 —— 素の矢印は時間の物。
    Nudge(f64, f64),
    /// 終わる(⌘Q)。未保存なら先に訊く。
    Quit,
    /// 枠の設定を開く(⌥⌘K。AE の Composition Settings は ⌘K だが、⌘K は切る手に使っている)。
    CompositionSettings,
    /// AE の P / S / R / T / A: 選んだ層を展開してその属性の行だけ出す。
    Reveal(&'static str),
    /// 選択を伸ばす(Shift+↑↓、Finder・AE)。
    SelectExtend(i32),
}

/// 区間のどちら側を寝かせるか。
#[derive(Clone, Copy, PartialEq)]
pub(super) enum EaseSide {
    Both,
    In,
    Out,
}

pub(super) fn primary_modifier(modifiers: Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.intersects(Modifiers::META | Modifiers::SUPER)
    } else {
        modifiers.ctrl()
    }
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
    Enter,
}

struct Binding {
    key: KeySpec,
    cmd: bool,
    shift: bool,
    alt: bool,
    intent: Intent,
}

const BINDINGS: &[Binding] = &[
    Binding {
        key: KeySpec::Char('k'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Split,
    },
    Binding {
        key: KeySpec::Char('a'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::SelectAll,
    },
    Binding {
        key: KeySpec::ArrowLeft,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::StepFrame(-1),
    },
    Binding {
        key: KeySpec::ArrowLeft,
        cmd: false,
        shift: true,
        alt: false,
        intent: Intent::StepFrame(-10),
    },
    Binding {
        key: KeySpec::ArrowRight,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::StepFrame(1),
    },
    Binding {
        key: KeySpec::ArrowRight,
        cmd: false,
        shift: true,
        alt: false,
        intent: Intent::StepFrame(10),
    },
    Binding {
        key: KeySpec::Home,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::Home,
    },
    Binding {
        key: KeySpec::End,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::End,
    },
    Binding {
        key: KeySpec::Escape,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::Deselect,
    },
    Binding {
        key: KeySpec::Char(' '),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::PlayPause,
    },
    Binding {
        key: KeySpec::Delete,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::DeleteLayer,
    },
    Binding {
        key: KeySpec::Char('z'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Undo,
    },
    Binding {
        key: KeySpec::Char('z'),
        cmd: true,
        shift: true,
        alt: false,
        intent: Intent::Redo,
    },
    Binding {
        key: KeySpec::Char(']'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Reorder(1),
    },
    Binding {
        key: KeySpec::Char('['),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Reorder(-1),
    },
    Binding {
        key: KeySpec::Char('['),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::SnapEdgeToPlayhead(false),
    },
    Binding {
        key: KeySpec::Char(']'),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::SnapEdgeToPlayhead(true),
    },
    Binding {
        key: KeySpec::Char('['),
        cmd: false,
        shift: false,
        alt: true,
        intent: Intent::TrimToPlayhead(false),
    },
    Binding {
        key: KeySpec::Char(']'),
        cmd: false,
        shift: false,
        alt: true,
        intent: Intent::TrimToPlayhead(true),
    },
    Binding {
        key: KeySpec::ArrowUp,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::SelectStep(-1),
    },
    Binding {
        key: KeySpec::Char('d'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Duplicate,
    },
    Binding {
        key: KeySpec::Char('g'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Group,
    },
    Binding {
        key: KeySpec::Char('g'),
        cmd: true,
        shift: true,
        alt: false,
        intent: Intent::Ungroup,
    },
    Binding {
        key: KeySpec::F9,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::EasyEase(EaseSide::Both),
    },
    Binding {
        key: KeySpec::F9,
        cmd: false,
        shift: true,
        alt: false,
        intent: Intent::EasyEase(EaseSide::In),
    },
    Binding {
        key: KeySpec::F9,
        cmd: true,
        shift: true,
        alt: false,
        intent: Intent::EasyEase(EaseSide::Out),
    },
    Binding {
        key: KeySpec::Char('u'),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::ToggleKeyedOnly,
    },
    Binding {
        key: KeySpec::Char('*'),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::ToggleMarker,
    },
    Binding {
        key: KeySpec::Char('*'),
        cmd: false,
        shift: true,
        alt: false,
        intent: Intent::ToggleMarker,
    },
    Binding {
        key: KeySpec::ArrowLeft,
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::JumpMarker(-1),
    },
    Binding {
        key: KeySpec::ArrowRight,
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::JumpMarker(1),
    },
    Binding {
        key: KeySpec::ArrowDown,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::SelectStep(1),
    },
    Binding {
        key: KeySpec::Char('s'),
        cmd: true,
        shift: false,
        alt: false,
        intent: Intent::Save,
    },
    Binding {
        key: KeySpec::Enter,
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::Rename,
    },
    Binding { key: KeySpec::Char('s'), cmd: true, shift: true, alt: false, intent: Intent::SaveAs },
    Binding { key: KeySpec::Char('n'), cmd: true, shift: false, alt: false, intent: Intent::NewProject },
    Binding { key: KeySpec::Char('o'), cmd: true, shift: false, alt: false, intent: Intent::OpenProject },
    Binding { key: KeySpec::Char('='), cmd: true, shift: false, alt: false, intent: Intent::View(crate::ui::session::ViewRequest::Step(1.25)) },
    Binding { key: KeySpec::Char('-'), cmd: true, shift: false, alt: false, intent: Intent::View(crate::ui::session::ViewRequest::Step(0.8)) },
    Binding { key: KeySpec::Char('0'), cmd: true, shift: false, alt: false, intent: Intent::View(crate::ui::session::ViewRequest::Fit) },
    Binding { key: KeySpec::Char('1'), cmd: true, shift: false, alt: false, intent: Intent::View(crate::ui::session::ViewRequest::Actual) },
    Binding { key: KeySpec::Char('q'), cmd: true, shift: false, alt: false, intent: Intent::Quit },
    // AE の指: ⌘⇧D で分割(⌘K も切る)。F9 一族は macOS が食うので ⌘⌥E 一族を並べる。
    Binding { key: KeySpec::Char('d'), cmd: true, shift: true, alt: false, intent: Intent::Split },
    Binding { key: KeySpec::Char('p'), cmd: false, shift: false, alt: false, intent: Intent::Reveal(crate::doc::store::property::POSITION) },
    Binding { key: KeySpec::Char('s'), cmd: false, shift: false, alt: false, intent: Intent::Reveal(crate::doc::store::property::SCALE) },
    Binding { key: KeySpec::Char('r'), cmd: false, shift: false, alt: false, intent: Intent::Reveal(crate::doc::store::property::ROTATION) },
    Binding { key: KeySpec::Char('t'), cmd: false, shift: false, alt: false, intent: Intent::Reveal(crate::doc::store::property::OPACITY) },
    Binding { key: KeySpec::Char('a'), cmd: false, shift: false, alt: false, intent: Intent::Reveal(crate::doc::store::property::ANCHOR) },
    Binding { key: KeySpec::Char('e'), cmd: true, shift: false, alt: true, intent: Intent::EasyEase(EaseSide::Both) },
    Binding { key: KeySpec::Char('e'), cmd: true, shift: true, alt: true, intent: Intent::EasyEase(EaseSide::In) },
    Binding { key: KeySpec::Char('e'), cmd: false, shift: true, alt: true, intent: Intent::EasyEase(EaseSide::Out) },
    // ⌘K は切る(NLE)。枠の設定は ⌥⌘K(AE の ⌘K は取られている)。
    Binding { key: KeySpec::Char('k'), cmd: true, shift: false, alt: true, intent: Intent::CompositionSettings },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: false, alt: true, intent: Intent::Nudge(-1.0, 0.0) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: false, alt: true, intent: Intent::Nudge(1.0, 0.0) },
    Binding { key: KeySpec::ArrowUp, cmd: false, shift: false, alt: true, intent: Intent::Nudge(0.0, -1.0) },
    Binding { key: KeySpec::ArrowDown, cmd: false, shift: false, alt: true, intent: Intent::Nudge(0.0, 1.0) },
    Binding { key: KeySpec::ArrowLeft, cmd: false, shift: true, alt: true, intent: Intent::Nudge(-10.0, 0.0) },
    Binding { key: KeySpec::ArrowRight, cmd: false, shift: true, alt: true, intent: Intent::Nudge(10.0, 0.0) },
    Binding { key: KeySpec::ArrowUp, cmd: false, shift: true, alt: true, intent: Intent::Nudge(0.0, -10.0) },
    Binding { key: KeySpec::ArrowDown, cmd: false, shift: true, alt: true, intent: Intent::Nudge(0.0, 10.0) },
    Binding { key: KeySpec::ArrowUp, cmd: false, shift: true, alt: false, intent: Intent::SelectExtend(-1) },
    Binding { key: KeySpec::ArrowDown, cmd: false, shift: true, alt: false, intent: Intent::SelectExtend(1) },
    // マーカーは NLE の M でも打てる(AE のテンキー `*` と並べる)。
    Binding {
        key: KeySpec::Char('m'),
        cmd: false,
        shift: false,
        alt: false,
        intent: Intent::ToggleMarker,
    },
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
        Key::Enter => KeySpec::Enter,
        _ => return None,
    };
    BINDINGS
        .iter()
        .find(|b| b.key == spec && b.cmd == cmd && b.shift == shift && b.alt == alt)
        .map(|b| b.intent)
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
        Code::KeyA => 'a', Code::KeyB => 'b', Code::KeyC => 'c', Code::KeyD => 'd', Code::KeyE => 'e',
        Code::KeyF => 'f', Code::KeyG => 'g', Code::KeyH => 'h', Code::KeyI => 'i', Code::KeyJ => 'j',
        Code::KeyK => 'k', Code::KeyL => 'l', Code::KeyM => 'm', Code::KeyN => 'n', Code::KeyO => 'o',
        Code::KeyP => 'p', Code::KeyQ => 'q', Code::KeyR => 'r', Code::KeyS => 's', Code::KeyT => 't',
        Code::KeyU => 'u', Code::KeyV => 'v', Code::KeyW => 'w', Code::KeyX => 'x', Code::KeyY => 'y',
        Code::KeyZ => 'z',
        Code::Digit0 => '0', Code::Digit1 => '1', Code::Digit2 => '2', Code::Digit3 => '3', Code::Digit4 => '4',
        Code::Digit5 => '5', Code::Digit6 => '6', Code::Digit7 => '7', Code::Digit8 => '8', Code::Digit9 => '9',
        Code::Minus => '-', Code::Equal => '=',
        _ => return None,
    })
}

/// 今 Shift が押されているか(⇧Tab の判定。winit の KeyEvent は修飾を持たない)。
pub(crate) fn shift_held() -> bool {
    held::get().1
}
