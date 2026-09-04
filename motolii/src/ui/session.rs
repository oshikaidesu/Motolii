use std::sync::{Arc, Mutex};

use crate::doc::store::{Document, LayerId, Revision};

use crate::ui::playback::Clock;
use crate::ui::timeline_widget::TimelineMsg;
use crate::ui::tokens::UiScale;

#[derive(Clone, Default)]
pub(super) struct GestureSurface {
    active: Arc<std::sync::atomic::AtomicBool>,
    cancel: Arc<std::sync::atomic::AtomicU32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CustomSurface {
    Stage,
    Timeline,
    Ease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CapturePhase {
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy)]
struct CaptureOwner {
    surface: CustomSurface,
    id: blitz_traits::events::BlitzPointerId,
    is_primary: bool,
    origin: [f32; 2],
}

#[derive(Clone, Copy)]
struct DirectPointer {
    surface: CustomSurface,
    phase: CapturePhase,
    id: blitz_traits::events::BlitzPointerId,
    is_primary: bool,
    client: [f32; 2],
}

#[derive(Clone, Copy)]
pub(super) struct CapturedPointer {
    phase: CapturePhase,
    id: blitz_traits::events::BlitzPointerId,
    is_primary: bool,
    client: [f32; 2],
    element: [f32; 2],
    primary_held: bool,
    mods: keyboard_types::Modifiers,
}

impl CapturedPointer {
    pub(super) fn event(self) -> blitz_traits::events::UiEvent {
        let pointer = blitz_traits::events::BlitzPointerEvent {
            id: self.id,
            is_primary: self.is_primary,
            coords: blitz_traits::events::PointerCoords {
                page_x: self.client[0],
                page_y: self.client[1],
                screen_x: self.client[0],
                screen_y: self.client[1],
                client_x: self.client[0],
                client_y: self.client[1],
            },
            button: blitz_traits::events::MouseEventButton::Main,
            buttons: if self.primary_held {
                blitz_traits::events::MouseEventButtons::Primary
            } else {
                blitz_traits::events::MouseEventButtons::None
            },
            mods: self.mods,
            details: Default::default(),
            element: blitz_traits::events::Point {
                x: self.element[0],
                y: self.element[1],
            },
            active_pointers: Default::default(),
        };
        match self.phase {
            CapturePhase::Move => blitz_traits::events::UiEvent::PointerMove(pointer),
            CapturePhase::Up => blitz_traits::events::UiEvent::PointerUp(pointer),
            CapturePhase::Cancel => blitz_traits::events::UiEvent::PointerCancel(pointer),
        }
    }
}

#[derive(Default)]
struct SurfaceCaptureState {
    owner: Option<CaptureOwner>,
    last_direct: Option<DirectPointer>,
    pending: std::collections::VecDeque<(CustomSurface, CapturedPointer)>,
}

/// Custom widgets do not receive move/up after the pointer crosses into DOM chrome.
/// This is the one window-local capture relay until Blitz exposes pointer capture.
#[derive(Clone, Default)]
pub(super) struct SurfaceCapture(Arc<Mutex<SurfaceCaptureState>>);

impl SurfaceCapture {
    pub(super) fn begin(
        &self,
        surface: CustomSurface,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) -> bool {
        if !pointer.is_primary {
            return false;
        }
        let mut state = self.0.lock().unwrap();
        if state.owner.is_some_and(|owner| {
            owner.surface != surface || owner.id != pointer.id || !owner.is_primary
        }) {
            return false;
        }
        state.owner = Some(CaptureOwner {
            surface,
            id: pointer.id,
            is_primary: pointer.is_primary,
            origin: [
                pointer.client_x() - pointer.element.x,
                pointer.client_y() - pointer.element.y,
            ],
        });
        true
    }

    pub(super) fn owns(
        &self,
        surface: CustomSurface,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) -> bool {
        self.0.lock().unwrap().owner.is_some_and(|owner| {
            owner.surface == surface
                && owner.id == pointer.id
                && owner.is_primary == pointer.is_primary
        })
    }

    pub(super) fn blocks(
        &self,
        surface: CustomSurface,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) -> bool {
        self.0.lock().unwrap().owner.is_some_and(|owner| {
            owner.surface != surface
                || owner.id != pointer.id
                || owner.is_primary != pointer.is_primary
        })
    }

    pub(super) fn finish(
        &self,
        surface: CustomSurface,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) {
        let mut state = self.0.lock().unwrap();
        if state.owner.is_some_and(|owner| {
            owner.surface == surface
                && owner.id == pointer.id
                && owner.is_primary == pointer.is_primary
        }) {
            state.owner = None;
        }
        state.last_direct = None;
    }

    pub(super) fn note_direct(
        &self,
        surface: CustomSurface,
        phase: CapturePhase,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) {
        let mut state = self.0.lock().unwrap();
        if state.owner.is_some_and(|owner| {
            owner.surface == surface
                && owner.id == pointer.id
                && owner.is_primary == pointer.is_primary
        }) {
            state.last_direct = Some(DirectPointer {
                surface,
                phase,
                id: pointer.id,
                is_primary: pointer.is_primary,
                client: [pointer.client_x(), pointer.client_y()],
            });
        }
    }

    pub(super) fn cancel_all(&self) {
        let mut state = self.0.lock().unwrap();
        state.owner = None;
        state.last_direct = None;
        state.pending.clear();
    }

    pub(super) fn cancel(&self, surface: CustomSurface) {
        let mut state = self.0.lock().unwrap();
        if state.owner.is_some_and(|owner| owner.surface == surface) {
            state.owner = None;
        }
        if state
            .last_direct
            .is_some_and(|pointer| pointer.surface == surface)
        {
            state.last_direct = None;
        }
        state.pending.retain(|(target, _)| *target != surface);
    }

    fn dom_identity_matches(owner: CaptureOwner, pointer_type: &str, pointer_id: i32) -> bool {
        match owner.id {
            blitz_traits::events::BlitzPointerId::Mouse => {
                pointer_type == "mouse" && pointer_id == 0
            }
            blitz_traits::events::BlitzPointerId::Pen => pointer_type == "pen" && pointer_id == 0,
            blitz_traits::events::BlitzPointerId::Finger(id) => {
                pointer_type == "touch" && pointer_id == id as i32
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn relay_dom(
        &self,
        phase: CapturePhase,
        pointer_type: &str,
        pointer_id: i32,
        is_primary: bool,
        client: [f64; 2],
        primary_held: bool,
        mods: keyboard_types::Modifiers,
    ) -> bool {
        let mut state = self.0.lock().unwrap();
        let Some(owner) = state.owner else {
            return false;
        };
        if owner.is_primary != is_primary
            || !Self::dom_identity_matches(owner, pointer_type, pointer_id)
        {
            return false;
        }
        let client = [client[0] as f32, client[1] as f32];
        if state.last_direct.is_some_and(|direct| {
            direct.surface == owner.surface
                && direct.phase == phase
                && direct.id == owner.id
                && direct.is_primary == is_primary
                && direct.client == client
        }) {
            state.last_direct = None;
            return false;
        }
        state.last_direct = None;
        state.pending.push_back((
            owner.surface,
            CapturedPointer {
                phase,
                id: owner.id,
                is_primary,
                client,
                element: [client[0] - owner.origin[0], client[1] - owner.origin[1]],
                primary_held,
                mods,
            },
        ));
        if !matches!(phase, CapturePhase::Move) {
            state.owner = None;
        }
        true
    }

    pub(super) fn relay_from_other_surface(
        &self,
        current: CustomSurface,
        phase: CapturePhase,
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) -> bool {
        let mut state = self.0.lock().unwrap();
        let Some(owner) = state.owner else {
            return false;
        };
        if owner.surface == current
            || owner.id != pointer.id
            || owner.is_primary != pointer.is_primary
        {
            return false;
        }
        state.pending.push_back((
            owner.surface,
            CapturedPointer {
                phase,
                id: pointer.id,
                is_primary: pointer.is_primary,
                client: [pointer.client_x(), pointer.client_y()],
                element: [
                    pointer.client_x() - owner.origin[0],
                    pointer.client_y() - owner.origin[1],
                ],
                primary_held: !pointer.buttons.is_empty(),
                mods: pointer.mods,
            },
        ));
        if !matches!(phase, CapturePhase::Move) {
            state.owner = None;
        }
        true
    }

    pub(super) fn take(&self, surface: CustomSurface) -> Vec<CapturedPointer> {
        let mut state = self.0.lock().unwrap();
        let mut taken = Vec::new();
        let mut keep = std::collections::VecDeque::new();
        while let Some((target, pointer)) = state.pending.pop_front() {
            if target == surface {
                taken.push(pointer);
            } else {
                keep.push_back((target, pointer));
            }
        }
        state.pending = keep;
        taken
    }

    pub(super) fn has_pending(&self, surface: CustomSurface) -> bool {
        self.0
            .lock()
            .unwrap()
            .pending
            .iter()
            .any(|(target, _)| *target == surface)
    }
}

#[derive(Clone, Default)]
pub(super) struct FileDropSurface(Arc<Mutex<Vec<std::path::PathBuf>>>);

impl FileDropSurface {
    pub(super) fn enter(&self, paths: &[std::path::PathBuf]) {
        *self.0.lock().unwrap() = paths.to_vec();
    }

    pub(super) fn leave(&self) {
        self.0.lock().unwrap().clear();
    }

    pub(super) fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    pub(super) fn paths(&self) -> Vec<std::path::PathBuf> {
        self.0.lock().unwrap().clone()
    }
}

impl GestureSurface {
    pub(super) fn begin(&self) {
        self.active
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub(super) fn end(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(super) fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(super) fn cancel(&self) -> bool {
        if !self
            .active
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            return false;
        }
        self.cancel
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        true
    }

    pub(super) fn cancelled(&self, seen: &mut u32) -> bool {
        let current = self.cancel.load(std::sync::atomic::Ordering::Relaxed);
        if current == *seen {
            return false;
        }
        *seen = current;
        true
    }
}

#[cfg(test)]
mod gesture_tests {
    use super::{FileDropSurface, GestureSurface};

    #[test]
    fn one_cancel_generation_reaches_every_surface_once() {
        let gesture = GestureSurface::default();
        let mut stage = 0;
        let mut timeline = 0;
        let mut ease = 0;

        gesture.begin();
        assert!(gesture.cancel());
        assert!(!gesture.is_active());
        assert!(gesture.cancelled(&mut stage));
        assert!(gesture.cancelled(&mut timeline));
        assert!(gesture.cancelled(&mut ease));
        assert!(!gesture.cancelled(&mut stage));
        assert!(!gesture.cancel());
    }

    #[test]
    fn file_drop_hover_is_one_shared_lifecycle() {
        let drop = FileDropSurface::default();
        drop.enter(&["a.mov".into(), "b.wav".into()]);
        assert_eq!(drop.count(), 2);
        drop.leave();
        assert_eq!(drop.count(), 0);
    }
}

pub(super) fn edit_rejection(
    view: &crate::doc::store::StoreView<'_>,
    layer: LayerId,
) -> Option<&'static str> {
    crate::ui::functions::lens::edit_rejection(view, layer)
        .unwrap_or(Some("layer could not be read"))
}

#[derive(Default)]
struct SelectionState {
    layers: Vec<LayerId>,
    active: Option<LayerId>,
    anchor: Option<LayerId>,
    keys: Option<std::sync::Weak<Mutex<Vec<KeySel>>>>,
}

#[derive(Clone, Default)]
pub(super) struct Selection(Arc<Mutex<SelectionState>>);

impl Selection {
    fn with_keys(keys: &Arc<Mutex<Vec<KeySel>>>) -> Self {
        Self(Arc::new(Mutex::new(SelectionState {
            keys: Some(Arc::downgrade(keys)),
            ..Default::default()
        })))
    }

    fn clear_keys_in(state: &SelectionState) {
        if let Some(keys) = state.keys.as_ref().and_then(std::sync::Weak::upgrade) {
            keys.lock().unwrap().clear();
        }
    }

    pub(super) fn clear_keys(&self) {
        Self::clear_keys_in(&self.0.lock().unwrap());
    }

    pub(super) fn keys(&self) -> Vec<KeySel> {
        self.0
            .lock()
            .unwrap()
            .keys
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .map(|keys| keys.lock().unwrap().clone())
            .unwrap_or_default()
    }

    pub(super) fn restore_keys(&self, values: Vec<KeySel>) {
        if let Some(keys) = self
            .0
            .lock()
            .unwrap()
            .keys
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
        {
            *keys.lock().unwrap() = values;
        }
    }

    pub(super) fn clear(&self) {
        let mut state = self.0.lock().unwrap();
        Self::clear_keys_in(&state);
        state.layers.clear();
        state.active = None;
        state.anchor = None;
    }

    pub(super) fn get(&self) -> Option<LayerId> {
        self.active()
    }

    pub(super) fn active(&self) -> Option<LayerId> {
        self.0.lock().unwrap().active
    }

    pub(super) fn anchor(&self) -> Option<LayerId> {
        self.0.lock().unwrap().anchor
    }

    pub(super) fn set(&self, layer: Option<LayerId>) {
        let mut state = self.0.lock().unwrap();
        Self::clear_keys_in(&state);
        state.layers.clear();
        if let Some(l) = layer {
            state.layers.push(l);
        }
        state.active = layer;
        state.anchor = layer;
    }

    /// Make one member the active/anchor end without discarding the group.
    pub(super) fn activate(&self, layer: LayerId) {
        let mut state = self.0.lock().unwrap();
        if !state.layers.contains(&layer) {
            return;
        }
        Self::clear_keys_in(&state);
        state.active = Some(layer);
        state.anchor = Some(layer);
    }

    pub(super) fn all(&self) -> Vec<LayerId> {
        self.0.lock().unwrap().layers.clone()
    }

    pub(super) fn targets(&self, clicked: Option<LayerId>) -> Vec<LayerId> {
        let selected = self.all();
        match clicked {
            Some(layer) if !selected.contains(&layer) => vec![layer],
            _ => selected,
        }
    }

    pub(super) fn replace(&self, layers: impl IntoIterator<Item = LayerId>) {
        self.replace_impl(layers, false);
    }

    pub(super) fn replace_preserving_keys(&self, layers: impl IntoIterator<Item = LayerId>) {
        self.replace_impl(layers, true);
    }

    fn replace_impl(&self, layers: impl IntoIterator<Item = LayerId>, preserve_keys: bool) {
        let mut state = self.0.lock().unwrap();
        if !preserve_keys {
            Self::clear_keys_in(&state);
        }
        state.layers.clear();
        for layer in layers {
            if !state.layers.contains(&layer) {
                state.layers.push(layer);
            }
        }
        state.anchor = state.layers.first().copied();
        state.active = state.layers.last().copied();
    }

    pub(super) fn contains(&self, layer: LayerId) -> bool {
        self.0.lock().unwrap().layers.contains(&layer)
    }

    pub(super) fn toggle(&self, layer: LayerId) {
        self.toggle_impl(layer, false);
    }

    pub(super) fn toggle_preserving_keys(&self, layer: LayerId) {
        self.toggle_impl(layer, true);
    }

    fn toggle_impl(&self, layer: LayerId, preserve_keys: bool) {
        let mut state = self.0.lock().unwrap();
        if !preserve_keys {
            Self::clear_keys_in(&state);
        }
        match state.layers.iter().position(|l| *l == layer) {
            Some(i) => {
                state.layers.remove(i);
                if state.layers.is_empty() {
                    state.active = None;
                    state.anchor = None;
                } else {
                    if state.active == Some(layer) {
                        state.active = state.layers.last().copied();
                    }
                    if state.anchor == Some(layer) {
                        state.anchor = state.active;
                    }
                }
            }
            None => {
                state.layers.push(layer);
                state.active = Some(layer);
                state.anchor.get_or_insert(layer);
            }
        }
    }

    /// Shift selection. The anchor stays fixed while active can cross it, so the
    /// contiguous range grows, shrinks and reverses like a browser list selection.
    pub(super) fn extend_to(&self, ordered: &[LayerId], target: LayerId) {
        self.extend_to_impl(ordered, target, false);
    }

    fn extend_to_impl(&self, ordered: &[LayerId], target: LayerId, preserve_keys: bool) {
        let mut state = self.0.lock().unwrap();
        if !preserve_keys {
            Self::clear_keys_in(&state);
        }
        let anchor = state
            .anchor
            .filter(|layer| ordered.contains(layer))
            .or_else(|| state.active.filter(|layer| ordered.contains(layer)))
            .unwrap_or(target);
        let Some(anchor_index) = ordered.iter().position(|layer| *layer == anchor) else {
            return;
        };
        let Some(target_index) = ordered.iter().position(|layer| *layer == target) else {
            return;
        };
        let (start, end) = if anchor_index <= target_index {
            (anchor_index, target_index)
        } else {
            (target_index, anchor_index)
        };
        state.layers = ordered[start..=end].to_vec();
        state.anchor = Some(anchor);
        state.active = Some(target);
    }

    /// Move the active end through an ordered list. With `extend`, retain the
    /// anchor and replace the selection by the contiguous interval.
    pub(super) fn step(&self, ordered: &[LayerId], delta: i32, extend: bool) -> Option<LayerId> {
        if ordered.is_empty() {
            return None;
        }
        let active = self.active();
        let target = match active.and_then(|layer| ordered.iter().position(|item| *item == layer)) {
            Some(index) => {
                let next = (index as i32 + delta).clamp(0, ordered.len() as i32 - 1) as usize;
                ordered[next]
            }
            None if delta < 0 => *ordered.last().expect("non-empty order"),
            None => ordered[0],
        };
        if extend {
            self.extend_to(ordered, target);
        } else {
            self.set(Some(target));
        }
        Some(target)
    }
}

#[derive(Clone)]
pub(super) struct Session {
    pub doc: Arc<Mutex<Document>>,
    pub clock: Arc<Clock>,
    pub scale: Arc<UiScale>,
    pub selection: Selection,
    /// 選択中の層の箱の大きさ。Stage が毎フレーム書き、ユーティリティが読む
    /// (箱は engine が形/文字から測るので、Document だけでは出せない)。
    pub selected_size: Arc<Mutex<Option<[f32; 2]>>>,
    /// Stage のギズモが 3D(向きと奥行き)を掴む側に居るか。
    /// タイムラインの盤面へ積む口。盤面は置き場を移すと作り直されるので、
    /// 口は窓の側で持つ。
    pub timeline_tx: std::sync::mpsc::Sender<TimelineMsg>,
    pub timeline_rx: std::rc::Rc<std::sync::mpsc::Receiver<TimelineMsg>>,
    /// 起動時に読んだ素材と見出し。動かないので窓が何枚でも1つ。
    pub ui: Arc<crate::ui::fixture::UiData>,
    /// 曲線を手で範囲の外へ出してよいか。既定は OFF(AM-KG-07)。
    /// 型そのものが行き過ぎる物(Elastic 系)は型の意味として ON になる。
    /// 向きの輪と奥行きの点を描くか。掴んだ所の意味は変えない、散らかりの加減だけ。
    pub rings: Arc<std::sync::atomic::AtomicBool>,
    /// Stage を「出す物だけ」で映す(取っ手も枠も無し)。View menu の Output only。
    pub output_only: Arc<std::sync::atomic::AtomicBool>,
    /// 枠の外へかける膜の濃さ(%)。見る側の設定で、作品には入らない。
    pub frame_dim: Arc<std::sync::atomic::AtomicU32>,
    pub gesture: GestureSurface,
    pub surface_capture: SurfaceCapture,
    pub file_drop: FileDropSurface,
    pub overshoot: Arc<std::sync::atomic::AtomicBool>,
    pub export: crate::ui::output::ExportController,
    pub project_notice: Arc<Mutex<String>>,
    /// 終わる注文(⌘Q / File ▸ Quit)。未保存の確認が通ったら立ち、窓の糸が拾って終わる。
    pub quit: Arc<std::sync::atomic::AtomicBool>,
    /// 別の糸で指紋を取り終えた取り込み。窓の糸が echo の度に拾って棚へ入れる。
    pub imports: Arc<Mutex<Vec<Vec<crate::ui::fixture::Prepared>>>>,
    /// Browser の候補選択。panel の置き場を変えても同じ候補集合を指す。
    pub browser_selection: Arc<Mutex<crate::ui::browser_selection::BrowserSelection>>,
    /// 今の作品の仕舞い先。`Save` が問い直さないために覚える。
    pub project_path: Arc<Mutex<Option<std::path::PathBuf>>>,
    /// 最後に保存／読込／NewしたDocument revision。dirtyは現在との差だけで決まる。
    pub saved_revision: Arc<Mutex<Revision>>,
    pub curve_clip: Arc<Mutex<Option<crate::doc::store::Interp>>>,
    /// 見る側のカメラ(User View)。**Document には入らない** — 書き出しには出ない。
    pub view_camera: Arc<Mutex<crate::render::engine::ObservationCamera>>,
    /// 視点への注文(Fit / 100% / 段階)。Stage が次の描画で取り込む —— 100% は
    /// 窓に収める倍率を知っている Stage にしか解けない。
    pub view_request: Arc<Mutex<Option<ViewRequest>>>,
    /// 今どのキーを掴んでいるか。イージングを触る口が要る(Document には入らない)。
    pub selected_keys: Arc<Mutex<Vec<KeySel>>>,
    /// 今どの値に手が触れているか。Inspector が行を光らせて書き、机が覗く。
    /// 机を呼ぶ口ではない — 机は Document とこれを読むだけ。
    pub focus: Arc<Mutex<Option<Focus>>>,
    /// 机の引き出しの開閉。窓をまたいで 1 つ。
    pub desk: Arc<Mutex<DeskState>>,
    pub field: Arc<Mutex<Option<OpenField>>>,
    /// 仕舞っている最中(dialog を待つ間)。Cmd+S の連打で 2 枚開けない。
    pub saving: Arc<std::sync::atomic::AtomicBool>,
    /// 数値を擦っている最中。窓の外で放しても、Escape でも、ここから終える。
    pub scrub: Arc<Mutex<Option<crate::ui::inspector::ValueDrag>>>,
    /// 面が「この panel を前に出して」と頼む口。app が revision ごとに拾う。
    pub panel_ask: Arc<Mutex<Option<crate::ui::dock::Panel>>>,
}

/// 机の引き出し。焦点に付いて行くか、手で開けたか、手で閉じたか。
/// 手で閉じた物は、焦点が導く物が変わるまで開かない(閉じたそばから開き直さない)。
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(super) enum DeskState {
    #[default]
    Follow,
    Open(crate::ui::desk::Drawer),
    Shut,
}

/// 焦点の型。机の引き出しは型に一つで、機能名では増やさない。
/// 視点の注文。
pub(super) use crate::ui::contracts::ViewRequest;

#[derive(Clone, PartialEq, Debug)]
pub(super) enum Focus {
    Blend(LayerId),
    Color(ColorSlot),
}

/// 色が居る場所。property ではなく shape / text の data を指す(書き戻しもそこ)。
#[derive(Clone, PartialEq, Debug)]
pub(super) enum ColorSlot {
    TextFill {
        layer: LayerId,
        style: crate::doc::store::TextStyleId,
    },
    TextStroke {
        layer: LayerId,
        style: crate::doc::store::TextStyleId,
    },
    /// ShapeNode の木の中の葉。index の列で指す。
    ShapeFill { layer: LayerId, path: Vec<usize> },
    /// 2色gradientの端。`end=false` が最小offset、`end=true` が最大offset。
    /// VecのindexをUIへ漏らさないので、stopの並び順が違う文書でも同じ端を指せる。
    ShapeGradientStop {
        layer: LayerId,
        path: Vec<usize>,
        end: bool,
    },
}

impl ColorSlot {
    pub(super) fn layer(&self) -> LayerId {
        match self {
            Self::TextFill { layer, .. }
            | Self::TextStroke { layer, .. }
            | Self::ShapeFill { layer, .. }
            | Self::ShapeGradientStop { layer, .. } => *layer,
        }
    }

    pub(super) fn is_shape_fill(&self) -> bool {
        matches!(
            self,
            Self::ShapeFill { .. } | Self::ShapeGradientStop { .. }
        )
    }
}

/// 開いている欄。窓に同時に 1 つで、持ち主はここだけ。面は開ける・読む・閉じるだけ。
/// 閉じ損ねは鍵の全喪失になる(`host::aim_keystrokes`)ので、閉じ方は `Field` の 1 箇所。
#[derive(Clone, PartialEq, Debug)]
pub(super) struct OpenField {
    pub at: FieldAt,
    pub draft: String,
    pub number_basis: Option<NumberBasis>,
}

#[derive(Clone, PartialEq, Debug)]
pub(super) struct NumberBasis {
    pub revision: Revision,
    pub at: crate::doc::store::RationalTime,
    pub targets: Vec<LayerId>,
}

/// 欄が指す物。値の型ではなく置き場で見分ける。
#[derive(Clone, PartialEq, Debug)]
pub(super) enum FieldAt {
    /// マーカーの本文。印は並べ替えられ消されるので、index でなく時刻で指す。
    Note(crate::doc::store::RationalTime),
    Number {
        layer: LayerId,
        property: String,
        axis: usize,
    },
    Content(LayerId),
    Name(LayerId),
    /// 色の hex(Colors の輪の下)。
    Hex(ColorSlot),
}

/// タイムラインで選んだキー。区間は「このキーから次のキーまで」。
#[derive(Clone, PartialEq, Debug)]
pub(super) struct KeySel {
    pub layer: LayerId,
    /// 属性の行なら1つ。層の行なら束(その時刻に在る全部)。
    pub property: Option<crate::doc::store::PropertyId>,
    pub at_sec: f64,
}

/// 部品はどれも同じ物への取っ手なので、同じ Document を指していれば同じ session。
impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.doc, &other.doc)
    }
}

impl Session {
    pub(super) fn new(doc: Document, duration_sec: f64, ui: crate::ui::fixture::UiData) -> Self {
        let (timeline_tx, timeline_rx) = std::sync::mpsc::channel();
        let clock = Arc::new(Clock::from_document(&doc, duration_sec));
        let saved_revision = doc.revision();
        let selected_keys = Arc::new(Mutex::new(Vec::new()));
        Self {
            doc: Arc::new(Mutex::new(doc)),
            clock,
            scale: Arc::new(UiScale::new(100)),
            selection: Selection::with_keys(&selected_keys),
            selected_size: Arc::new(Mutex::new(None)),
            timeline_tx,
            timeline_rx: std::rc::Rc::new(timeline_rx),
            ui: Arc::new(ui),
            selected_keys,
            focus: Arc::new(Mutex::new(None)),
            desk: Arc::new(Mutex::new(DeskState::Follow)),
            field: Arc::new(Mutex::new(None)),
            saving: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            scrub: Arc::new(Mutex::new(None)),
            panel_ask: Arc::new(Mutex::new(None)),
            view_camera: Arc::new(Mutex::new(Default::default())),
            view_request: Arc::new(Mutex::new(None)),
            rings: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            output_only: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            frame_dim: Arc::new(std::sync::atomic::AtomicU32::new(75)),
            gesture: GestureSurface::default(),
            surface_capture: SurfaceCapture::default(),
            file_drop: FileDropSurface::default(),
            overshoot: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            export: Default::default(),
            project_notice: Arc::new(Mutex::new(String::new())),
            quit: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            imports: Arc::new(Mutex::new(Vec::new())),
            browser_selection: Arc::new(Mutex::new(Default::default())),
            project_path: Arc::new(Mutex::new(None)),
            saved_revision: Arc::new(Mutex::new(saved_revision)),
            curve_clip: Arc::new(Mutex::new(None)),
        }
    }

    /// 書類の名前(title bar・alert)。仕舞っていなければ Untitled。
    pub(super) fn document_title(&self) -> String {
        self.project_path
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "Untitled".to_owned())
    }

    pub(super) fn is_dirty(&self) -> bool {
        let current = self.doc.lock().unwrap().revision();
        current != *self.saved_revision.lock().unwrap()
    }

    /// 仕舞った revision は、仕舞った時に同じ lock の中で読んだ物を渡す(取り直すと嘘になる)。
    pub(super) fn mark_saved(&self, path: std::path::PathBuf, revision: Revision) {
        *self.project_path.lock().unwrap() = Some(path);
        *self.saved_revision.lock().unwrap() = revision;
    }

    pub(super) fn replace_project(&self, document: Document, path: Option<std::path::PathBuf>) {
        let revision = document.revision();
        *self.doc.lock().unwrap() = document;
        *self.project_path.lock().unwrap() = path;
        *self.saved_revision.lock().unwrap() = revision;
        // 層の id を名指す窓側の手は、作品が変われば全部嘘になる。
        self.selection.clear();
        *self.focus.lock().unwrap() = None;
        *self.desk.lock().unwrap() = DeskState::Follow;
        *self.field.lock().unwrap() = None;
        *self.scrub.lock().unwrap() = None;
        *self.panel_ask.lock().unwrap() = None;
        self.selected_keys.lock().unwrap().clear();
        *self.selected_size.lock().unwrap() = None;
        *self.curve_clip.lock().unwrap() = None;
        *self.view_request.lock().unwrap() = None;
        self.imports.lock().unwrap().clear();
        *self.project_notice.lock().unwrap() = String::new();
        crate::ui::keymap::set_typing(false);
    }

    /// 錠の掛かっていない層だけが書ける。**書く経路は全部ここを通す**(擦り・鍵・色・差し替え・掴み)。
    pub(super) fn writable(&self, layer: LayerId) -> bool {
        edit_rejection(&self.doc.lock().unwrap().view(), layer).is_none()
    }

    pub(super) fn targets(&self, clicked: Option<LayerId>) -> Vec<LayerId> {
        self.selection.targets(clicked)
    }

    pub(super) fn editable_targets(&self, clicked: Option<LayerId>) -> Vec<LayerId> {
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        let mut skipped = Vec::new();
        let targets = self
            .targets(clicked)
            .into_iter()
            .filter(|layer| {
                if let Some(reason) = edit_rejection(&view, *layer) {
                    skipped.push(format!("{}: {reason}", layer.0));
                    false
                } else {
                    true
                }
            })
            .collect();
        if !skipped.is_empty() {
            *self.project_notice.lock().unwrap() = format!("Skipped {}", skipped.join(", "));
        }
        targets
    }

    pub(super) fn property_targets(
        &self,
        clicked: Option<LayerId>,
        property: &crate::doc::store::PropertyId,
    ) -> Result<Vec<LayerId>, crate::doc::store::StoreError> {
        let targets = self.editable_targets(clicked);
        let doc = self.doc.lock().unwrap();
        let view = doc.view().without_transients();
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for layer in targets {
            match view.property_write_rejection(layer, property)? {
                Some(reason) => rejected.push(format!("{}: {reason}", layer.0)),
                None => accepted.push(layer),
            }
        }
        if !rejected.is_empty() {
            *self.project_notice.lock().unwrap() = format!("Skipped {}", rejected.join(", "));
        }
        Ok(accepted)
    }

    pub(super) fn apply_each(
        &self,
        clicked: Option<LayerId>,
        mut verb: impl FnMut(
            &Document,
            LayerId,
        )
            -> Result<Vec<crate::doc::store::Intent>, crate::doc::store::StoreError>,
    ) -> Result<usize, crate::doc::store::StoreError> {
        self.apply_blocks(clicked, |doc, layer| {
            verb(doc, layer).map(crate::ui::functions::compose::Block::Edits)
        })
    }

    pub(super) fn apply_blocks(
        &self,
        clicked: Option<LayerId>,
        verb: impl FnMut(
            &Document,
            LayerId,
        )
            -> Result<crate::ui::functions::compose::Block, crate::doc::store::StoreError>,
    ) -> Result<usize, crate::doc::store::StoreError> {
        let targets = self.editable_targets(clicked);
        let mut doc = self.doc.lock().unwrap();
        let (intents, count, rejected) =
            crate::ui::functions::compose::independent_blocks(&doc, &targets, verb)?;
        doc.apply_all(intents)?;
        if !rejected.is_empty() {
            *self.project_notice.lock().unwrap() = format!(
                "Skipped {}",
                rejected
                    .into_iter()
                    .map(|(layer, reason)| format!("{}: {reason}", layer.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Ok(count)
    }

    /// Undo / Redo の後。消えた層を名指す窓側の手を全部手放す(層の id は嘘になっている)。
    pub(super) fn forget_dead_layers(&self) {
        let live = self.doc.lock().unwrap().view().layers();
        let dead: Vec<LayerId> = self
            .selection
            .all()
            .into_iter()
            .filter(|l| !live.contains(l))
            .collect();
        for l in &dead {
            self.selection.toggle(*l);
        }
        let focus_dead = self.focus.lock().unwrap().as_ref().is_some_and(|f| {
            let layer = match f {
                Focus::Blend(l) => *l,
                Focus::Color(slot) => slot.layer(),
            };
            !live.contains(&layer)
        });
        if focus_dead {
            *self.focus.lock().unwrap() = None;
        }
        let field_dead = self.field().is_some_and(|f| match f.at {
            FieldAt::Number { layer, .. } | FieldAt::Content(layer) | FieldAt::Name(layer) => {
                !live.contains(&layer)
            }
            FieldAt::Hex(ref slot) => !live.contains(&slot.layer()),
            FieldAt::Note(_) => false,
        });
        if field_dead {
            self.close_field();
        }
        *self.scrub.lock().unwrap() = None;
        self.selected_keys
            .lock()
            .unwrap()
            .retain(|k| live.contains(&k.layer));
    }

    /// 錠の掛かっていない選択。書く経路はこちらを見る(錠は Timeline が掛ける)。
    pub(super) fn editable_selection(&self) -> Vec<LayerId> {
        self.editable_targets(None)
    }

    pub(super) fn open_field(&self, at: FieldAt, draft: String) {
        crate::ui::keymap::set_typing(true);
        let number_basis = match &at {
            FieldAt::Number {
                layer, property, ..
            } => {
                let targets = crate::doc::store::PropertyId::new(property)
                    .and_then(|property| self.property_targets(Some(*layer), &property))
                    .unwrap_or_else(|error| {
                        *self.project_notice.lock().unwrap() = error.to_string();
                        Vec::new()
                    });
                let revision = self.doc.lock().unwrap().revision();
                Some(NumberBasis {
                    revision,
                    at: self.clock.current_time(),
                    targets,
                })
            }
            _ => None,
        };
        *self.field.lock().unwrap() = Some(OpenField {
            at,
            draft,
            number_basis,
        });
    }

    pub(super) fn field(&self) -> Option<OpenField> {
        self.field.lock().unwrap().clone()
    }

    /// この置き場の欄が開いていれば、その下書き。
    pub(super) fn field_at(&self, at: &FieldAt) -> Option<String> {
        self.field().filter(|f| f.at == *at).map(|f| f.draft)
    }

    pub(super) fn edit_field(&self, draft: String) {
        if let Some(f) = self.field.lock().unwrap().as_mut() {
            f.draft = draft;
        }
    }

    pub(super) fn close_field(&self) -> Option<OpenField> {
        // flag は欄と同じ寿命。長生きさせると閉じた直後の 1 打鍵(Space)が食われる。
        crate::ui::keymap::set_typing(false);
        self.field.lock().unwrap().take()
    }

    pub(super) fn ask_panel(&self, panel: crate::ui::dock::Panel) {
        *self.panel_ask.lock().unwrap() = Some(panel);
    }

    pub(super) fn take_panel_ask(&self) -> Option<crate::ui::dock::Panel> {
        self.panel_ask.lock().unwrap().take()
    }

    /// 生きている焦点。選んでいる層を指す物だけ。層が変われば焦点は消えたも同じ。
    pub(super) fn live_focus(&self) -> Option<Focus> {
        let focus = self.focus.lock().unwrap().clone()?;
        let layer = match &focus {
            Focus::Blend(layer) => *layer,
            Focus::Color(slot) => slot.layer(),
        };
        (self.selection.get() == Some(layer)).then_some(focus)
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;

    #[test]
    fn ordered_selection_starts_at_the_near_edge_and_reverses_around_a_fixed_anchor() {
        let order = [LayerId(1), LayerId(2), LayerId(3), LayerId(4)];
        let selection = Selection::default();

        assert_eq!(selection.step(&order, 1, false), Some(LayerId(1)));
        selection.clear();
        assert_eq!(selection.step(&order, -1, false), Some(LayerId(4)));

        selection.set(Some(LayerId(2)));
        assert_eq!(selection.step(&order, 1, true), Some(LayerId(3)));
        assert_eq!(selection.all(), vec![LayerId(2), LayerId(3)]);
        assert_eq!(selection.anchor(), Some(LayerId(2)));
        assert_eq!(selection.active(), Some(LayerId(3)));

        assert_eq!(selection.step(&order, -1, true), Some(LayerId(2)));
        assert_eq!(selection.all(), vec![LayerId(2)]);
        assert_eq!(selection.step(&order, -1, true), Some(LayerId(1)));
        assert_eq!(selection.all(), vec![LayerId(1), LayerId(2)]);
        assert_eq!(selection.anchor(), Some(LayerId(2)));
        assert_eq!(selection.active(), Some(LayerId(1)));

        selection.activate(LayerId(2));
        assert_eq!(selection.all(), vec![LayerId(1), LayerId(2)]);
        assert_eq!(selection.active(), Some(LayerId(2)));
        assert_eq!(selection.anchor(), Some(LayerId(2)));
    }

    #[test]
    fn layer_domain_changes_clear_keys_but_a_mixed_marquee_can_publish_both() {
        let keys = Arc::new(Mutex::new(vec![KeySel {
            layer: LayerId(1),
            property: None,
            at_sec: 1.0,
        }]));
        let selection = Selection::with_keys(&keys);
        selection.set(Some(LayerId(2)));
        assert!(keys.lock().unwrap().is_empty());

        keys.lock().unwrap().push(KeySel {
            layer: LayerId(2),
            property: None,
            at_sec: 2.0,
        });
        selection.replace_preserving_keys([LayerId(2), LayerId(3)]);
        assert_eq!(selection.all(), vec![LayerId(2), LayerId(3)]);
        assert_eq!(keys.lock().unwrap().len(), 1);
    }

    fn pointer(
        id: blitz_traits::events::BlitzPointerId,
        client: [f32; 2],
    ) -> blitz_traits::events::BlitzPointerEvent {
        blitz_traits::events::BlitzPointerEvent {
            id,
            is_primary: true,
            coords: blitz_traits::events::PointerCoords {
                page_x: client[0],
                page_y: client[1],
                screen_x: client[0],
                screen_y: client[1],
                client_x: client[0],
                client_y: client[1],
            },
            button: blitz_traits::events::MouseEventButton::Main,
            buttons: blitz_traits::events::MouseEventButtons::Primary,
            mods: Default::default(),
            details: Default::default(),
            element: blitz_traits::events::Point {
                x: client[0] - 10.0,
                y: client[1] - 20.0,
            },
            active_pointers: Default::default(),
        }
    }

    #[test]
    fn capture_ignores_foreign_pointer_and_relays_matching_terminal_once() {
        let capture = SurfaceCapture::default();
        let first = pointer(
            blitz_traits::events::BlitzPointerId::Finger(7),
            [40.0, 60.0],
        );
        let other = pointer(
            blitz_traits::events::BlitzPointerId::Finger(8),
            [50.0, 70.0],
        );
        assert!(capture.begin(CustomSurface::Stage, &first));
        capture.finish(CustomSurface::Stage, &other);
        assert!(capture.owns(CustomSurface::Stage, &first));
        assert!(!capture.relay_from_other_surface(
            CustomSurface::Timeline,
            CapturePhase::Up,
            &other,
        ));
        assert!(capture.relay_from_other_surface(
            CustomSurface::Timeline,
            CapturePhase::Move,
            &blitz_traits::events::BlitzPointerEvent {
                coords: blitz_traits::events::PointerCoords {
                    page_x: 50.0,
                    page_y: 80.0,
                    screen_x: 50.0,
                    screen_y: 80.0,
                    client_x: 50.0,
                    client_y: 80.0,
                },
                element: blitz_traits::events::Point { x: 1.0, y: 1.0 },
                ..first.clone()
            },
        ));
        let moved = capture.take(CustomSurface::Stage);
        let blitz_traits::events::UiEvent::PointerMove(moved) = moved[0].event() else {
            panic!("capture did not preserve a move")
        };
        assert_eq!((moved.element.x, moved.element.y), (40.0, 60.0));
        assert!(capture.relay_from_other_surface(
            CustomSurface::Timeline,
            CapturePhase::Up,
            &first,
        ));
        assert_eq!(capture.take(CustomSurface::Stage).len(), 1);
        assert!(capture.take(CustomSurface::Stage).is_empty());
    }

    #[test]
    fn a_direct_move_is_not_queued_again_when_it_bubbles_to_the_root() {
        let capture = SurfaceCapture::default();
        let direct = pointer(blitz_traits::events::BlitzPointerId::Mouse, [40.0, 60.0]);
        assert!(capture.begin(CustomSurface::Stage, &direct));
        capture.note_direct(CustomSurface::Stage, CapturePhase::Move, &direct);
        assert!(!capture.relay_dom(
            CapturePhase::Move,
            "mouse",
            0,
            true,
            [40.0, 60.0],
            true,
            Default::default(),
        ));
        assert!(capture.take(CustomSurface::Stage).is_empty());
        assert!(capture.relay_dom(
            CapturePhase::Move,
            "mouse",
            0,
            true,
            [42.0, 61.0],
            true,
            Default::default(),
        ));
        assert_eq!(capture.take(CustomSurface::Stage).len(), 1);
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

    /// Undo で消えた層を名指す手は全部手放す。
    #[test]
    fn undo_forgets_selection_focus_and_field_of_a_dead_layer() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let layer = LayerId(session.doc.lock().unwrap().view().next_layer_id());
        session
            .doc
            .lock()
            .unwrap()
            .apply(crate::doc::store::Intent::AddLayer(layer))
            .unwrap();
        session.selection.set(Some(layer));
        *session.focus.lock().unwrap() = Some(Focus::Blend(layer));
        session.open_field(FieldAt::Name(layer), "x".into());
        assert!(session.doc.lock().unwrap().undo());
        session.forget_dead_layers();
        assert_eq!(session.selection.get(), None);
        assert!(session.focus.lock().unwrap().is_none());
        assert!(session.field().is_none());
    }

    #[test]
    fn dirty_state_is_only_the_difference_from_the_saved_revision() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        assert!(!session.is_dirty());

        let layer = LayerId(session.doc.lock().unwrap().view().next_layer_id());
        session
            .doc
            .lock()
            .unwrap()
            .apply(crate::doc::store::Intent::AddLayer(layer))
            .unwrap();
        assert!(session.is_dirty());

        let rev = session.doc.lock().unwrap().revision();
        session.mark_saved(std::path::PathBuf::from("song.rrd"), rev);
        assert!(!session.is_dirty());

        session
            .doc
            .lock()
            .unwrap()
            .apply(crate::doc::store::Intent::AddLayer(LayerId(layer.0 + 1)))
            .unwrap();
        assert!(session.is_dirty());

        session.replace_project(crate::ui::blank_project(), None);
        assert!(!session.is_dirty());
        assert!(session.project_path.lock().unwrap().is_none());
    }
}

/// 書き込みの結果を 1 箇所で扱う: 通れば revision を上げ、通らなければ PROBE に残す。
/// 同じ 4 行が 20 箇所に在った(Rust 初学者の会議)。
pub(super) fn noted<T>(
    result: Result<T, crate::doc::store::StoreError>,
    mut revision: dioxus_native::prelude::Signal<u32>,
) {
    use dioxus_native::prelude::WritableExt;
    match result {
        Ok(_) => *revision.write() += 1,
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}
