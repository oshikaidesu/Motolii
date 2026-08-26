pub use makepad_widgets;

use makepad_widgets::*;
use motolii_engine::Engine;
use motolii_shell_state::Session;
use motolii_store::{Document, Intent, LayerId, RationalTime};
use motolii_timeline_pane::{self as timeline_pane, stacking::restacked, StackDirection};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

mod browser_surface;
mod chrome;
mod export_surface;
mod gesture_input;
mod inspector_surface;
mod settings_surface;
mod stage_chrome;
mod stage_import;
mod stage_surface;
mod timeline_surface;
use stage_surface::{SharedOsHandle, SharedSurfaceDesc, StagePresent};
use timeline_surface::{
    TimelineLane, TimelineModel, TimelinePropertyLane, TimelineSurface, TimelineSurfaceAction,
};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.HotPanelBase = #(HotPanel::register_widget(vm))
    mod.widgets.HotPanel = set_type_default() do mod.widgets.HotPanelBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x1c1c1c
        panel_error := Label{width: Fill height: Fill align: Align{x: 0.5 y: 0.5} text: "" draw_text.color: #xe8c48a draw_text.text_style: theme.font_code{font_size: 10}}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1440, 900)
                window.title: "Motolii Makepad Panel"
                body +: {
                    panel_host := mod.widgets.HotPanel{
                        width: Fill
                        height: Fill
                        flow: Down
                    }
                }
            }
        }
    }
}

const HOT_PANEL_PREFIX: &str =
    "use mod.prelude.widgets.*\nuse mod.widgets.*\nView{width:Fill height:Fill flow:Down, ";

const PANEL_ERROR_SOURCE: &str = concat!(
    "use mod.prelude.widgets.*\nuse mod.widgets.*\n",
    "SolidView{width:Fill height:Fill flow:Down show_bg:true new_batch:true draw_bg.color:#x1c1c1c ",
    "panel_error := Label{width:Fill height:Fill align:Align{x:0.5 y:0.5} text:\"\" ",
    "draw_text.color:#xe8c48a draw_text.text_style:theme.font_code{font_size:10}}}",
);

fn format_panel_eval_errors(errors: &[String]) -> String {
    match errors.first() {
        Some(first) if errors.len() == 1 => format!("panel.splash: {first}"),
        Some(first) => format!("panel.splash: {first} （+{}）", errors.len() - 1),
        None => "panel.splash を評価できない".to_string(),
    }
}

fn view_has_children(view: &View) -> bool {
    !view.children.is_empty()
}

/// Makepad view adapter. Writes go to Document / Session; pixels come from Engine.
/// widgets never keep a second Document.
struct BackendBridge {
    doc: Document,
    session: Session,
    engine: Engine,
    frame: Option<(u32, u32, Vec<u8>)>,
    status: Option<String>,
    playing: bool,
    present: StagePresent,
    stage_texture: Option<Texture>,
    stage_gpu: Option<wgpu::Texture>,
}

enum TimelineUpdate {
    None,
    Stage(String),
    ModelAndStage(String),
    Status(String),
}

enum SharedPresentResult {
    Shown,
    CreateFailed,
    WriteFailed,
}

/// A one-slot mailbox for expensive projections. Producers may run at pointer
/// frequency; the consumer runs at display frequency and only observes the
/// newest request. The payload can later become a `StageSurfaceSlot` without
/// changing Timeline or App event routing.
#[derive(Default)]
struct LatestFrameRequest {
    pending: bool,
}

impl LatestFrameRequest {
    fn request(&mut self) -> bool {
        std::mem::replace(&mut self.pending, true)
    }

    fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

impl BackendBridge {
    fn new_fixture() -> Self {
        let built = motolii_fixture::build();
        Self {
            session: Session {
                playhead: built.playhead,
                selection: Some(built.selected),
                selected_layers: vec![built.selected],
                ..Session::default()
            },
            doc: built.doc,
            engine: Engine::new().expect("GPU を用意できない"),
            frame: None,
            status: Some(built.status),
            playing: false,
            present: StagePresent::FallbackCpu,
            stage_texture: None,
            stage_gpu: None,
        }
    }

    fn display_name(name: &str) -> String {
        match name {
            "タイトルロゴ" => "Title Logo",
            "メインボーカル映像" => "Main Vocal",
            "Bロール_街並み" => "B-roll City",
            "1番Aメロ歌詞" => "Verse A Lyrics",
            "Bメロ歌詞" => "Pre-chorus Lyrics",
            "ダンスカット" => "Dance Cut",
            "サビ歌詞" => "Chorus Lyrics",
            "グリッチトランジション" => "Glitch Transition",
            "2番Aメロ歌詞" => "Verse 2 Lyrics",
            "波形ビジュアライザ" => "Wave Visualizer",
            "リリックモーション背景" => "Lyric Motion BG",
            "ラスサビ歌詞" => "Last Chorus Lyrics",
            "Bロール_夜景" => "B-roll Night",
            "エンドカード" => "End Card",
            "クレジット" => "Credits",
            other => other,
        }
        .to_owned()
    }

    fn timeline_model(&self) -> TimelineModel {
        let store = self.doc.view();
        let mut rows = timeline_pane::rows(&store, &self.session);
        // Timeline top means Stage front. Both are derived from the same
        // `LayerMeta.order`; no independent lane-order state exists here.
        rows.sort_by(|left, right| {
            let left_order = store
                .meta(left.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            let right_order = store
                .meta(right.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            right_order
                .cmp(&left_order)
                .then_with(|| right.id.cmp(&left.id))
        });
        drop(store);

        let lanes = rows
            .into_iter()
            .map(|row| TimelineLane {
                id: row.id.0,
                name: Self::display_name(&row.name),
                hidden: row.hidden,
                solo: row.solo,
                locked: row.locked,
                label_color: row
                    .label_color
                    .map(usize::from)
                    .unwrap_or_else(|| row.id.0.saturating_sub(1) as usize),
                start: row.start,
                duration: row.duration,
                selected: row.selected,
            })
            .collect();
        let property_lanes = timeline_pane::property_rows(
            &self.doc.view(),
            &self.session,
            self.doc.view().composition().ok().flatten().map(|c| c.fps),
        )
        .into_iter()
        .map(|row| TimelinePropertyLane {
            layer_id: row.layer.0,
            name: format!("> {}", row.property.name()),
            keys: row.keys.into_iter().map(|key| key.frame).collect(),
        })
        .collect();
        let composition = self.doc.view().composition().ok().flatten();
        let (duration_frames, fps_num, fps_den) = composition
            .map(|composition| {
                (
                    composition.duration_frames,
                    composition.fps.num(),
                    composition.fps.den(),
                )
            })
            .unwrap_or((1, 30, 1));

        TimelineModel {
            lanes,
            property_lanes,
            duration_frames,
            playhead: self.session.playhead,
            fps_num,
            fps_den,
        }
    }

    fn scrub_to(&mut self, frame: i64) {
        let started = Instant::now();
        self.session.playhead = frame.max(0);
        self.frame = None;
        log!(
            "PERF store_scrub frame={} elapsed_us={}",
            frame,
            started.elapsed().as_micros()
        );
    }

    fn restack_from_timeline(&mut self, layer_id: u64, target_from_front: usize) -> String {
        let store = self.doc.view();
        let Some(layer) = timeline_pane::rows(&store, &self.session)
            .into_iter()
            .find(|row| row.id.0 == layer_id)
            .map(|row| row.id)
        else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        let layer_count = store.layers().len();
        drop(store);
        let target_from_front = target_from_front.min(layer_count.saturating_sub(1));
        let target_from_back = layer_count
            .saturating_sub(1)
            .saturating_sub(target_from_front);

        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];

        let store = self.doc.view();
        let stack: Vec<(LayerId, i16)> = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
            .collect();
        drop(store);
        let changes = restacked(
            &stack,
            &[layer],
            StackDirection::ToIndexFromBack(target_from_back),
        );
        if !changes.is_empty() {
            let intents: Vec<Intent> = changes
                .into_iter()
                .map(|(layer, order)| Intent::SetOrder { layer, order })
                .collect();
            if let Err(error) = self.doc.apply_all(intents) {
                return format!("重なりを書けない: {error}");
            }
            self.frame = None;
        }
        self.status = Some(format!(
            "Timeline: layer {} moved to lane {} / Stage stack updated",
            layer_id,
            target_from_front + 1
        ));
        self.status.clone().expect("just set")
    }

    /// screenshot / export など明示 fallback 専用。通常の playhead / 再生からは呼ばない。
    fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        if self.frame.is_none() {
            let composition = self.doc.view().composition().ok().flatten()?;
            let t = RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()?;
            let rgba = self.engine.render_frame(&self.doc.view(), t).ok()?;
            self.frame = Some((composition.width, composition.height, rgba));
        }
        self.frame
            .as_ref()
            .map(|(width, height, rgba)| (*width, *height, rgba.as_slice()))
    }

    fn toggle_playback(&mut self) -> bool {
        self.playing = !self.playing;
        self.playing
    }

    fn playback_tick(&mut self) -> bool {
        if !self.playing {
            return false;
        }
        let started = Instant::now();
        let duration = self
            .doc
            .view()
            .composition()
            .ok()
            .flatten()
            .map(|composition| composition.duration_frames)
            .unwrap_or(1)
            .max(1);
        self.session.playhead = (self.session.playhead + 1) % duration;
        self.frame = None;
        log!(
            "PERF store_playback frame={} elapsed_us={}",
            self.session.playhead,
            started.elapsed().as_micros()
        );
        true
    }

    fn apply_timeline_action(&mut self, action: &TimelineSurfaceAction) -> TimelineUpdate {
        match *action {
            TimelineSurfaceAction::None => TimelineUpdate::None,
            TimelineSurfaceAction::Scrub(frame) => {
                self.scrub_to(frame);
                TimelineUpdate::Stage(format!("SCRUB  ·  FRAME {frame}"))
            }
            TimelineSurfaceAction::Restack {
                layer_id,
                target_from_front,
            } => TimelineUpdate::ModelAndStage(
                self.restack_from_timeline(layer_id, target_from_front),
            ),
            TimelineSurfaceAction::ZoomChanged {
                start_frame,
                visible_frames,
            } => TimelineUpdate::Status(format!(
                "TIME ZOOM  ·  X ONLY  ·  START {start_frame}  ·  SPAN {visible_frames}F"
            )),
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct HotPanel {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[live]
    body: ArcStringMut,
    #[rust]
    host_error: Option<String>,
}

impl WidgetNode for HotPanel {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for HotPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.body.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, value: &str) {
        if self.body.as_ref() == value && self.host_error.is_none() {
            log!("panel source unchanged; keeping installed view");
            return;
        }

        let code = format!("{}{}", HOT_PANEL_PREFIX, value);
        match Self::eval_panel_view(cx, "panel.splash", &code) {
            Ok(view) => {
                self.body.set(value);
                self.view = view;
                self.host_error = None;
                // Eval apply does not keep type-default / PREFIX walk+flow on the
                // replaced View (Fit parent + Fill child = 0; default flow Right
                // puts shell:Fill next to status:Fill). Set them on the instance.
                self.apply_host_layout();
                // The initial placeholder has no useful draw area yet. Redrawing only
                // this widget can therefore miss the first frame after replacement.
                cx.redraw_all();
                log!("panel view installed: {} bytes", value.len());
            }
            Err(error) => self.install_panel_error(cx, &error),
        }
    }
}

impl HotPanel {
    fn apply_host_layout(&mut self) {
        self.view.walk = Walk::fill();
        self.view.layout.flow = Flow::Down;
    }

    fn eval_panel_view(cx: &mut Cx, file: &str, code: &str) -> Result<View, String> {
        cx.with_vm(|vm| {
            let script_mod = ScriptMod {
                cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
                module_path: "r7_makepad_panel::hot_panel".to_string(),
                file: file.to_string(),
                line: 0,
                column: 0,
                code: String::new(),
                values: vec![],
            };
            let value = vm.eval_with_append_source(script_mod, code, NIL.into());
            let errors = vm.take_errors();
            if value.is_err() || !errors.is_empty() {
                return Err(format_panel_eval_errors(&errors));
            }
            if value.as_object().is_none() {
                return Err("空の View になった".to_string());
            }
            let view = View::script_from_value(vm, value);
            if !view_has_children(&view) {
                return Err("空の View になった".to_string());
            }
            Ok(view)
        })
    }

    fn install_panel_error(&mut self, cx: &mut Cx, text: &str) {
        if self.host_error.as_deref() == Some(text) {
            return;
        }
        let has_error_label = !self
            .view
            .child_by_path(ids!(panel_error))
            .as_label()
            .is_empty();
        if !has_error_label {
            match Self::eval_panel_view(cx, "panel_error.splash", PANEL_ERROR_SOURCE) {
                Ok(view) => self.view = view,
                Err(_) => {}
            }
        }
        self.apply_host_layout();
        self.view
            .child_by_path(ids!(panel_error))
            .as_label()
            .set_text(cx, text);
        self.host_error = Some(text.to_string());
        self.body.set("");
        cx.redraw_all();
    }

    fn dock(&self) -> DockRef {
        self.view.child_by_path(ids!(dock)).as_dock()
    }

    fn set_stage_texture(&mut self, cx: &mut Cx, texture: Texture) -> bool {
        let stage_image = self
            .dock()
            .item(id!(stage))
            .child_by_path(ids!(stage_frame))
            .as_image();
        let found = !stage_image.is_empty();
        stage_image.set_texture(cx, Some(texture));
        found
    }

    fn set_stage_error(&self, cx: &mut Cx, text: &str) {
        self.dock()
            .item(id!(stage))
            .child_by_path(ids!(stage_error))
            .as_label()
            .set_text(cx, text);
    }

    fn timeline_ref(&self) -> WidgetRef {
        self.dock()
            .item(id!(timeline))
            .child_by_path(ids!(timeline_surface))
    }

    fn play_ref(&self) -> WidgetRef {
        self.dock()
            .item(id!(timeline))
            .child_by_path(ids!(play_toggle))
    }

    fn set_timeline_model(&mut self, cx: &mut Cx, model: TimelineModel) -> bool {
        let timeline = self.timeline_ref();
        let found = !timeline.is_empty();
        if let Some(mut timeline) = timeline.borrow_mut::<TimelineSurface>() {
            timeline.set_model(cx, model);
        }
        found
    }

    fn set_status(&self, cx: &mut Cx, text: &str) {
        self.view
            .child_by_path(ids!(status))
            .as_label()
            .set_text(cx, text);
    }

    fn last_install_ok(&self) -> bool {
        self.host_error.is_none() && !self.body.as_ref().is_empty()
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    panel_path: PathBuf,
    #[rust]
    panel_signature: Option<(SystemTime, u64)>,
    #[rust]
    pending_signature: Option<(SystemTime, u64)>,
    #[rust]
    panel_timer: Timer,
    #[rust]
    playback_timer: Timer,
    #[rust]
    stage_next_frame: NextFrame,
    #[rust]
    stage_request: LatestFrameRequest,
    /// The existing product shell remains the sole Document/Engine owner. This
    /// probe only reads its compositor output for the Makepad Stage image.
    #[rust]
    backend: Option<BackendBridge>,
}

impl App {
    fn install_stage_frame(&mut self, cx: &mut Cx) {
        match self.try_present_shared(cx) {
            SharedPresentResult::Shown => self.set_stage_error(cx, ""),
            SharedPresentResult::CreateFailed => self.set_stage_error(cx, "共有面が作れなかった"),
            SharedPresentResult::WriteFailed => self.set_stage_error(cx, "書けなかった"),
        }
    }

    fn try_present_shared(&mut self, cx: &mut Cx) -> SharedPresentResult {
        let Some(backend) = self.backend.as_mut() else {
            return SharedPresentResult::CreateFailed;
        };
        let Some(composition) = backend.doc.view().composition().ok().flatten() else {
            return SharedPresentResult::CreateFailed;
        };
        let desc = SharedSurfaceDesc::from_comp(composition.width, composition.height);
        let recreate = backend.present.needs_recreate(desc) || backend.stage_gpu.is_none();
        if recreate {
            let (texture, handle) = cx.create_presentable_texture(
                desc.width,
                desc.height,
                SharedPresentablePixel::Rgba8Srgb,
            );
            let handle = match handle {
                makepad_widgets::SharedOsHandle::IoSurfaceId(id) => SharedOsHandle::IoSurfaceId(id),
                makepad_widgets::SharedOsHandle::DxgiSharedHandle(v) => {
                    SharedOsHandle::DxgiSharedHandle(v)
                }
                makepad_widgets::SharedOsHandle::DmaBufFd(fd) => SharedOsHandle::DmaBufFd(fd),
            };
            let Some(present) = StagePresent::shared(desc, handle) else {
                return SharedPresentResult::CreateFailed;
            };
            let Some(gpu) =
                stage_import::import_presentable(backend.engine.gpu_device(), desc, handle)
            else {
                return SharedPresentResult::CreateFailed;
            };
            backend.stage_texture = Some(texture);
            backend.stage_gpu = Some(gpu);
            backend.present = present;
        }
        let Some(gpu) = backend.stage_gpu.as_ref() else {
            return SharedPresentResult::CreateFailed;
        };
        let Ok(t) = RationalTime::try_from_frame(backend.session.playhead, composition.fps) else {
            return SharedPresentResult::WriteFailed;
        };
        if backend
            .engine
            .render_frame_into(&backend.doc.view(), t, gpu)
            .is_err()
        {
            return SharedPresentResult::WriteFailed;
        }
        if let Some(texture) = backend.stage_texture.clone() {
            let panel = self.ui.widget(cx, ids!(panel_host));
            let _ = panel
                .borrow_mut::<HotPanel>()
                .map(|mut panel| panel.set_stage_texture(cx, texture))
                .unwrap_or(false);
        }
        self.ui.widget(cx, ids!(panel_host)).redraw(cx);
        SharedPresentResult::Shown
    }

    fn set_stage_error(&self, cx: &mut Cx, text: &str) {
        let panel = self.ui.widget(cx, ids!(panel_host));
        if let Some(panel) = panel.borrow::<HotPanel>() {
            panel.set_stage_error(cx, text);
        }
        panel.redraw(cx);
    }

    fn request_stage_frame(&mut self, cx: &mut Cx) {
        if !self.stage_request.request() {
            self.stage_next_frame = cx.new_next_frame();
        }
    }

    fn install_timeline_model(&mut self, cx: &mut Cx) {
        let started = Instant::now();
        let Some(model) = self.backend.as_ref().map(BackendBridge::timeline_model) else {
            return;
        };
        let panel = self.ui.widget(cx, ids!(panel_host));
        let timeline_found = panel
            .borrow_mut::<HotPanel>()
            .map(|mut panel| panel.set_timeline_model(cx, model))
            .unwrap_or(false);
        log!(
            "PERF timeline_projection elapsed_us={} timeline_found={}",
            started.elapsed().as_micros(),
            timeline_found,
        );
    }

    fn set_status(&self, cx: &mut Cx, status: &str) {
        let panel = self.ui.widget(cx, ids!(panel_host));
        if let Some(panel) = panel.borrow::<HotPanel>() {
            panel.set_status(cx, status);
        };
    }

    fn timeline_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let panel = self.ui.widget(cx, ids!(panel_host));
        panel.borrow::<HotPanel>().and_then(|panel| {
            let timeline = panel.timeline_ref();
            (!timeline.is_empty()).then(|| timeline.widget_uid())
        })
    }

    fn play_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let panel = self.ui.widget(cx, ids!(panel_host));
        panel.borrow::<HotPanel>().and_then(|panel| {
            let play = panel.play_ref();
            (!play.is_empty()).then(|| play.widget_uid())
        })
    }

    fn toggle_playback(&mut self, cx: &mut Cx) {
        let playing = self
            .backend
            .as_mut()
            .map(BackendBridge::toggle_playback)
            .unwrap_or(false);
        self.set_status(
            cx,
            if playing {
                "PLAYING  ·  SPACE TO PAUSE"
            } else {
                "PAUSED"
            },
        );
        self.install_timeline_model(cx);
        self.request_stage_frame(cx);
    }

    fn show_panel_error(&self, cx: &mut Cx, text: &str) {
        if let Some(mut panel) = self
            .ui
            .widget(cx, ids!(panel_host))
            .borrow_mut::<HotPanel>()
        {
            panel.install_panel_error(cx, text);
        }
    }

    fn load_panel(&mut self, cx: &mut Cx) {
        let Ok(metadata) = fs::metadata(&self.panel_path) else {
            self.show_panel_error(cx, "panel.splash が無い");
            return;
        };
        let Ok(modified) = metadata.modified() else {
            self.show_panel_error(cx, "panel.splash の更新時刻が取れない");
            return;
        };
        let signature = (modified, metadata.len());
        if self.panel_signature == Some(signature) {
            return;
        }
        // First install is immediate so Dock leaves exist before the first present.
        // Later reloads keep the two-tick debounce so a half-written splash is not applied.
        if self.panel_signature.is_some() && self.pending_signature != Some(signature) {
            self.pending_signature = Some(signature);
            return;
        }

        match fs::read_to_string(&self.panel_path) {
            Ok(source) => {
                let panel = self.ui.widget(cx, ids!(panel_host));
                panel.set_text(cx, &source);
                self.panel_signature = Some(signature);
                self.pending_signature = None;
                let ok = panel
                    .borrow::<HotPanel>()
                    .map(|panel| panel.last_install_ok())
                    .unwrap_or(false);
                if ok {
                    self.install_timeline_model(cx);
                    self.request_stage_frame(cx);
                    log!("reloaded {:?}", self.panel_path);
                }
            }
            Err(error) => {
                self.show_panel_error(cx, &format!("panel.splash を読めない: {error}"));
            }
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.backend = Some(BackendBridge::new_fixture());
        self.panel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("panel.splash");
        self.panel_timer = cx.start_interval(0.12);
        self.playback_timer = cx.start_interval(1.0 / 60.0);
        self.load_panel(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(uid) = self.play_uid(cx) {
            if actions
                .filter_widget_actions(uid)
                .any(|action| matches!(action.cast(), ButtonAction::Clicked { .. }))
            {
                self.toggle_playback(cx);
            }
        }
        let Some(uid) = self.timeline_uid(cx) else {
            return;
        };
        let timeline_actions: Vec<TimelineSurfaceAction> =
            actions.filter_widget_actions_cast(uid).collect();
        for action in timeline_actions {
            let update = self
                .backend
                .as_mut()
                .map(|backend| backend.apply_timeline_action(&action))
                .unwrap_or(TimelineUpdate::None);
            match update {
                TimelineUpdate::None => {}
                TimelineUpdate::Stage(status) => {
                    self.request_stage_frame(cx);
                    self.set_status(cx, &status);
                }
                TimelineUpdate::ModelAndStage(status) => {
                    self.install_timeline_model(cx);
                    self.request_stage_frame(cx);
                    self.set_status(cx, &status);
                }
                TimelineUpdate::Status(status) => {
                    self.set_status(cx, &status);
                }
            }
        }
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.panel_timer.is_timer(event).is_some() {
            self.load_panel(cx);
        }
        if self.playback_timer.is_timer(event).is_some()
            && self
                .backend
                .as_mut()
                .map(BackendBridge::playback_tick)
                .unwrap_or(false)
        {
            self.install_timeline_model(cx);
            self.request_stage_frame(cx);
        }
    }

    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        if event.key_code == KeyCode::Space && !event.is_repeat {
            self.toggle_playback(cx);
        }
    }

    fn handle_next_frame(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        if event.set.contains(&self.stage_next_frame) && self.stage_request.take() {
            self.install_stage_frame(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        crate::browser_surface::script_mod(vm);
        crate::stage_chrome::script_mod(vm);
        crate::inspector_surface::script_mod(vm);
        crate::export_surface::script_mod(vm);
        crate::settings_surface::script_mod(vm);
        crate::timeline_surface::script_mod(vm);
        crate::chrome::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[cfg(test)]
mod backend_tests {
    use super::*;

    #[test]
    fn stage_present_starts_as_named_cpu_fallback() {
        let backend = BackendBridge::new_fixture();
        assert_eq!(backend.present, StagePresent::FallbackCpu);
        assert!(!backend.present.is_zero_copy());
    }

    #[test]
    fn scrub_action_reaches_session() {
        let mut backend = BackendBridge::new_fixture();
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Scrub(600));

        assert!(matches!(update, TimelineUpdate::Stage(_)));
        assert_eq!(backend.session.playhead, 600);
    }

    #[test]
    fn lane_restack_action_changes_document_derived_stage_order() {
        let mut backend = BackendBridge::new_fixture();
        let layer_id = backend
            .timeline_model()
            .lanes
            .last()
            .expect("fixture lane")
            .id;
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Restack {
            layer_id,
            target_from_front: 0,
        });

        assert!(matches!(update, TimelineUpdate::ModelAndStage(_)));
        assert_eq!(backend.timeline_model().lanes[0].id, layer_id);
    }

    #[test]
    fn panel_eval_failure_is_a_visible_sentence() {
        assert_eq!(
            format_panel_eval_errors(&[]),
            "panel.splash を評価できない"
        );
        assert_eq!(
            format_panel_eval_errors(&["kind ChromeGallery is not registered".into()]),
            "panel.splash: kind ChromeGallery is not registered"
        );
    }

    #[test]
    fn stage_requests_coalesce_to_one_latest_delivery() {
        let mut requests = LatestFrameRequest::default();
        assert!(!requests.request(), "first request schedules a consumer");
        assert!(
            requests.request(),
            "later requests reuse the pending consumer"
        );
        assert!(requests.take());
        assert!(
            !requests.take(),
            "one delivery consumes every coalesced request"
        );
    }
}
