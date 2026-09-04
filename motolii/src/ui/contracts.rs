use crate::doc::store::LayerId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Intent {
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
    View(ViewRequest),
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EaseSide {
    Both,
    In,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum KeySpec {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Binding {
    pub key: KeySpec,
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub intent: Intent,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ViewRequest {
    /// 窓に収める(⌘0)。
    Fit,
    /// 画素等倍(⌘1)。
    Actual,
    /// 段階で寄る / 引く(⌘= / ⌘−)。
    Step(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MenuTarget {
    StageLayer(LayerId),
    TimelineLayer(LayerId),
    TimelineKey { layer: LayerId },
    Timeline,
    Stage,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MenuRequest {
    pub x: f64,
    pub y: f64,
    pub target: MenuTarget,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MenuRow {
    pub label: String,
    pub hint: Option<String>,
    pub intent: Intent,
    pub disabled: bool,
}
