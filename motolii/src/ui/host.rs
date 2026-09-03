use std::sync::mpsc::{channel, Receiver, Sender};

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
    on_primary_pointer_release: std::rc::Rc<
        std::cell::RefCell<Option<(WindowId, std::rc::Rc<dyn Fn(f64, f64, bool)>)>>,
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
        *self.on_primary_pointer_release.borrow_mut() =
            Some((window, std::rc::Rc::new(callback)));
    }

    /// 窓の外で放しても届く線。blitz は当たりの無い pointerup を root へ落とし、
    /// `#app` へ下りてこない。窓の shell(winit・harness)がここへ直に配る。
    pub(crate) fn primary_pointer_released(&self, window: WindowId, x: f64, y: f64, outside: bool) {
        let callback = self
            .on_primary_pointer_release
            .borrow()
            .as_ref()
            .filter(|(owner, _)| *owner == window)
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
        let Some(field) = inner.query_selector(FIELD).ok().flatten() else { return };
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

/// 落とした先が机なら参考画像、他は素材。落とす口は 1 つで、役目だけが場所で決まる。
pub(crate) fn drop_role_at(doc: &DioxusDocument, x: f32, y: f32) -> crate::doc::store::AssetRole {
    let inner = doc.inner();
    let desk = inner.query_selector("#desk").ok().flatten();
    let mut cur = inner.hit(x, y).map(|hit| hit.node_id);
    while let (Some(node), Some(desk)) = (cur, desk) {
        if node == desk {
            return crate::doc::store::AssetRole::Reference;
        }
        cur = inner.get_node(node).and_then(|n| n.parent);
    }
    crate::doc::store::AssetRole::Material
}

/// 窓に開く欄。1 行は `input`、書き置きは `textarea`。同時に 1 つ(`Session.field`)。
const FIELD: &str = "input, textarea";

/// 欄は許可制。押すまで無く、Enter・Escape・**外を押す**のどれでも欄ごと消える。
/// 欄を持つ面がそれぞれ閉じ方を書くのではなく、外を押した時は欄へ Cmd+Enter を送る
/// (書き置きは Enter が改行なので、確定は Cmd 付き)。
/// 窓に欄が在る間は打鍵が全部そこへ行く(`aim_keystrokes`)ので、閉じ損ねは鍵の全喪失になる。
pub(crate) fn commit_field_outside(doc: &mut DioxusDocument, x: f32, y: f32) {
    let Some(field) = doc.inner().query_selector(FIELD).ok().flatten() else { return };
    if doc.inner().hit(x, y).is_some_and(|hit| hit.node_id == field) {
        return;
    }
    commit_field(doc);
}

/// 窓を離れる時も欄は確定して消える(§6b)。
pub(crate) fn commit_field(doc: &mut DioxusDocument) {
    if doc.inner().query_selector(FIELD).ok().flatten().is_none() {
        return;
    }
    aim_keystrokes(doc);
    send_chord(doc, keyboard_types::Key::Enter, keyboard_types::Code::Enter);
}

/// 開いたばかりの欄: 1 行(input)は全選択 — 打てば置き換わる(Finder・AE の名前と同じ)。
/// 書き置き(textarea)は caret を末尾へ。blitz は欄の editor を node より後に作り、その時
/// caret を先頭に置くので、editor が出来るまでは何もせず次の event で再び見る。
pub(crate) fn select_new_field(doc: &mut DioxusDocument, seen: &mut Option<blitz_dom::NodeId>) {
    let field = doc.inner().query_selector(FIELD).ok().flatten();
    if field == *seen {
        return;
    }
    let Some(node) = field else {
        *seen = None;
        return;
    };
    let ready = {
        let inner = doc.inner();
        inner.get_node(node).and_then(|n| {
            let element = n.element_data()?;
            element.text_input_data()?;
            Some(element.name.local.as_ref() == "textarea")
        })
    };
    let Some(multiline) = ready else { return };
    *seen = field;
    aim_keystrokes(doc);
    if multiline {
        send_chord(doc, keyboard_types::Key::End, keyboard_types::Code::End);
    } else {
        send_chord(doc, keyboard_types::Key::Character("a".into()), keyboard_types::Code::KeyA);
    }
}

fn send_chord(doc: &mut DioxusDocument, key: keyboard_types::Key, code: keyboard_types::Code) {
    let event = |state| blitz_traits::events::BlitzKeyEvent {
        key: key.clone(),
        code,
        modifiers: keyboard_types::Modifiers::SUPER,
        location: keyboard_types::Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state,
        text: None,
    };
    doc.handle_ui_event(blitz_traits::events::UiEvent::KeyDown(event(
        blitz_traits::events::KeyState::Pressed,
    )));
    doc.handle_ui_event(blitz_traits::events::UiEvent::KeyUp(event(
        blitz_traits::events::KeyState::Released,
    )));
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

/// macOS の Application Support。他 OS は v1 の対象外(V2-6)なので HOME 直下。
fn settings_dir() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    let base = if cfg!(target_os = "macos") {
        std::path::Path::new(&home).join("Library/Application Support")
    } else {
        std::path::Path::new(&home).join(".config")
    };
    Some(base.join("Motolii"))
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

/// 打鍵をどこへ配るかを決める。**窓の側と試験の側で同じ規則を通す** —— 分けると、
/// 利用者が歩く道(欄を開けて打つ)の試験が書けない。
pub(crate) fn aim_keystrokes(doc: &mut DioxusDocument) {
    let field = doc.inner().query_selector(FIELD).ok().flatten();
    crate::ui::keymap::set_typing(field.is_some());
    let target = {
        let inner = doc.inner();
        field.or_else(|| {
            ["#app", "#detached"]
                .into_iter()
                .find_map(|s| inner.query_selector(s).ok().flatten())
        })
    };
    let Some(target) = target else { return };
    let mut inner = doc.inner_mut();
    if inner.get_focussed_node_id() != Some(target) {
        inner.set_focus_to(target);
    }
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
        // 打鍵は焦点のある節へ配られ、無ければ `<html>` へ行く。`<html>` からは
        // 下りてこないので、当て直さないと**打鍵が1つも届かない**。
        // 上流の autofocus は属性が付く前に可否を見ていて効かない。
        // 打ち込み中の欄が在る時はそちらへ当てる —— 根へ引き戻すと入力欄に
        // 文字が1つも入らない。窓に開く欄は同時に1つだけ。
        if let Some(view) = self.inner.windows.get_mut(&window_id) {
            let seen = self.seen_field.entry(window_id).or_default();
            select_new_field(view.downcast_doc_mut::<DioxusDocument>(), seen);
        }
        if matches!(event, WindowEvent::KeyboardInput { .. }) {
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                aim_keystrokes(view.downcast_doc_mut::<DioxusDocument>());
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
                let answer = rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Warning)
                    .set_title("Unsaved changes")
                    .set_description("Close this project without saving your changes?")
                    .set_buttons(rfd::MessageButtons::OkCancelCustom(
                        "Close Without Saving".to_owned(),
                        "Cancel".to_owned(),
                    ))
                    .show();
                if answer != rfd::MessageDialogResult::Custom("Close Without Saving".to_owned()) {
                    return;
                }
            }
            if let Some(panel) = self.detached.remove(&window_id) {
                self.host.closed(panel);
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
        })
        .unwrap();
}
