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
    /// Stage を「出す物だけ」で映す(取っ手も枠も無し)。View menu の Output only。
    pub output_only: Arc<std::sync::atomic::AtomicBool>,
    /// 枠の外へかける膜の濃さ(%)。見る側の設定で、作品には入らない。
    pub frame_dim: Arc<std::sync::atomic::AtomicU32>,
    pub gesture: GestureSurface,
    pub file_drop: FileDropSurface,
    pub overshoot: Arc<std::sync::atomic::AtomicBool>,
    pub export: crate::ui::output::ExportController,
    pub project_notice: Arc<Mutex<String>>,
    /// 終わる注文(⌘Q / File ▸ Quit)。未保存の確認が通ったら立ち、窓の糸が拾って終わる。
    pub quit: Arc<std::sync::atomic::AtomicBool>,
    /// 別の糸で指紋を取り終えた取り込み。窓の糸が echo の度に拾って棚へ入れる。
    pub imports: Arc<Mutex<Vec<Vec<crate::ui::fixture::Prepared>>>>,
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum ViewRequest {
    /// 窓に収める(⌘0)。
    Fit,
    /// 画素等倍(⌘1)。
    Actual,
    /// 段階で寄る / 引く(⌘= / ⌘−)。
    Step(f64),
}

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
        matches!(self, Self::ShapeFill { .. } | Self::ShapeGradientStop { .. })
    }
}

/// 開いている欄。窓に同時に 1 つで、持ち主はここだけ。面は開ける・読む・閉じるだけ。
/// 閉じ損ねは鍵の全喪失になる(`host::aim_keystrokes`)ので、閉じ方は `Field` の 1 箇所。
#[derive(Clone, PartialEq, Debug)]
pub(super) struct OpenField {
    pub at: FieldAt,
    pub draft: String,
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
            file_drop: FileDropSurface::default(),
            overshoot: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            export: Default::default(),
            project_notice: Arc::new(Mutex::new(String::new())),
            quit: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            imports: Arc::new(Mutex::new(Vec::new())),
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
        self.doc
            .lock()
            .unwrap()
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .is_none_or(|a| !a.locked)
    }

    /// Undo / Redo の後。消えた層を名指す窓側の手を全部手放す(層の id は嘘になっている)。
    pub(super) fn forget_dead_layers(&self) {
        let live = self.doc.lock().unwrap().view().layers();
        let dead: Vec<LayerId> = self.selection.all().into_iter().filter(|l| !live.contains(l)).collect();
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
            FieldAt::Number { layer, .. } | FieldAt::Content(layer) | FieldAt::Name(layer) => !live.contains(&layer),
            FieldAt::Hex(ref slot) => !live.contains(&slot.layer()),
            FieldAt::Note(_) => false,
        });
        if field_dead {
            self.close_field();
        }
        *self.scrub.lock().unwrap() = None;
        self.selected_keys.lock().unwrap().retain(|k| live.contains(&k.layer));
    }

    /// 錠の掛かっていない選択。書く経路はこちらを見る(錠は Timeline が掛ける)。
    pub(super) fn editable_selection(&self) -> Vec<LayerId> {
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        self.selection
            .all()
            .into_iter()
            .filter(|l| !view.attrs(*l).ok().flatten().is_some_and(|a| a.locked))
            .collect()
    }

    pub(super) fn open_field(&self, at: FieldAt, draft: String) {
        crate::ui::keymap::set_typing(true);
        *self.field.lock().unwrap() = Some(OpenField { at, draft });
    }

    pub(super) fn field(&self) -> Option<OpenField> {
        self.field.lock().unwrap().clone()
    }

    /// この置き場の欄が開いていれば、その下書き。
    pub(super) fn field_at(&self, at: &FieldAt) -> Option<String> {
        self.field()
            .filter(|f| f.at == *at)
            .map(|f| f.draft)
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
mod project_tests {
    use super::*;

    /// Undo で消えた層を名指す手は全部手放す。
    #[test]
    fn undo_forgets_selection_focus_and_field_of_a_dead_layer() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let layer = LayerId(session.doc.lock().unwrap().view().next_layer_id());
        session.doc.lock().unwrap().apply(crate::doc::store::Intent::AddLayer(layer)).unwrap();
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
pub(super) fn noted<T>(result: Result<T, crate::doc::store::StoreError>, mut revision: dioxus_native::prelude::Signal<u32>) {
    use dioxus_native::prelude::WritableExt;
    match result {
        Ok(_) => *revision.write() += 1,
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}
