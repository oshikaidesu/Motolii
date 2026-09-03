use std::sync::mpsc::{channel, Receiver, Sender};

use crate::ui::keys::{activate_focused_control, aim_keystrokes, commit_field, commit_field_outside, drop_role_at, select_new_field, FIELD};
use blitz_shell::{create_default_event_loop, BlitzShellEvent, BlitzShellProxy, WindowConfig};
use blitz_shell::{BlitzApplication, View};
use dioxus_native::prelude::VirtualDom;
use dioxus_native::prelude::{provide_context, ScopeId};
use dioxus_native::winit::application::ApplicationHandler;
use dioxus_native::winit::event::{
    ButtonSource, ElementState, MouseButton, StartCause, WindowEvent,
};
use dioxus_native::winit::event_loop::ActiveEventLoop;
use dioxus_native::winit::window::{WindowAttributes, WindowId};
use dioxus_native::{
    DioxusDocument, DioxusNativeEvent, DioxusNativeWindowRenderer, RendererOptions,
};

use blitz_dom::{Document, DocumentConfig};
use blitz_traits::net::{NetHandler, NetProvider, Request};

use crate::ui::dock::Panel;
use crate::ui::fixture::{load_fixture, Loaded};
use crate::ui::session::Session;

/// 窓を1枚足してくれ、という頼み。窓の外(event loop)へ渡る。
pub(crate) enum Ask {
    Open(Panel),
}

/// パネルを別窓へ出す口。窓の中のコードはこれしか触らない。
#[derive(Clone)]
pub(crate) struct Host {
    tx: Sender<Ask>,
    proxy: Option<BlitzShellProxy>,
    /// 別窓が閉じた時に本体へ知らせる線。本体が自分の runtime を包んで置く。
    on_close: std::rc::Rc<std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Panel)>>>>,
    on_focus_lost: std::rc::Rc<std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>>>,
    /// 窓ごとに 1 本。別窓で擦って外で放しても、その窓の線が受ける。
    on_primary_pointer_release: std::rc::Rc<
        std::cell::RefCell<Vec<(WindowId, std::rc::Rc<dyn Fn(f64, f64, bool)>)>>,
    >,
    /// 窓を起こす線。窓ごとに1本、自分の runtime を包んで置く。
    /// 状態は全窓で1つなので、誰かが書いたら他の窓も描き直す必要がある。
    wakers: std::rc::Rc<std::cell::RefCell<Vec<(u64, std::rc::Rc<dyn Fn()>)>>>,
    next_id: std::rc::Rc<std::cell::Cell<u64>>,
    /// 人の設定の置き場(配置・preset)。窓を開けない時は無く、何も仕舞わない。
    settings_dir: Option<std::path::PathBuf>,
}

impl PartialEq for Host {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Host {
    /// この窓を起こす線を置く。返るのは自分の番号(自分は起こさないため)。
    pub(crate) fn listen(&self, wake: impl Fn() + 'static) -> u64 {
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        self.wakers.borrow_mut().push((id, std::rc::Rc::new(wake)));
        id
    }

    /// 窓の外で状態が変わった時に、全ての窓を描き直させる。
    pub(crate) fn wake_all(&self) {
        let wakers: Vec<_> = self
            .wakers
            .borrow()
            .iter()
            .map(|(_, w)| w.clone())
            .collect();
        for wake in wakers {
            wake();
        }
    }

    /// 自分以外の窓を描き直させる。
    pub(crate) fn wake_others(&self, me: u64) {
        let wakers: Vec<_> = self
            .wakers
            .borrow()
            .iter()
            .filter(|(id, _)| *id != me)
            .map(|(_, w)| w.clone())
            .collect();
        for wake in wakers {
            wake();
        }
    }

    /// 別窓が閉じた時に呼ばれる物を置く。本体の窓だけが置く。
    pub(crate) fn on_close(&self, f: impl Fn(Panel) + 'static) {
        *self.on_close.borrow_mut() = Some(std::rc::Rc::new(f));
    }

    pub(crate) fn closed(&self, panel: Panel) {
        let f = self.on_close.borrow().clone();
        if let Some(f) = f {
            f(panel);
        }
    }

    pub(crate) fn settings_file(&self, name: &str) -> Option<std::path::PathBuf> {
        self.settings_dir.as_ref().map(|dir| dir.join(name))
    }

    pub(crate) fn on_focus_lost(&self, callback: impl Fn() + 'static) {
        *self.on_focus_lost.borrow_mut() = Some(std::rc::Rc::new(callback));
    }

    pub(crate) fn focus_lost(&self) {
        if let Some(callback) = self.on_focus_lost.borrow().clone() {
            callback();
        }
    }

    /// 放した場所と、それが窓の外かどうか。外なら tab は別窓へ出る。
    pub(crate) fn on_primary_pointer_release(
        &self,
        window: WindowId,
        callback: impl Fn(f64, f64, bool) + 'static,
    ) {
        let mut hooks = self.on_primary_pointer_release.borrow_mut();
        hooks.retain(|(owner, _)| *owner != window);
        hooks.push((window, std::rc::Rc::new(callback)));
    }

    /// 窓の外で放しても届く線。blitz は当たりの無い pointerup を root へ落とし、
    /// `#app` へ下りてこない。窓の shell(winit・harness)がここへ直に配る。
    pub(crate) fn primary_pointer_released(&self, window: WindowId, x: f64, y: f64, outside: bool) {
        let callback = self
            .on_primary_pointer_release
            .borrow()
            .iter()
            .find(|(owner, _)| *owner == window)
            .map(|(_, callback)| callback.clone());
        if let Some(callback) = callback {
            callback(x, y, outside);
        }
    }

    /// 窓を開けずに動かす時の窓 id。harness はこれで窓の持ち主になる。
    pub(crate) const HEADLESS: WindowId = WindowId::from_raw(0);

    /// 窓を開けずに動かす時用。頼みは誰も受け取らない。
    #[cfg(test)]
    pub(crate) fn for_tests() -> Self {
        let (tx, rx) = channel();
        std::mem::forget(rx);
        Self {
            tx,
            proxy: None,
            on_close: Default::default(),
            on_focus_lost: Default::default(),
            on_primary_pointer_release: Default::default(),
            wakers: Default::default(),
            next_id: Default::default(),
            settings_dir: None,
        }
    }

    pub(crate) fn open(&self, panel: Panel) {
        if self.tx.send(Ask::Open(panel)).is_ok() {
            self.poke();
        }
    }

    /// event loop を起こす。中身は使わないが、これで proxy_wake_up が回る。
    pub(crate) fn poke(&self) {
        if let Some(proxy) = &self.proxy {
            proxy.send_event(BlitzShellEvent::embedder_event(Woken));
        }
    }

    /// 別の糸から窓を起こす線。糸をまたぐので proxy だけを渡す。
    pub(crate) fn poker(&self) -> Poke {
        Poke(self.proxy.clone())
    }
}

fn primary_mouse_press(event: &WindowEvent) -> Option<dioxus_native::winit::dpi::PhysicalPosition<f64>> {
    match event {
        WindowEvent::PointerButton {
            state: ElementState::Pressed,
            button: ButtonSource::Mouse(MouseButton::Left),
            primary: true,
            position,
            ..
        } => Some(*position),
        _ => None,
    }
}

/// blitz-shell は IME を能力ゼロで有効にする(`ImeCapabilities::new()`)。winit はその場合、
/// 変換候補の窓を出す位置が無いとして候補を隠す — 日本語の打ち心地が悪い根。欄が居る間は
/// host が `cursor_area` 付きで有効にし直し、候補を欄の箱の位置へ置く。
fn place_ime(view: &mut blitz_shell::View<DioxusNativeWindowRenderer>) {
    use dioxus_native::winit::dpi::{LogicalPosition, LogicalSize};
    use dioxus_native::winit::window::{ImeCapabilities, ImeEnableRequest, ImeRequest, ImeRequestData};
    let area = {
        let doc: &DioxusDocument = view.downcast_doc_mut();
        let inner = doc.inner();
        let Some(field) = inner.query_selector(FIELD).ok().flatten() else {
            drop(inner);
            // 欄が閉じたら IME も閉じる。開いたままだと 1 文字の鍵(m・Space)が変換に吸われる。
            if view.window.ime_capabilities().is_some() {
                let _ = view.window.request_ime_update(ImeRequest::Disable);
            }
            return;
        };
        let Some(node) = inner.get_node(field) else { return };
        let pos = node.absolute_position(0.0, 0.0);
        let layout = node.final_layout();
        (
            pos.x + layout.content_box_x(),
            pos.y + layout.content_box_y(),
            layout.content_box_width(),
            layout.content_box_height(),
        )
    };
    let data = ImeRequestData::default().with_cursor_area(
        LogicalPosition::new(area.0, area.1).into(),
        LogicalSize::new(area.2, area.3).into(),
    );
    let window = &view.window;
    if window.ime_capabilities().is_some_and(|caps| caps.cursor_area()) {
        let _ = window.request_ime_update(ImeRequest::Update(data));
        return;
    }
    let _ = window.request_ime_update(ImeRequest::Disable);
    if let Some(enable) = ImeEnableRequest::new(ImeCapabilities::new().with_cursor_area(), data) {
        let _ = window.request_ime_update(ImeRequest::Enable(enable));
    }
}

fn primary_mouse_release(event: &WindowEvent) -> Option<dioxus_native::winit::dpi::PhysicalPosition<f64>> {
    match event {
        WindowEvent::PointerButton {
            state: ElementState::Released,
            button: ButtonSource::Mouse(MouseButton::Left),
            primary: true,
            position,
            ..
        } => Some(*position),
        _ => None,
    }
}

#[cfg(test)]
#[test]
fn host_routes_only_primary_mouse_release_to_the_owning_window() {
    let host = Host::for_tests();
    let owner = WindowId::from_raw(11);
    let other = WindowId::from_raw(12);
    let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let seen = calls.clone();
    host.on_primary_pointer_release(owner, move |x, y, out| seen.borrow_mut().push((x, y, out)));

    host.primary_pointer_released(other, 1.0, 2.0, false);
    host.primary_pointer_released(owner, -3.0, 4.0, true);

    assert_eq!(&*calls.borrow(), &[(-3.0, 4.0, true)]);
}

/// macOS の Application Support(v1 は macOS だけ、V2-6)。
fn settings_dir() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(std::path::Path::new(&home).join("Library/Application Support/Motolii"))
}

/// 窓の外の糸が持つ、窓を起こすだけの口。
#[derive(Clone)]
pub(crate) struct Poke(Option<BlitzShellProxy>);

impl Poke {
    pub(crate) fn poke(&self) {
        if let Some(proxy) = &self.0 {
            proxy.send_event(BlitzShellEvent::embedder_event(Woken));
        }
    }
}

struct Woken;

struct MotoliiNetProvider {
    fallback: std::sync::Arc<dyn NetProvider>,
}

impl NetProvider for MotoliiNetProvider {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        if request.url.scheme() == "dioxus" {
            if let Ok(response) = dioxus_asset_resolver::native::serve_asset(request.url.path()) {
                handler.bytes(request.url.to_string(), response.into_body().into());
            }
        } else {
            self.fallback.fetch(doc_id, request, handler);
        }
    }
}

fn wakes_shared_state(event: &BlitzShellEvent) -> bool {
    matches!(event, BlitzShellEvent::Embedder(value) if value.is::<Woken>())
}

#[cfg(test)]
#[test]
fn only_explicit_external_state_events_wake_every_window() {
    assert!(wakes_shared_state(&BlitzShellEvent::embedder_event(Woken)));
    assert!(!wakes_shared_state(&BlitzShellEvent::embedder_event(())));
}

fn window(
    root: fn() -> dioxus_native::prelude::Element,
    title: &str,
    size: (u32, u32),
    contexts: Vec<Box<dyn std::any::Any>>,
) -> WindowConfig<DioxusNativeWindowRenderer> {
    let mut vdom = VirtualDom::new(root);
    for context in contexts {
        vdom.insert_any_root_context(context);
    }
    let doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            net_provider: Some(std::sync::Arc::new(MotoliiNetProvider {
                fallback: std::sync::Arc::new(blitz_shell::DataUriNetProvider::new(None)),
            })),
            ..Default::default()
        },
    );
    let renderer = DioxusNativeWindowRenderer::with_options(RendererOptions::default());
    WindowConfig::with_attributes(
        Box::new(doc) as _,
        renderer,
        WindowAttributes::default()
            .with_title(title.to_string())
            .with_surface_size(dioxus_native::winit::dpi::LogicalSize::new(size.0, size.1)),
    )
}



/// 窓を増やせるようにするための薄い包み。頼みを先に食べて、残りは上流へ流す。
struct Windows {
    inner: BlitzApplication<DioxusNativeWindowRenderer>,
    asks: Receiver<Ask>,
    session: Session,
    host: Host,
    pending: Vec<(WindowConfig<DioxusNativeWindowRenderer>, Option<Panel>)>,
    /// どの窓がどのパネルの別窓か。閉じた時に本体へ返すのに要る。
    detached: std::collections::HashMap<WindowId, Panel>,
    /// 最後に指が居た所。摘まみの事象は場所を持たないので、ここから借りる。
    cursor: std::collections::HashMap<WindowId, (f32, f32)>,
    /// 窓ごとに前に見た欄。新しく開いた欄を全選択するための印。
    seen_field: std::collections::HashMap<WindowId, Option<blitz_dom::NodeId>>,
    /// title bar に映した(名前, 編集済み)。同じ物を毎 event 書かない。
    reflected: std::collections::HashMap<WindowId, (String, bool)>,
}

/// 窓1枚を実体にする。`BlitzApplication::add_window` は dioxus の配線
/// (context・最初の build)を通らないので、上流が `pending_window` 1枚へ
/// やっている手順をそのまま踏む。
fn realise(
    app: &mut BlitzApplication<DioxusNativeWindowRenderer>,
    config: WindowConfig<DioxusNativeWindowRenderer>,
    event_loop: &dyn ActiveEventLoop,
) -> WindowId {
    let proxy = app.proxy.clone();
    let mut view = View::init(config, event_loop, &proxy);
    let winit_window = std::sync::Arc::clone(&view.window);
    let renderer = view.renderer.clone();
    let window_id = view.window_id();
    let doc = view.downcast_doc_mut::<DioxusDocument>();

    let shell_provider = doc.inner.borrow().shell_provider.clone();
    doc.vdom.in_scope(ScopeId::ROOT, move || {
        provide_context(shell_provider);
        provide_context(renderer);
        provide_context(winit_window);
    });
    doc.initial_build();
    // 作った直後の winit は macOS で is_visible() = false を返すことがある。
    // View はそれを掴んだまま描画を止めるので、見えている前提で建てる。
    view.is_visible = true;
    view.resume();
    view.request_redraw();
    app.windows.insert(window_id, view);
    window_id
}

impl Windows {

    /// 閉じる時の Save。行き先が無ければ同期の panel で聞く(event の中なので async は使えない)。
    fn save_now(&self) -> bool {
        let known = self.session.project_path.lock().unwrap().clone();
        let out = match known {
            Some(p) => p,
            None => {
                let Some(mut p) = rfd::FileDialog::new()
                    .add_filter("Motolii", &["rrd"])
                    .set_file_name("song.rrd")
                    .save_file()
                else {
                    return false;
                };
                if p.extension().is_none_or(|e| !e.eq_ignore_ascii_case("rrd")) {
                    p.set_extension("rrd");
                }
                p
            }
        };
        let saved = {
            let d = self.session.doc.lock().unwrap();
            d.save(&out).map(|()| d.revision())
        };
        match saved {
            Ok(rev) => {
                self.session.mark_saved(out, rev);
                true
            }
            Err(e) => {
                *self.session.project_notice.lock().unwrap() = format!("Save failed: {e}");
                false
            }
        }
    }

    /// title bar が書類を指す: 名前と、編集済みの●(macOS)。変わった時だけ触る。
    fn reflect_document(&mut self, window_id: WindowId) {
        if self.detached.contains_key(&window_id) {
            return;
        }
        let title = self.session.document_title();
        let dirty = self.session.is_dirty();
        if self.reflected.get(&window_id) == Some(&(title.clone(), dirty)) {
            return;
        }
        if let Some(view) = self.inner.windows.get(&window_id) {
            view.window.set_title(&title);
            #[cfg(target_os = "macos")]
            {
                use dioxus_native::winit::platform::macos::WindowExtMacOS;
                view.window.set_document_edited(dirty);
            }
        }
        self.reflected.insert(window_id, (title, dirty));
    }

    fn handle_native_event(&mut self, event_loop: &dyn ActiveEventLoop, event: &DioxusNativeEvent) {
        match event {
            #[cfg(debug_assertions)]
            DioxusNativeEvent::DevserverEvent(event) => match event {
                dioxus_devtools::DevserverMsg::HotReload(message) => {
                    for (index, window) in self.inner.windows.values_mut().enumerate() {
                        let doc = window.downcast_doc_mut::<DioxusDocument>();
                        dioxus_devtools::apply_changes(&doc.vdom, message);
                        for asset in &message.assets {
                            if let Some(url) = asset.to_str() {
                                doc.inner.borrow_mut().reload_resource_by_href(url);
                            }
                        }
                        println!("PROBE room=reload verdict=applied window={index}");
                        window.poll();
                    }
                }
                dioxus_devtools::DevserverMsg::Shutdown => event_loop.exit(),
                _ => {}
            },
            DioxusNativeEvent::CreateHeadElement {
                name,
                attributes,
                contents,
                window,
            } => {
                if let Some(view) = self.inner.windows.get_mut(window) {
                    view.downcast_doc_mut::<DioxusDocument>()
                        .create_head_element(name, attributes, contents);
                    view.poll();
                }
            }
        }
    }

    fn realise_pending(&mut self, event_loop: &dyn ActiveEventLoop) {
        for (config, panel) in std::mem::take(&mut self.pending) {
            let id = realise(&mut self.inner, config, event_loop);
            if let Some(panel) = panel {
                self.detached.insert(id, panel);
            }
        }
    }

    fn serve_asks(&mut self) {
        while let Ok(ask) = self.asks.try_recv() {
            match ask {
                Ask::Open(panel) => self.pending.push((
                    window(
                        crate::ui::app::detached,
                        panel.label(),
                        panel.window_size(),
                        vec![
                            Box::new(self.session.clone()),
                            Box::new(self.host.clone()),
                            Box::new(panel),
                        ],
                    ),
                    Some(panel),
                )),
            }
        }
    }
}

impl ApplicationHandler for Windows {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.destroy_surfaces(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.realise_pending(event_loop);
        self.inner.can_create_surfaces(event_loop);
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.session.quit.load(std::sync::atomic::Ordering::Relaxed) {
            event_loop.exit();
            return;
        }
        // 打鍵は焦点のある節へ配られ、無ければ `<html>` へ行く。`<html>` からは
        // 下りてこないので、当て直さないと**打鍵が1つも届かない**。
        // 上流の autofocus は属性が付く前に可否を見ていて効かない。
        // 打ち込み中の欄が在る時はそちらへ当てる —— 根へ引き戻すと入力欄に
        // 文字が1つも入らない。窓に開く欄は同時に1つだけ。
        if let Some(view) = self.inner.windows.get_mut(&window_id) {
            let seen = self.seen_field.entry(window_id).or_default();
            select_new_field(view.downcast_doc_mut::<DioxusDocument>(), seen);
        }
        if let WindowEvent::KeyboardInput { event: key, .. } = &event {
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                let doc = view.downcast_doc_mut::<DioxusDocument>();
                aim_keystrokes(doc);
                if key.state.is_pressed() {
                    let logical = keyboard_types::Key::Character(key.text.as_deref().unwrap_or("").to_string());
                    let is_enter = matches!(key.logical_key, dioxus_native::winit::keyboard::Key::Named(dioxus_native::winit::keyboard::NamedKey::Enter));
                    let k = if is_enter { keyboard_types::Key::Enter } else { logical };
                    if activate_focused_control(doc, &k) {
                        return;
                    }
                }
            }
        }
        if let Some(position) = primary_mouse_press(&event) {
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                let coords = view.pointer_coords(position);
                commit_field_outside(
                    view.downcast_doc_mut::<DioxusDocument>(),
                    coords.client_x,
                    coords.client_y,
                );
            }
        }
        if let WindowEvent::PointerMoved { position, .. } = &event {
            self.cursor
                .insert(window_id, (position.x as f32, position.y as f32));
        }
        if let Some(position) = primary_mouse_release(&event) {
            if let Some(view) = self.inner.windows.get(&window_id) {
                let coords = view.pointer_coords(position);
                let size = view.window.surface_size();
                let outside = position.x < 0.0
                    || position.y < 0.0
                    || position.x > f64::from(size.width)
                    || position.y > f64::from(size.height);
                self.host.primary_pointer_released(
                    window_id,
                    f64::from(coords.client_x),
                    f64::from(coords.client_y),
                    outside,
                );
            }
        }
        // トラックパッドの摘まみ。blitz の `UiEvent` に摘まみは無いので、
        // **Ctrl+ホイールへ翻訳**して配る(ブラウザと同じ作法で、盤も窓も
        // 既にその形で拡縮する)。
        if let WindowEvent::PinchGesture { delta, .. } = &event {
            if delta.is_finite() && *delta != 0.0 {
                let (x, y) = self.cursor.get(&window_id).copied().unwrap_or((0.0, 0.0));
                if let Some(view) = self.inner.windows.get_mut(&window_id) {
                    let doc = view.downcast_doc_mut::<DioxusDocument>();
                    doc.handle_ui_event(blitz_traits::events::UiEvent::Wheel(
                        blitz_traits::events::BlitzWheelEvent {
                            delta: blitz_traits::events::BlitzWheelDelta::Pixels(
                                0.0,
                                *delta * 400.0,
                            ),
                            coords: blitz_traits::events::PointerCoords {
                                page_x: x,
                                page_y: y,
                                screen_x: x,
                                screen_y: y,
                                client_x: x,
                                client_y: y,
                            },
                            buttons: blitz_traits::events::MouseEventButtons::None,
                            mods: keyboard_types::Modifiers::CONTROL,
                            element: blitz_traits::events::Point { x, y },
                        },
                    ));
                    view.request_redraw();
                }
            }
            return;
        }
        if let WindowEvent::DragEntered { paths, .. } = &event {
            self.host.focus_lost();
            self.session.file_drop.enter(paths);
            self.host.wake_all();
        }
        if matches!(event, WindowEvent::DragLeft { .. }) {
            self.session.file_drop.leave();
            self.host.wake_all();
        }
        if let WindowEvent::DragDropped { paths, position } = &event {
            self.session.file_drop.leave();
            let role = self
                .inner
                .windows
                .get_mut(&window_id)
                .map(|view| {
                    let coords = view.pointer_coords(*position);
                    let doc: &DioxusDocument = view.downcast_doc_mut();
                    drop_role_at(doc, coords.client_x, coords.client_y)
                })
                .unwrap_or_default();
            let summary = {
                let mut doc = self.session.doc.lock().unwrap();
                crate::ui::fixture::admit_paths(&mut doc, paths, role)
            };
            println!(
                "PROBE room=browser verdict=drop admitted={} of={}",
                summary.admitted, summary.total
            );
            *self.session.project_notice.lock().unwrap() = summary.notice();
            self.host.wake_all();
        }
        if matches!(event, WindowEvent::CloseRequested) {
            if !self.detached.contains_key(&window_id) && self.session.is_dirty() {
                // Mac の書類: Save / Don't Save / Cancel、既定は Save(HIG Alerts、TextEdit と同じ文面)。
                let name = self.session.document_title();
                let mut dialog = rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Warning)
                    .set_title(format!("Do you want to save the changes you made to {name}?"))
                    .set_description("Your changes will be lost if you don't save them.")
                    .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
                        "Save".to_owned(),
                        "Don't Save".to_owned(),
                        "Cancel".to_owned(),
                    ));
                if let Some(view) = self.inner.windows.get(&window_id) {
                    dialog = dialog.set_parent(&*view.window);
                }
                match dialog.show() {
                    rfd::MessageDialogResult::Custom(label) if label == "Save" => {
                        if !self.save_now() {
                            return;
                        }
                    }
                    rfd::MessageDialogResult::Custom(label) if label == "Don't Save" => {}
                    _ => return,
                }
            }
            if let Some(panel) = self.detached.remove(&window_id) {
                self.host.closed(panel);
            } else {
                // 主窓を閉じたら終わる。panel の別窓だけを残さない(Mac の document app)。
                event_loop.exit();
                return;
            }
        }
        if matches!(event, WindowEvent::Focused(false)) {
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                commit_field(view.downcast_doc_mut::<DioxusDocument>());
            }
            self.host.focus_lost();
        }
        self.inner.window_event(event_loop, window_id, event);
        if let Some(view) = self.inner.windows.get_mut(&window_id) {
            place_ime(view);
        }
        self.reflect_document(window_id);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut wake_shared = false;
        while let Ok(event) = self.inner.event_queue.try_recv() {
            let event_wakes_shared = wakes_shared_state(&event);
            match event {
                BlitzShellEvent::Embedder(event) => {
                    if let Some(event) = event.downcast_ref::<DioxusNativeEvent>() {
                        self.handle_native_event(event_loop, event);
                    } else if event_wakes_shared {
                        wake_shared = true;
                    }
                }
                event => self.inner.handle_blitz_shell_event(event_loop, event),
            }
        }
        if wake_shared {
            self.host.wake_all();
        }
        self.serve_asks();
        self.realise_pending(event_loop);
    }
}

pub fn launch(title: &str) {
    let event_loop = create_default_event_loop();
    let (proxy, event_queue) = BlitzShellProxy::new(event_loop.create_proxy());

    #[cfg(debug_assertions)]
    {
        let proxy = proxy.clone();
        dioxus_devtools::connect(move |event| {
            proxy.send_event(BlitzShellEvent::embedder_event(
                DioxusNativeEvent::DevserverEvent(event),
            ));
        });
    }

    let Loaded {
        doc,
        ui,
        duration_sec,
    } = load_fixture();
    // 普通のソフトは白紙で起動する。見本の作品は MOTOLII_FIXTURE=1 の時だけ(試験と実窓の検分用)。
    let doc = if std::env::var_os("MOTOLII_FIXTURE").is_some() { doc } else { crate::ui::blank_project() };
    let session = Session::new(doc, duration_sec, ui);
    let (tx, asks) = channel();
    let host = Host {
        tx,
        proxy: Some(proxy.clone()),
        on_close: Default::default(),
        on_focus_lost: Default::default(),
        on_primary_pointer_release: Default::default(),
        wakers: Default::default(),
        next_id: Default::default(),
        settings_dir: settings_dir(),
    };

    let main = window(
        crate::ui::app::app,
        title,
        (1600, 1000),
        vec![Box::new(session.clone()), Box::new(host.clone())],
    );
    let inner = BlitzApplication::new(proxy, event_queue);

    event_loop
        .run_app(Windows {
            inner,
            asks,
            session,
            host,
            pending: vec![(main, None)],
            detached: Default::default(),
            cursor: Default::default(),
            seen_field: Default::default(),
            reflected: Default::default(),
        })
        .unwrap();
}
