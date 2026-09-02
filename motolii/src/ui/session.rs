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

#[derive(Clone, Default)]
pub(super) struct Selection(Arc<Mutex<Vec<LayerId>>>);

impl Selection {
    pub(super) fn clear(&self) {
        self.0.lock().unwrap().clear();
    }

    pub(super) fn get(&self) -> Option<LayerId> {
        self.0.lock().unwrap().last().copied()
    }

    pub(super) fn set(&self, layer: Option<LayerId>) {
        let mut v = self.0.lock().unwrap();
        v.clear();
        if let Some(l) = layer {
            v.push(l);
        }
    }

    pub(super) fn all(&self) -> Vec<LayerId> {
        self.0.lock().unwrap().clone()
    }

    pub(super) fn contains(&self, layer: LayerId) -> bool {
        self.0.lock().unwrap().contains(&layer)
    }

    pub(super) fn toggle(&self, layer: LayerId) {
        let mut v = self.0.lock().unwrap();
        match v.iter().position(|l| *l == layer) {
            Some(i) => {
                v.remove(i);
            }
            None => v.push(layer),
        }
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
    /// 枠の外へかける膜の濃さ(%)。見る側の設定で、作品には入らない。
    pub frame_dim: Arc<std::sync::atomic::AtomicU32>,
    pub gesture: GestureSurface,
    pub file_drop: FileDropSurface,
    pub overshoot: Arc<std::sync::atomic::AtomicBool>,
    pub export: crate::ui::output::ExportController,
    pub project_notice: Arc<Mutex<String>>,
    /// 今の作品の仕舞い先。`Save` が問い直さないために覚える。
    pub project_path: Arc<Mutex<Option<std::path::PathBuf>>>,
    /// 最後に保存／読込／NewしたDocument revision。dirtyは現在との差だけで決まる。
    pub saved_revision: Arc<Mutex<Revision>>,
    pub curve_clip: Arc<Mutex<Option<crate::doc::store::Interp>>>,
    /// 見る側のカメラ(User View)。**Document には入らない** — 書き出しには出ない。
    pub view_camera: Arc<Mutex<crate::render::engine::ObservationCamera>>,
    /// 今どのキーを掴んでいるか。イージングを触る口が要る(Document には入らない)。
    pub selected_keys: Arc<Mutex<Vec<KeySel>>>,
    /// 今どの値に手が触れているか。Inspector が行を光らせて書き、机が覗く。
    /// 机を呼ぶ口ではない — 机は Document とこれを読むだけ。
    pub focus: Arc<Mutex<Option<Focus>>>,
    /// 机の引き出しの開閉。窓をまたいで 1 つ。
    pub desk: Arc<Mutex<DeskState>>,
}

/// 机の引き出し。焦点に付いて行くか、手で開けたか、手で閉じたか。
/// 手で閉じた物は、焦点が導く物が変わるまで開かない(閉じたそばから開き直さない)。
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(super) enum DeskState {
    #[default]
    Follow,
    Open(crate::ui::desk::Drawer),
    Shut(Option<crate::ui::desk::Drawer>),
}

/// 焦点の型。机の引き出しは型に一つで、機能名では増やさない。
#[derive(Clone, PartialEq, Debug)]
pub(super) enum Focus {
    Blend(LayerId),
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
        Self {
            doc: Arc::new(Mutex::new(doc)),
            clock,
            scale: Arc::new(UiScale::new(100)),
            selection: Selection::default(),
            selected_size: Arc::new(Mutex::new(None)),
            timeline_tx,
            timeline_rx: std::rc::Rc::new(timeline_rx),
            ui: Arc::new(ui),
            selected_keys: Arc::new(Mutex::new(Vec::new())),
            focus: Arc::new(Mutex::new(None)),
            desk: Arc::new(Mutex::new(DeskState::Follow)),
            view_camera: Arc::new(Mutex::new(Default::default())),
            rings: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            frame_dim: Arc::new(std::sync::atomic::AtomicU32::new(75)),
            gesture: GestureSurface::default(),
            file_drop: FileDropSurface::default(),
            overshoot: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            export: Default::default(),
            project_notice: Arc::new(Mutex::new(String::new())),
            project_path: Arc::new(Mutex::new(None)),
            saved_revision: Arc::new(Mutex::new(saved_revision)),
            curve_clip: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn is_dirty(&self) -> bool {
        let current = self.doc.lock().unwrap().revision();
        current != *self.saved_revision.lock().unwrap()
    }

    pub(super) fn mark_saved(&self, path: std::path::PathBuf) {
        let current = self.doc.lock().unwrap().revision();
        *self.project_path.lock().unwrap() = Some(path);
        *self.saved_revision.lock().unwrap() = current;
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
        self.selected_keys.lock().unwrap().clear();
        *self.selected_size.lock().unwrap() = None;
        *self.curve_clip.lock().unwrap() = None;
    }

    /// 生きている焦点。選んでいる層を指す物だけ。層が変われば焦点は消えたも同じ。
    pub(super) fn live_focus(&self) -> Option<Focus> {
        let focus = self.focus.lock().unwrap().clone()?;
        match focus {
            Focus::Blend(layer) if self.selection.get() == Some(layer) => Some(focus),
            Focus::Blend(_) => None,
        }
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

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

        session.mark_saved(std::path::PathBuf::from("song.rrd"));
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
