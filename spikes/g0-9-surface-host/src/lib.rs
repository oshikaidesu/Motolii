use thiserror::Error;

pub const LEFT_WEBVIEW_WIDTH: f64 = 240.0;
pub const RIGHT_WEBVIEW_WIDTH: f64 = 260.0;
pub const STAGE_SHARE: f64 = 0.72;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLayout {
    pub logical_width: f64,
    pub logical_height: f64,
    pub scale_factor: f64,
    pub native_x: f32,
    pub native_width: f32,
    pub stage_height: f32,
    pub timeline_y: f32,
    pub timeline_height: f32,
}

impl SurfaceLayout {
    pub fn try_new(physical_width: u32, physical_height: u32, scale_factor: f64) -> Option<Self> {
        if physical_width == 0
            || physical_height == 0
            || !scale_factor.is_finite()
            || scale_factor <= 0.0
        {
            return None;
        }

        let logical_width = f64::from(physical_width) / scale_factor;
        let logical_height = f64::from(physical_height) / scale_factor;
        let native_logical_width = logical_width - LEFT_WEBVIEW_WIDTH - RIGHT_WEBVIEW_WIDTH;
        if native_logical_width <= 1.0 {
            return None;
        }

        let native_x = (LEFT_WEBVIEW_WIDTH * scale_factor) as f32;
        let native_width = (native_logical_width * scale_factor) as f32;
        let stage_height = (f64::from(physical_height) * STAGE_SHARE) as f32;
        let timeline_y = stage_height;
        let timeline_height = physical_height as f32 - stage_height;

        Some(Self {
            logical_width,
            logical_height,
            scale_factor,
            native_x,
            native_width,
            stage_height,
            timeline_y,
            timeline_height,
        })
    }

    pub fn cursor_is_over_webview(self, physical_x: f64) -> bool {
        let left_edge = LEFT_WEBVIEW_WIDTH * self.scale_factor;
        let right_edge = (self.logical_width - RIGHT_WEBVIEW_WIDTH) * self.scale_factor;
        physical_x < left_edge || physical_x >= right_edge
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AcceptanceCounters {
    pub acquire_count: u64,
    pub present_count: u64,
    pub readback_count: u64,
    pub resize_events: u32,
    pub layout_epoch: u64,
    pub native_drag_moves: u32,
    pub native_drag_crossed_webview: bool,
    pub native_drag_released: bool,
    pub web_drag_started: u32,
    pub web_drag_moved: u32,
    pub web_drag_ended: u32,
    pub web_input_events: u32,
}

impl AcceptanceCounters {
    pub fn present_invariant_holds(self) -> bool {
        self.acquire_count > 0
            && self.readback_count == 0
            && self.acquire_count == self.present_count
    }

    pub fn resize_target_passes(self, target: u32) -> bool {
        self.resize_events >= target && self.layout_epoch >= u64::from(target)
    }
}

/// native Stage -> left Web -> native Timeline -> right Webの順で固定し、
/// 両方向とも決定的にwrapさせる。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FocusRole {
    NativeStage,
    WebLeft,
    NativeTimeline,
    WebRight,
}

impl FocusRole {
    const RING: [FocusRole; 4] = [
        FocusRole::NativeStage,
        FocusRole::WebLeft,
        FocusRole::NativeTimeline,
        FocusRole::WebRight,
    ];

    fn ring_index(self) -> usize {
        Self::RING
            .iter()
            .position(|role| *role == self)
            .expect("FocusRole::RING covers every variant")
    }

    pub fn next(self) -> Self {
        Self::RING[(self.ring_index() + 1) % Self::RING.len()]
    }

    pub fn prev(self) -> Self {
        Self::RING[(self.ring_index() + Self::RING.len() - 1) % Self::RING.len()]
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "native-stage" => Some(Self::NativeStage),
            "web-left" => Some(Self::WebLeft),
            "native-timeline" => Some(Self::NativeTimeline),
            "web-right" => Some(Self::WebRight),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum FocusRequestError {
    #[error("stale focus epoch")]
    StaleEpoch,
}

/// Web IPCとOSアクセシビリティActionのどちらから来た明示的focus要求も、
/// 同じ状態機械を通すことでepoch/rejectの扱いを一本化する。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusCoordinator {
    current: FocusRole,
    epoch: u64,
}

impl FocusCoordinator {
    pub fn new(epoch: u64) -> Self {
        Self {
            current: FocusRole::NativeStage,
            epoch,
        }
    }

    pub fn current(self) -> FocusRole {
        self.current
    }

    pub fn epoch(self) -> u64 {
        self.epoch
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    /// 既に起きた事実の報告であり要求ではないため、epochを持たず
    /// rejectもされない。
    pub fn sync_current(&mut self, role: FocusRole) {
        self.current = role;
    }

    pub fn tab_forward(&mut self) -> FocusRole {
        self.current = self.current.next();
        self.current
    }

    pub fn tab_backward(&mut self) -> FocusRole {
        self.current = self.current.prev();
        self.current
    }

    /// stale epochは`current`を変更せずrejectする。
    pub fn request_focus(
        &mut self,
        target: FocusRole,
        epoch: u64,
    ) -> Result<FocusRole, FocusRequestError> {
        if epoch != self.epoch {
            return Err(FocusRequestError::StaleEpoch);
        }
        self.current = target;
        Ok(self.current)
    }
}

/// 実DOMの`compositionstart`/`compositionend`のみから記録し、合成しない。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositionState {
    #[default]
    Idle,
    Composing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Enter,
    Escape,
    Space,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutCounters {
    pub enter: u32,
    pub escape: u32,
    pub space: u32,
}

/// 実DOM compositionが有効な間、Enter/Escape/Spaceはこのsinkに届かせない。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutSink {
    composition: CompositionState,
    counters: ShortcutCounters,
    composition_updates: u32,
}

impl ShortcutSink {
    pub fn set_composition(&mut self, state: CompositionState) {
        self.composition = state;
    }

    pub fn composition(self) -> CompositionState {
        self.composition
    }

    pub fn counters(self) -> ShortcutCounters {
        self.counters
    }

    /// 観測用であり、それ自体はショートカットではない。
    pub fn record_composition_update(&mut self) {
        self.composition_updates += 1;
    }

    pub fn composition_updates(self) -> u32 {
        self.composition_updates
    }

    /// composition中に飲み込まれたか否かを呼び出し側が判別できるよう
    /// bool を返す。
    pub fn observe(&mut self, key: ShortcutKey) -> bool {
        if self.composition == CompositionState::Composing {
            return false;
        }
        match key {
            ShortcutKey::Enter => self.counters.enter += 1,
            ShortcutKey::Escape => self.counters.escape += 1,
            ShortcutKey::Space => self.counters.space += 1,
        }
        true
    }
}

/// opaque child WebViewごとに1つのWeb IPC送信元。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSource {
    Left,
    Right,
}

impl WebSource {
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    pub fn focus_role(self) -> FocusRole {
        match self {
            Self::Left => FocusRole::WebLeft,
            Self::Right => FocusRole::WebRight,
        }
    }

    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// この列挙外は型付きparseエラーとし、部分文字列一致では推測しない。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebMessage {
    DragStart,
    DragMove,
    DragEnd,
    Input,
    Ready,
    /// coordinatorが起こしていないOS focus変化(クリック等)を追従させる。
    FocusIn,
    /// host側の修飾キー推測ではなく、page側スクリプトが中継した実イベント。
    TabForward,
    TabBackward,
    ShortcutEnter,
    ShortcutEscape,
    ShortcutSpace,
    CompositionStart,
    CompositionUpdate,
    CompositionEnd,
    FocusRequest {
        target: FocusRole,
        epoch: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WebMessageError {
    #[error("unknown web message source")]
    UnknownSource,
    #[error("unknown web message kind")]
    UnknownKind,
    #[error("unknown focus-request target")]
    UnknownTarget,
    #[error("malformed focus-request epoch")]
    MalformedEpoch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WebEnvelopeError {
    #[error("unknown web message source")]
    UnknownSource,
    #[error("missing WebView instance epoch")]
    MissingInstanceEpoch,
    #[error("malformed WebView instance epoch")]
    MalformedInstanceEpoch,
    #[error("stale WebView instance epoch")]
    StaleInstanceEpoch,
}

/// WebKit content processの再生成前に滞留したIPCを、message文法へ渡す前に
/// instance epochで拒否する。
pub fn validate_web_envelope(
    raw: &str,
    expected_left_epoch: u64,
    expected_right_epoch: u64,
) -> Result<(WebSource, &str), WebEnvelopeError> {
    let mut fields = raw.splitn(3, ':');
    let source =
        WebSource::from_wire(fields.next().unwrap_or("")).ok_or(WebEnvelopeError::UnknownSource)?;
    let epoch = fields
        .next()
        .ok_or(WebEnvelopeError::MissingInstanceEpoch)?
        .parse::<u64>()
        .map_err(|_| WebEnvelopeError::MalformedInstanceEpoch)?;
    let message = fields
        .next()
        .ok_or(WebEnvelopeError::MissingInstanceEpoch)?;
    let expected = match source {
        WebSource::Left => expected_left_epoch,
        WebSource::Right => expected_right_epoch,
    };
    if epoch != expected {
        return Err(WebEnvelopeError::StaleInstanceEpoch);
    }
    Ok((source, message))
}

pub fn parse_web_message(raw: &str) -> Result<(WebSource, WebMessage), WebMessageError> {
    let mut top = raw.splitn(2, ':');
    let source_raw = top.next().unwrap_or("");
    let rest = top.next().ok_or(WebMessageError::UnknownKind)?;
    let source = WebSource::from_wire(source_raw).ok_or(WebMessageError::UnknownSource)?;

    let message = match rest {
        "drag-start" => WebMessage::DragStart,
        "drag-move" => WebMessage::DragMove,
        "drag-end" => WebMessage::DragEnd,
        "input" => WebMessage::Input,
        "ready" => WebMessage::Ready,
        "focus-in" => WebMessage::FocusIn,
        "tab-forward" => WebMessage::TabForward,
        "tab-backward" => WebMessage::TabBackward,
        "shortcut-enter" => WebMessage::ShortcutEnter,
        "shortcut-escape" => WebMessage::ShortcutEscape,
        "shortcut-space" => WebMessage::ShortcutSpace,
        "composition-start" => WebMessage::CompositionStart,
        "composition-update" => WebMessage::CompositionUpdate,
        "composition-end" => WebMessage::CompositionEnd,
        other => {
            let mut fields = other.splitn(3, ':');
            match (fields.next(), fields.next(), fields.next()) {
                (Some("focus-request"), Some(target_raw), Some(epoch_raw)) => {
                    let target =
                        FocusRole::from_wire(target_raw).ok_or(WebMessageError::UnknownTarget)?;
                    let epoch: u64 = epoch_raw
                        .parse()
                        .map_err(|_| WebMessageError::MalformedEpoch)?;
                    WebMessage::FocusRequest { target, epoch }
                }
                _ => return Err(WebMessageError::UnknownKind),
            }
        }
    };
    Ok((source, message))
}

/// bounded accessibility projectionへ渡す合成サイズ。ノード数はこれらに
/// 連動してはならない。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SemanticCounts {
    pub clip_count: usize,
    pub key_count: usize,
    pub selection_count: usize,
}

/// OSアクセシビリティアダプタは`main`側で配線するため、意図的に
/// `accesskit::NodeId`ではなくこのcrate固有のidにしている。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxNodeId {
    Root,
    Stage,
    StageCanvas,
    Timeline,
    TimelineTracks,
    TimelinePlayhead,
}

/// この crate の公開面が`accesskit`のtoolkit型を直接名指ししないよう
/// 分離したroleの型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxRole {
    Window,
    Pane,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxNode {
    pub id: AxNodeId,
    pub parent: Option<AxNodeId>,
    pub children: &'static [AxNodeId],
    pub role: AxRole,
    pub label: &'static str,
    pub bounds: AxRect,
    /// StageとTimelineのpaneのみがfocus対象で、子ノードは表示専用のため
    /// Focus actionを持たせない。
    pub focusable: bool,
}

const ROOT_CHILDREN: [AxNodeId; 2] = [AxNodeId::Stage, AxNodeId::Timeline];
const STAGE_CHILDREN: [AxNodeId; 1] = [AxNodeId::StageCanvas];
const TIMELINE_CHILDREN: [AxNodeId; 2] = [AxNodeId::TimelineTracks, AxNodeId::TimelinePlayhead];
const LEAF_CHILDREN: [AxNodeId; 0] = [];

pub const STAGE_AX_NODE_COUNT: usize = 2;
pub const TIMELINE_AX_NODE_COUNT: usize = 3;

/// `main`のOSアクセシビリティアダプタも同じモデルを使うため、ここで
/// 証明した上限がそのまま実アダプタ側の上限になる。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccessibilityProjection {
    nodes: [AxNode; 6],
}

impl AccessibilityProjection {
    /// Documentが存在しないため、ノード構成は`semantic`に依存させない。
    pub fn project(_semantic: SemanticCounts, layout: SurfaceLayout) -> Self {
        let scale = layout.scale_factor;
        let stage_bounds = AxRect {
            x0: f64::from(layout.native_x) / scale,
            y0: 0.0,
            x1: f64::from(layout.native_x + layout.native_width) / scale,
            y1: f64::from(layout.stage_height) / scale,
        };
        let timeline_bounds = AxRect {
            x0: f64::from(layout.native_x) / scale,
            y0: f64::from(layout.timeline_y) / scale,
            x1: f64::from(layout.native_x + layout.native_width) / scale,
            y1: f64::from(layout.timeline_y + layout.timeline_height) / scale,
        };
        let root_bounds = AxRect {
            x0: 0.0,
            y0: 0.0,
            x1: layout.logical_width,
            y1: layout.logical_height,
        };

        Self {
            nodes: [
                AxNode {
                    id: AxNodeId::Root,
                    parent: None,
                    children: &ROOT_CHILDREN,
                    role: AxRole::Window,
                    label: "G0-9 surface host",
                    bounds: root_bounds,
                    focusable: false,
                },
                AxNode {
                    id: AxNodeId::Stage,
                    parent: Some(AxNodeId::Root),
                    children: &STAGE_CHILDREN,
                    role: AxRole::Pane,
                    label: "Stage",
                    bounds: stage_bounds,
                    focusable: true,
                },
                AxNode {
                    id: AxNodeId::StageCanvas,
                    parent: Some(AxNodeId::Stage),
                    children: &LEAF_CHILDREN,
                    role: AxRole::Generic,
                    label: "Stage viewport",
                    bounds: stage_bounds,
                    focusable: false,
                },
                AxNode {
                    id: AxNodeId::Timeline,
                    parent: Some(AxNodeId::Root),
                    children: &TIMELINE_CHILDREN,
                    role: AxRole::Pane,
                    label: "Timeline",
                    bounds: timeline_bounds,
                    focusable: true,
                },
                AxNode {
                    id: AxNodeId::TimelineTracks,
                    parent: Some(AxNodeId::Timeline),
                    children: &LEAF_CHILDREN,
                    role: AxRole::Generic,
                    label: "Timeline tracks",
                    bounds: timeline_bounds,
                    focusable: false,
                },
                AxNode {
                    id: AxNodeId::TimelinePlayhead,
                    parent: Some(AxNodeId::Timeline),
                    children: &LEAF_CHILDREN,
                    role: AxRole::Generic,
                    label: "Timeline playhead",
                    bounds: timeline_bounds,
                    focusable: false,
                },
            ],
        }
    }

    pub fn nodes(&self) -> &[AxNode; 6] {
        &self.nodes
    }

    pub fn stage_node_count(&self) -> usize {
        STAGE_AX_NODE_COUNT
    }

    pub fn timeline_node_count(&self) -> usize {
        TIMELINE_AX_NODE_COUNT
    }

    /// WebView系roleはWKWebView自身のaccessibility treeが内容を持つため
    /// 対応するnative AXノードがなく、rootへfocusを報告する。
    pub fn focused(&self, focus: FocusRole) -> AxNodeId {
        match focus {
            FocusRole::NativeStage => AxNodeId::Stage,
            FocusRole::NativeTimeline => AxNodeId::Timeline,
            FocusRole::WebLeft | FocusRole::WebRight => AxNodeId::Root,
        }
    }
}

/// CU-0G03での人手判定用に遷移前後のfocus/IME owner観測を記録する。
/// pass/fail自体はここで判定しない。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LifecycleObservation {
    pub requested_focus: Option<FocusRole>,
    pub requested_ime_owner: Option<FocusRole>,
    pub restored_focus: Option<FocusRole>,
    pub restored_ime_owner: Option<FocusRole>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LifecycleRecorder {
    observation: LifecycleObservation,
}

impl LifecycleRecorder {
    pub fn record_pre_transition(&mut self, focus: FocusRole, ime_owner: Option<FocusRole>) {
        self.observation.requested_focus = Some(focus);
        self.observation.requested_ime_owner = ime_owner;
    }

    pub fn record_restored(&mut self, focus: FocusRole, ime_owner: Option<FocusRole>) {
        self.observation.restored_focus = Some(focus);
        self.observation.restored_ime_owner = ime_owner;
    }

    pub fn observation(self) -> LifecycleObservation {
        self.observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_partitions_one_surface_without_overlap() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert_eq!(layout.logical_width, 1200.0);
        assert_eq!(layout.native_x, 480.0);
        assert_eq!(layout.native_width, 1400.0);
        assert_eq!(layout.stage_height + layout.timeline_height, 1600.0);
        assert_eq!(layout.timeline_y, layout.stage_height);
    }

    #[test]
    fn zero_small_and_invalid_surfaces_are_rejected() {
        assert!(SurfaceLayout::try_new(0, 100, 1.0).is_none());
        assert!(SurfaceLayout::try_new(100, 0, 1.0).is_none());
        assert!(SurfaceLayout::try_new(100, 100, 0.0).is_none());
        assert!(SurfaceLayout::try_new(100, 100, f64::NAN).is_none());
        assert!(SurfaceLayout::try_new(400, 800, 1.0).is_none());
    }

    #[test]
    fn cursor_regions_use_physical_pixels_at_any_scale() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert!(layout.cursor_is_over_webview(479.0));
        assert!(!layout.cursor_is_over_webview(480.0));
        assert!(!layout.cursor_is_over_webview(1879.0));
        assert!(layout.cursor_is_over_webview(1880.0));
    }

    #[test]
    fn acceptance_requires_balanced_present_and_no_readback() {
        let good = AcceptanceCounters {
            acquire_count: 42,
            present_count: 42,
            resize_events: 100,
            layout_epoch: 100,
            ..Default::default()
        };
        assert!(good.present_invariant_holds());
        assert!(good.resize_target_passes(100));

        assert!(!AcceptanceCounters {
            readback_count: 1,
            ..good
        }
        .present_invariant_holds());
        assert!(!AcceptanceCounters {
            present_count: 41,
            ..good
        }
        .present_invariant_holds());
        assert!(!AcceptanceCounters::default().present_invariant_holds());
    }

    #[test]
    fn focus_ring_traverses_native_web_native_forward_and_wraps() {
        let mut coordinator = FocusCoordinator::new(1);
        assert_eq!(coordinator.current(), FocusRole::NativeStage);
        assert_eq!(coordinator.tab_forward(), FocusRole::WebLeft);
        assert_eq!(coordinator.tab_forward(), FocusRole::NativeTimeline);
        assert_eq!(coordinator.tab_forward(), FocusRole::WebRight);
        assert_eq!(coordinator.tab_forward(), FocusRole::NativeStage);
    }

    #[test]
    fn focus_ring_traverses_in_reverse_and_wraps() {
        let mut coordinator = FocusCoordinator::new(1);
        assert_eq!(coordinator.tab_backward(), FocusRole::WebRight);
        assert_eq!(coordinator.tab_backward(), FocusRole::NativeTimeline);
        assert_eq!(coordinator.tab_backward(), FocusRole::WebLeft);
        assert_eq!(coordinator.tab_backward(), FocusRole::NativeStage);
    }

    #[test]
    fn explicit_focus_request_accepts_current_epoch_and_known_target() {
        let mut coordinator = FocusCoordinator::new(7);
        let result = coordinator.request_focus(FocusRole::NativeTimeline, 7);
        assert_eq!(result, Ok(FocusRole::NativeTimeline));
        assert_eq!(coordinator.current(), FocusRole::NativeTimeline);
    }

    #[test]
    fn explicit_focus_request_rejects_stale_epoch_without_mutation() {
        let mut coordinator = FocusCoordinator::new(7);
        let before = coordinator;
        let result = coordinator.request_focus(FocusRole::WebRight, 6);
        assert_eq!(result, Err(FocusRequestError::StaleEpoch));
        assert_eq!(coordinator, before);
    }

    #[test]
    fn bounded_accessibility_projection_is_constant_for_zero_and_huge_inputs() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        let empty = AccessibilityProjection::project(SemanticCounts::default(), layout);
        let huge = AccessibilityProjection::project(
            SemanticCounts {
                clip_count: usize::MAX,
                key_count: usize::MAX,
                selection_count: usize::MAX,
            },
            layout,
        );
        assert_eq!(empty, huge);
        assert_eq!(empty.stage_node_count(), STAGE_AX_NODE_COUNT);
        assert_eq!(empty.timeline_node_count(), TIMELINE_AX_NODE_COUNT);
        assert_eq!(empty.nodes().len(), 6);
    }

    #[test]
    fn accessibility_stage_and_timeline_are_focusable_with_real_bounds() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        let projection = AccessibilityProjection::project(SemanticCounts::default(), layout);
        let stage = projection
            .nodes()
            .iter()
            .find(|node| node.id == AxNodeId::Stage)
            .unwrap();
        let timeline = projection
            .nodes()
            .iter()
            .find(|node| node.id == AxNodeId::Timeline)
            .unwrap();
        assert!(stage.focusable);
        assert!(timeline.focusable);
        assert_ne!(stage.bounds, timeline.bounds);
        assert!(stage.bounds.y1 > stage.bounds.y0);
        assert!(timeline.bounds.y1 > timeline.bounds.y0);

        let leaves = projection
            .nodes()
            .iter()
            .filter(|node| !node.focusable && node.id != AxNodeId::Root);
        assert_eq!(leaves.count(), 3);

        assert_eq!(projection.focused(FocusRole::NativeStage), AxNodeId::Stage);
        assert_eq!(
            projection.focused(FocusRole::NativeTimeline),
            AxNodeId::Timeline
        );
        assert_eq!(projection.focused(FocusRole::WebLeft), AxNodeId::Root);
        assert_eq!(projection.focused(FocusRole::WebRight), AxNodeId::Root);
    }

    #[test]
    fn composition_transitions_are_parsed_and_recorded() {
        assert_eq!(
            parse_web_message("left:composition-start"),
            Ok((WebSource::Left, WebMessage::CompositionStart))
        );
        assert_eq!(
            parse_web_message("left:composition-update"),
            Ok((WebSource::Left, WebMessage::CompositionUpdate))
        );
        assert_eq!(
            parse_web_message("left:composition-end"),
            Ok((WebSource::Left, WebMessage::CompositionEnd))
        );

        let mut sink = ShortcutSink::default();
        sink.set_composition(CompositionState::Composing);
        assert_eq!(sink.composition(), CompositionState::Composing);
        assert_eq!(sink.composition_updates(), 0);
        sink.record_composition_update();
        sink.record_composition_update();
        assert_eq!(sink.composition_updates(), 2);
        sink.set_composition(CompositionState::Idle);
        assert_eq!(sink.composition(), CompositionState::Idle);
    }

    #[test]
    fn dom_relayed_tab_and_shortcut_messages_parse_by_exact_grammar() {
        assert_eq!(
            parse_web_message("left:ready"),
            Ok((WebSource::Left, WebMessage::Ready))
        );
        assert_eq!(
            parse_web_message("left:focus-in"),
            Ok((WebSource::Left, WebMessage::FocusIn))
        );
        assert_eq!(
            parse_web_message("right:tab-forward"),
            Ok((WebSource::Right, WebMessage::TabForward))
        );
        assert_eq!(
            parse_web_message("right:tab-backward"),
            Ok((WebSource::Right, WebMessage::TabBackward))
        );
        assert_eq!(
            parse_web_message("left:shortcut-enter"),
            Ok((WebSource::Left, WebMessage::ShortcutEnter))
        );
        assert_eq!(
            parse_web_message("left:shortcut-escape"),
            Ok((WebSource::Left, WebMessage::ShortcutEscape))
        );
        assert_eq!(
            parse_web_message("left:shortcut-space"),
            Ok((WebSource::Left, WebMessage::ShortcutSpace))
        );
    }

    #[test]
    fn web_focus_right_must_go_through_focus_request_grammar_with_epoch() {
        // 文法に単独の"focus-right"は残していない。WebView間のfocus要求も
        // native側と同じ型付きfocus-request経路のみを通す。
        assert_eq!(
            parse_web_message("left:focus-right"),
            Err(WebMessageError::UnknownKind)
        );
        let (source, message) = parse_web_message("left:focus-request:web-right:3").unwrap();
        assert_eq!(source, WebSource::Left);
        assert_eq!(
            message,
            WebMessage::FocusRequest {
                target: FocusRole::WebRight,
                epoch: 3,
            }
        );
    }

    #[test]
    fn focus_in_sync_updates_current_without_consuming_an_epoch() {
        let mut coordinator = FocusCoordinator::new(9);
        coordinator.sync_current(FocusRole::WebRight);
        assert_eq!(coordinator.current(), FocusRole::WebRight);
        // syncは要求ではないため、staleness判定に使うepochは変更しない。
        assert_eq!(coordinator.epoch(), 9);
    }

    #[test]
    fn shortcut_sink_swallows_keys_while_composing_and_observes_otherwise() {
        let mut sink = ShortcutSink::default();
        sink.set_composition(CompositionState::Composing);
        assert!(!sink.observe(ShortcutKey::Enter));
        assert!(!sink.observe(ShortcutKey::Escape));
        assert!(!sink.observe(ShortcutKey::Space));
        assert_eq!(sink.counters(), ShortcutCounters::default());

        sink.set_composition(CompositionState::Idle);
        assert!(sink.observe(ShortcutKey::Enter));
        assert!(sink.observe(ShortcutKey::Escape));
        assert!(sink.observe(ShortcutKey::Space));
        assert_eq!(
            sink.counters(),
            ShortcutCounters {
                enter: 1,
                escape: 1,
                space: 1,
            }
        );
    }

    #[test]
    fn unknown_message_is_a_typed_rejection() {
        assert_eq!(
            parse_web_message("left:not-a-real-message"),
            Err(WebMessageError::UnknownKind)
        );
        assert_eq!(
            parse_web_message("middle:input"),
            Err(WebMessageError::UnknownSource)
        );
        assert_eq!(parse_web_message("left"), Err(WebMessageError::UnknownKind));
    }

    #[test]
    fn unknown_focus_request_target_is_a_typed_rejection() {
        assert_eq!(
            parse_web_message("left:focus-request:not-a-role:1"),
            Err(WebMessageError::UnknownTarget)
        );
    }

    #[test]
    fn malformed_focus_request_epoch_is_a_typed_rejection() {
        assert_eq!(
            parse_web_message("left:focus-request:web-right:not-a-number"),
            Err(WebMessageError::MalformedEpoch)
        );
    }

    #[test]
    fn stale_focus_request_via_wire_grammar_is_rejected() {
        let (_, message) = parse_web_message("left:focus-request:web-right:5").unwrap();
        let WebMessage::FocusRequest { target, epoch } = message else {
            panic!("expected a focus request");
        };
        let mut coordinator = FocusCoordinator::new(6);
        let before = coordinator;
        assert_eq!(
            coordinator.request_focus(target, epoch),
            Err(FocusRequestError::StaleEpoch)
        );
        assert_eq!(coordinator, before);
    }

    #[test]
    fn web_envelope_accepts_only_the_current_instance_epoch() {
        assert_eq!(
            validate_web_envelope("left:4:ready", 4, 9),
            Ok((WebSource::Left, "ready"))
        );
        assert_eq!(
            validate_web_envelope("right:9:input", 4, 9),
            Ok((WebSource::Right, "input"))
        );
        assert_eq!(
            validate_web_envelope("left:3:ready", 4, 9),
            Err(WebEnvelopeError::StaleInstanceEpoch)
        );
    }

    #[test]
    fn web_envelope_rejects_unknown_or_malformed_fields() {
        assert_eq!(
            validate_web_envelope("middle:1:ready", 1, 1),
            Err(WebEnvelopeError::UnknownSource)
        );
        assert_eq!(
            validate_web_envelope("left:nope:ready", 1, 1),
            Err(WebEnvelopeError::MalformedInstanceEpoch)
        );
        assert_eq!(
            validate_web_envelope("left:1", 1, 1),
            Err(WebEnvelopeError::MissingInstanceEpoch)
        );
    }

    #[test]
    fn lifecycle_recorder_captures_pre_and_restored_owners() {
        let mut recorder = LifecycleRecorder::default();
        assert_eq!(recorder.observation(), LifecycleObservation::default());

        recorder.record_pre_transition(FocusRole::WebLeft, Some(FocusRole::WebLeft));
        recorder.record_restored(FocusRole::NativeStage, None);

        assert_eq!(
            recorder.observation(),
            LifecycleObservation {
                requested_focus: Some(FocusRole::WebLeft),
                requested_ime_owner: Some(FocusRole::WebLeft),
                restored_focus: Some(FocusRole::NativeStage),
                restored_ime_owner: None,
            }
        );
    }
}
