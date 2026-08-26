pub use makepad_widgets;

use makepad_widgets::*;
use motolii_shell::{Message, Shell};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

mod gesture_input;
mod timeline_surface;
use timeline_surface::{
    TimelineLane, TimelineModel, TimelinePropertyLane, TimelineSurface, TimelineSurfaceAction,
};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.HotPanelBase = #(HotPanel::register_widget(vm))
    mod.widgets.TimelineSurfaceBase = #(TimelineSurface::register_widget(vm))
    mod.widgets.TimelineSurface = set_type_default() do mod.widgets.TimelineSurfaceBase{
        width: Fill
        height: Fill
        draw_bg +: {color: #x363636}
        draw_item +: {color: #xffffff}
        draw_text +: {
            color: #xb8b8b8
            text_style: theme.font_code{font_size: 8}
        }
    }
    mod.widgets.HotPanel = set_type_default() do mod.widgets.HotPanelBase{
        width: Fill
        height: Fill
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
                    }
                }
            }
        }
    }
}

const HOT_PANEL_PREFIX: &str =
    "use mod.prelude.widgets.*\nuse mod.widgets.*\nView{width:Fill height:Fill flow:Down, ";

/// Makepad is an external Elm view adapter. This is the only translation point
/// between Makepad actions and the existing Iced-era `Shell::update(Message)`
/// core; widgets never write Document/Session state directly.
struct BackendBridge {
    shell: Shell,
}

enum TimelineUpdate {
    None,
    Stage(String),
    ModelAndStage(String),
    Status(String),
}

/// A one-slot mailbox for expensive projections. Producers may run at pointer
/// frequency; the consumer runs at display frequency and only observes the
/// newest request. The payload can later become an IOSurface handle without
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
        let (shell, _startup_task) = Shell::new_fixture();
        Self { shell }
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
        let mut rows = self.shell.timeline_rows();
        let store = self.shell.store_view();
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
        let property_lanes = self
            .shell
            .timeline_property_rows()
            .into_iter()
            .map(|row| TimelinePropertyLane {
                layer_id: row.layer.0,
                name: format!("> {}", row.property.name()),
                keys: row.keys.into_iter().map(|key| key.frame).collect(),
            })
            .collect();
        let composition = self.shell.composition();
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
            playhead: self.shell.session().playhead,
            fps_num,
            fps_den,
        }
    }

    fn scrub_to(&mut self, frame: i64) {
        let _ = self.shell.update(Message::ScrubTo(frame));
    }

    fn restack_from_timeline(&mut self, layer_id: u64, target_from_front: usize) -> String {
        let Some(layer) = self
            .shell
            .timeline_rows()
            .into_iter()
            .find(|row| row.id.0 == layer_id)
            .map(|row| row.id)
        else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        let layer_count = self.shell.store_view().layers().len();
        let target_from_front = target_from_front.min(layer_count.saturating_sub(1));
        let target_from_back = layer_count
            .saturating_sub(1)
            .saturating_sub(target_from_front);

        // Selection is UI/session state. Restack is a single Document apply_all,
        // so one completed lane drag creates exactly one undo step.
        let _ = self.shell.update(Message::Select(layer));
        let _ = self.shell.update(Message::Timeline(
            motolii_shell::timeline_pane::Message::RestackLayer(
                motolii_shell::timeline::StackDirection::ToIndexFromBack(target_from_back),
            ),
        ));
        self.shell
            .status()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "Timeline: layer {} moved to lane {} / Stage stack updated",
                    layer_id,
                    target_from_front + 1
                )
            })
    }

    fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        self.shell.frame_rgba()
    }

    fn toggle_playback(&mut self) -> bool {
        let _ = self.shell.update(Message::TogglePlayback);
        self.shell.is_playing()
    }

    fn playback_tick(&mut self) -> bool {
        if !self.shell.is_playing() {
            return false;
        }
        let _ = self.shell.update(Message::PlaybackTick);
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
        if self.body.as_ref() == value {
            log!("panel source unchanged; keeping installed view");
            return;
        }

        let code = format!("{}{}", HOT_PANEL_PREFIX, value);
        let file = "panel.splash".to_string();
        let next_view = cx.with_vm(|vm| {
            let script_mod = ScriptMod {
                cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
                module_path: "r7_makepad_panel::hot_panel".to_string(),
                file,
                line: 0,
                column: 0,
                code: String::new(),
                values: vec![],
            };
            let value = vm.eval_with_append_source(script_mod, &code, NIL.into());
            let errors = vm.take_errors();
            if value.is_err() || !errors.is_empty() {
                log!("panel evaluation failed: {:?}", errors);
                None
            } else {
                Some(View::script_from_value(vm, value))
            }
        });

        if let Some(view) = next_view {
            self.body.set(value);
            self.view = view;
            // The initial placeholder has no useful draw area yet. Redrawing only
            // this widget can therefore miss the first frame after replacement.
            cx.redraw_all();
            log!("panel view installed: {} bytes", value.len());
        }
    }
}

impl HotPanel {
    fn set_stage_texture(&mut self, cx: &mut Cx, texture: Texture) -> bool {
        let stage_image = self.view.child_by_path(ids!(stage_frame)).as_image();
        let found = !stage_image.is_empty();
        stage_image.set_texture(cx, Some(texture));
        found
    }

    fn timeline_ref(&self) -> WidgetRef {
        self.view.child_by_path(ids!(timeline_surface))
    }

    fn play_ref(&self) -> WidgetRef {
        self.view.child_by_path(ids!(play_toggle))
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
        let Some(backend) = self.backend.as_mut() else {
            return;
        };
        let Some((width, height, rgba)) = backend.frame_rgba() else {
            log!("Stage bridge: backend produced no frame");
            return;
        };
        let Ok(image) = ImageBuffer::new(rgba, width as usize, height as usize) else {
            log!("Stage bridge: invalid RGBA frame {}x{}", width, height);
            return;
        };
        let texture = image.into_new_texture(cx);
        // Enter the runtime host first; the startup tree cannot index descendants
        // that did not exist until panel.splash was evaluated.
        let panel = self.ui.widget(cx, ids!(panel_host));
        let image_found = panel
            .borrow_mut::<HotPanel>()
            .map(|mut panel| panel.set_stage_texture(cx, texture))
            .unwrap_or(false);
        panel.redraw(cx);
        log!(
            "Stage bridge: installed latest re_renderer frame {}x{} image_found={}",
            width,
            height,
            image_found
        );
    }

    fn request_stage_frame(&mut self, cx: &mut Cx) {
        if !self.stage_request.request() {
            self.stage_next_frame = cx.new_next_frame();
        }
    }

    fn install_timeline_model(&mut self, cx: &mut Cx) {
        let Some(model) = self.backend.as_ref().map(BackendBridge::timeline_model) else {
            return;
        };
        let panel = self.ui.widget(cx, ids!(panel_host));
        let timeline_found = panel
            .borrow_mut::<HotPanel>()
            .map(|mut panel| panel.set_timeline_model(cx, model))
            .unwrap_or(false);
        log!(
            "Timeline bridge: model installed timeline_found={}",
            timeline_found
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

    fn load_panel(&mut self, cx: &mut Cx) {
        let Ok(metadata) = fs::metadata(&self.panel_path) else {
            log!("panel file missing: {:?}", self.panel_path);
            return;
        };
        let Ok(modified) = metadata.modified() else {
            return;
        };
        let signature = (modified, metadata.len());
        if self.panel_signature == Some(signature) {
            return;
        }
        if self.pending_signature != Some(signature) {
            self.pending_signature = Some(signature);
            return;
        }

        match fs::read_to_string(&self.panel_path) {
            Ok(source) => {
                self.ui.widget(cx, ids!(panel_host)).set_text(cx, &source);
                self.panel_signature = Some(signature);
                self.pending_signature = None;
                self.install_timeline_model(cx);
                self.request_stage_frame(cx);
                log!("reloaded {:?}", self.panel_path);
            }
            Err(error) => log!("panel read failed: {:?}", error),
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
    fn scrub_action_reaches_shell_session() {
        let mut backend = BackendBridge::new_fixture();
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Scrub(600));

        assert!(matches!(update, TimelineUpdate::Stage(_)));
        assert_eq!(backend.shell.session().playhead, 600);
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
