use std::sync::{Arc, Mutex};

use crate::doc::store::{Document, LayerId};

use crate::ui::playback::Clock;
use crate::ui::timeline_widget::TimelineMsg;
use crate::ui::tokens::UiScale;

#[derive(Clone, Default)]
pub(super) struct Selection(Arc<Mutex<Vec<LayerId>>>);

impl Selection {
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
    /// 掴みの取り消し。上がるたびに、掴んでいる物は**書かずに**手を離す
    /// (規格が MUST で求める pointercancel の役)。
    pub cancel_gesture: Arc<std::sync::atomic::AtomicU32>,
    /// いま何かを掴んでいるか。`Esc` の意味を段で分けるために要る
    /// (掴んでいる間は取り消し、そうでなければ選択を解く)。
    pub gesture_active: Arc<std::sync::atomic::AtomicBool>,
    pub overshoot: Arc<std::sync::atomic::AtomicBool>,
    /// 写した曲線。区間から区間へ貼るための控え(Document には入らない)。
    /// 書き出しの一言。別の糸が書き、窓が読む。
    pub outgo: Arc<Mutex<String>>,
    pub curve_clip: Arc<Mutex<Option<crate::doc::store::Interp>>>,
    /// 見る側のカメラ(User View)。**Document には入らない** — 書き出しには出ない。
    pub view_camera: Arc<Mutex<crate::render::engine::ObservationCamera>>,
    /// 今どのキーを掴んでいるか。イージングを触る口が要る(Document には入らない)。
    pub selected_keys: Arc<Mutex<Vec<KeySel>>>,
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
        Self {
            doc: Arc::new(Mutex::new(doc)),
            clock: Arc::new(Clock::new(duration_sec)),
            scale: Arc::new(UiScale::new(100)),
            selection: Selection::default(),
            selected_size: Arc::new(Mutex::new(None)),
            timeline_tx,
            timeline_rx: std::rc::Rc::new(timeline_rx),
            ui: Arc::new(ui),
            selected_keys: Arc::new(Mutex::new(Vec::new())),
            view_camera: Arc::new(Mutex::new(Default::default())),
            rings: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            frame_dim: Arc::new(std::sync::atomic::AtomicU32::new(75)),
            cancel_gesture: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            gesture_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            overshoot: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            outgo: Arc::new(Mutex::new(String::new())),
            curve_clip: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(test)]
mod selection_invariants {
    use super::*;

    const A: LayerId = LayerId(1);
    const B: LayerId = LayerId(2);

    #[test]
    fn set_replaces_and_get_returns_it() {
        let s = Selection::default();
        s.set(Some(A));
        s.set(Some(B));
        assert_eq!(s.all(), vec![B]);
        assert_eq!(s.get(), Some(B));
    }

    #[test]
    fn set_none_empties() {
        let s = Selection::default();
        s.set(Some(A));
        s.set(None);
        assert!(s.all().is_empty());
        assert_eq!(s.get(), None);
    }

    #[test]
    fn toggle_twice_returns_to_start() {
        let s = Selection::default();
        s.set(Some(A));
        let before = s.all();
        s.toggle(B);
        s.toggle(B);
        assert_eq!(s.all(), before);
    }

    #[test]
    fn toggled_in_layer_becomes_primary() {
        let s = Selection::default();
        s.set(Some(A));
        s.toggle(B);
        assert_eq!(s.get(), Some(B));
        assert!(s.contains(A));
        assert_eq!(s.all().len(), 2);
    }

    #[test]
    fn toggling_out_the_primary_promotes_the_previous() {
        let s = Selection::default();
        s.set(Some(A));
        s.toggle(B);
        s.toggle(B);
        assert_eq!(s.get(), Some(A));
    }
}
