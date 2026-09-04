use std::sync::mpsc::{channel, Receiver, Sender};

use crate::ui::keys::{
    activate_focused_control, aim_keystrokes, commit_field, commit_field_outside, drop_role_at,
    select_new_field, FIELD,
};
use blitz_shell::{create_default_event_loop, BlitzShellEvent, BlitzShellProxy, WindowConfig};
use blitz_shell::{BlitzApplication, View};
use dioxus_native::prelude::VirtualDom;
use dioxus_native::prelude::{dioxus_core, dioxus_signals, provide_context, ScopeId, Signal};
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
    pub(crate) mounts: crate::ui::mount::MountStore,
    patch_epoch: std::rc::Rc<std::cell::Cell<u64>>,
    last_patch: std::rc::Rc<std::cell::RefCell<Option<std::path::PathBuf>>>,
    tx: Sender<Ask>,
    proxy: Option<BlitzShellProxy>,
    /// 別窓が閉じた時に本体へ知らせる線。本体が自分の runtime を包んで置く。
    on_close: std::rc::Rc<std::cell::RefCell<Option<(u64, std::rc::Rc<dyn Fn(Panel)>)>>>,
    on_focus_lost: std::rc::Rc<std::cell::RefCell<Vec<(WindowId, u64, std::rc::Rc<dyn Fn()>)>>>,
    /// 窓ごとに 1 本。別窓で擦って外で放しても、その窓の線が受ける。
    on_primary_pointer_release:
        std::rc::Rc<std::cell::RefCell<Vec<(WindowId, u64, std::rc::Rc<dyn Fn(f64, f64, bool)>)>>>,
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

    pub(crate) fn unlisten(&self, id: u64) {
        self.wakers.borrow_mut().retain(|(owner, _)| *owner != id);
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
    pub(crate) fn on_close(&self, f: impl Fn(Panel) + 'static) -> u64 {
        let id = self.new_callback_id();
        *self.on_close.borrow_mut() = Some((id, std::rc::Rc::new(f)));
        id
    }

    pub(crate) fn closed(&self, panel: Panel) {
        let f = self.on_close.borrow().clone();
        if let Some((_, f)) = f {
            f(panel);
        }
    }

    pub(crate) fn settings_file(&self, name: &str) -> Option<std::path::PathBuf> {
        self.settings_dir.as_ref().map(|dir| dir.join(name))
    }

    pub(crate) fn on_focus_lost(&self, window: WindowId, callback: impl Fn() + 'static) -> u64 {
        let id = self.new_callback_id();
        let mut callbacks = self.on_focus_lost.borrow_mut();
        callbacks.retain(|(owner, _, _)| *owner != window);
        callbacks.push((window, id, std::rc::Rc::new(callback)));
        id
    }

    pub(crate) fn focus_lost(&self) {
        let callbacks = self.on_focus_lost.borrow().clone();
        for (_, _, callback) in callbacks {
            callback();
        }
    }

    /// 放した場所と、それが窓の外かどうか。外なら tab は別窓へ出る。
    pub(crate) fn on_primary_pointer_release(
        &self,
        window: WindowId,
        callback: impl Fn(f64, f64, bool) + 'static,
    ) -> u64 {
        let id = self.new_callback_id();
        let mut hooks = self.on_primary_pointer_release.borrow_mut();
        hooks.retain(|(owner, _, _)| *owner != window);
        hooks.push((window, id, std::rc::Rc::new(callback)));
        id
    }

    fn new_callback_id(&self) -> u64 {
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        id
    }

    pub(crate) fn patch_epoch(&self) -> u64 {
        self.patch_epoch.get()
    }

    pub(crate) fn remove_callback(&self, id: u64) {
        let remove_close = self
            .on_close
            .borrow()
            .as_ref()
            .is_some_and(|(token, _)| *token == id);
        if remove_close {
            self.on_close.borrow_mut().take();
        }
        self.on_focus_lost
            .borrow_mut()
            .retain(|(_, token, _)| *token != id);
        self.on_primary_pointer_release
            .borrow_mut()
            .retain(|(_, token, _)| *token != id);
        self.unlisten(id);
    }

    fn retire_window_callbacks(&self, window: WindowId) -> usize {
        let focus = {
            let mut callbacks = self.on_focus_lost.borrow_mut();
            let mut retired = Vec::new();
            for index in (0..callbacks.len()).rev() {
                if callbacks[index].0 == window {
                    retired.push(callbacks.remove(index));
                }
            }
            retired
        };
        let release = {
            let mut callbacks = self.on_primary_pointer_release.borrow_mut();
            let mut retired = Vec::new();
            for index in (0..callbacks.len()).rev() {
                if callbacks[index].0 == window {
                    retired.push(callbacks.remove(index));
                }
            }
            retired
        };
        let count = focus.len() + release.len();
        drop((focus, release));
        count
    }

    fn clear_scope_callbacks(&self) {
        let close = self.on_close.borrow_mut().take();
        let focus = std::mem::take(&mut *self.on_focus_lost.borrow_mut());
        let release = std::mem::take(&mut *self.on_primary_pointer_release.borrow_mut());
        let wakers = std::mem::take(&mut *self.wakers.borrow_mut());
        drop((close, focus, release, wakers));
    }

    /// 窓の外で放しても届く線。blitz は当たりの無い pointerup を root へ落とし、
    /// `#app` へ下りてこない。窓の shell(winit・harness)がここへ直に配る。
    pub(crate) fn primary_pointer_released(&self, window: WindowId, x: f64, y: f64, outside: bool) {
        let callback = self
            .on_primary_pointer_release
            .borrow()
            .iter()
            .find(|(owner, _, _)| *owner == window)
            .map(|(_, _, callback)| callback.clone());
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
            mounts: Default::default(),
            patch_epoch: Default::default(),
            last_patch: Default::default(),
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

fn primary_mouse_press(
    event: &WindowEvent,
) -> Option<dioxus_native::winit::dpi::PhysicalPosition<f64>> {
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
    use dioxus_native::winit::window::{
        ImeCapabilities, ImeEnableRequest, ImeRequest, ImeRequestData,
    };
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
        let Some(node) = inner.get_node(field) else {
            return;
        };
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
    if window
        .ime_capabilities()
        .is_some_and(|caps| caps.cursor_area())
    {
        let _ = window.request_ime_update(ImeRequest::Update(data));
        return;
    }
    let _ = window.request_ime_update(ImeRequest::Disable);
    if let Some(enable) = ImeEnableRequest::new(ImeCapabilities::new().with_cursor_area(), data) {
        let _ = window.request_ime_update(ImeRequest::Enable(enable));
    }
}

fn primary_mouse_release(
    event: &WindowEvent,
) -> Option<dioxus_native::winit::dpi::PhysicalPosition<f64>> {
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

#[cfg(any(debug_assertions, test))]
fn carry_patch_aliases(
    incoming: &mut dioxus_devtools::subsecond::JumpTable,
    previous: &dioxus_devtools::subsecond::JumpTable,
    runtime_reference: u64,
) -> Result<usize, String> {
    let slide = runtime_reference
        .checked_sub(incoming.aslr_reference)
        .ok_or("Reload address is below the baseline ASLR reference")?;
    let mut aliases_by_target = std::collections::HashMap::<u64, Vec<u64>>::new();
    for (&address, &target) in &previous.map {
        aliases_by_target.entry(target).or_default().push(address);
    }
    for aliases in aliases_by_target.values_mut() {
        aliases.sort_unstable();
    }
    let mut originals = incoming
        .map
        .iter()
        .map(|(&address, &target)| (address, target))
        .collect::<Vec<_>>();
    originals.sort_unstable();
    let mut expanded = incoming.map.clone();
    for (baseline, next) in originals {
        let address = baseline
            .checked_add(slide)
            .ok_or("Reload baseline address overflow")?;
        let Some(&previous_target) = previous.map.get(&address) else {
            continue;
        };
        for old in std::iter::once(previous_target)
            .chain(aliases_by_target[&previous_target].iter().copied())
        {
            let key = old
                .checked_sub(slide)
                .ok_or("Reload alias is below the baseline ASLR slide")?;
            match expanded.entry(key) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(next);
                }
                std::collections::hash_map::Entry::Occupied(entry) if *entry.get() == next => {}
                std::collections::hash_map::Entry::Occupied(entry) => {
                    return Err(format!(
                        "Ambiguous retained function address {old:#x}: {:#x} or {next:#x}",
                        entry.get()
                    ));
                }
            }
        }
    }
    let added = expanded.len() - incoming.map.len();
    incoming.map = expanded;
    Ok(added)
}

#[cfg(test)]
#[test]
fn patch_aliases_follow_aslr_and_every_observed_generation() {
    use dioxus_devtools::subsecond::JumpTable;
    let table = |pairs: &[(u64, u64)]| JumpTable {
        lib: Default::default(),
        map: pairs.iter().copied().collect(),
        aslr_reference: 0x1000,
        new_base_address: 0,
        ifunc_count: 0,
    };
    let slide = 0x10000;
    let previous = table(&[
        (slide + 0x100, 0x30100),
        (slide + 0x200, 0x30200),
        (0x20100, 0x30100),
        (0x20200, 0x30200),
        (0x20900, 0x30900),
    ]);
    let mut incoming = table(&[(0x100, 0x10), (0x200, 0x20)]);
    assert_eq!(
        carry_patch_aliases(&mut incoming, &previous, slide + 0x1000).unwrap(),
        4
    );
    assert!(!incoming.map.contains_key(&(0x20900 - slide)));

    // Pinned Subsecond::apply_patch rebases keys by the baseline slide and values by the new image slide.
    let second = JumpTable {
        map: incoming
            .map
            .iter()
            .map(|(&key, &value)| (key + slide, value + 0x40000))
            .collect(),
        ..incoming
    };
    let mut third = table(&[(0x100, 0x50), (0x200, 0x60)]);
    assert_eq!(
        carry_patch_aliases(&mut third, &second, slide + 0x1000).unwrap(),
        6
    );
    for old in [slide + 0x100, 0x20100, 0x30100, 0x40010] {
        assert_eq!(third.map.get(&(old - slide)), Some(&0x50));
    }
    for old in [slide + 0x200, 0x20200, 0x30200, 0x40020] {
        assert_eq!(third.map.get(&(old - slide)), Some(&0x60));
    }
}

#[cfg(test)]
#[test]
fn patch_alias_collision_and_invalid_aslr_leave_incoming_unchanged() {
    use dioxus_devtools::subsecond::JumpTable;
    let mut incoming = JumpTable {
        lib: Default::default(),
        map: [(0x100, 0x10), (0x200, 0x20)].into_iter().collect(),
        aslr_reference: 0x1000,
        new_base_address: 0,
        ifunc_count: 0,
    };
    let previous = JumpTable {
        map: [(0x1100, 0x7000), (0x1200, 0x7000)].into_iter().collect(),
        ..incoming.clone()
    };
    let original = incoming.clone();
    assert!(carry_patch_aliases(&mut incoming, &previous, 0x2000)
        .unwrap_err()
        .contains("Ambiguous retained function address"));
    assert_eq!(incoming, original);
    assert!(carry_patch_aliases(&mut incoming, &previous, 0x0).is_err());
    assert_eq!(incoming, original);
    incoming.map.insert(u64::MAX, 0x30);
    let overflowing = incoming.clone();
    let empty_previous = JumpTable {
        map: Default::default(),
        ..previous
    };
    assert!(carry_patch_aliases(&mut incoming, &empty_previous, 0x2000)
        .unwrap_err()
        .contains("overflow"));
    assert_eq!(incoming, overflowing);
}

/// macOS の Application Support(v1 は macOS だけ、V2-6)。
pub(crate) use crate::ui::project::settings_dir;

pub(crate) use crate::ui::poke::Poke;

pub(crate) struct Woken;

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

pub(crate) use crate::ui::poke::wakes_shared_state;

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
    WindowConfig::with_attributes(Box::new(doc) as _, renderer, {
        let mut attrs = WindowAttributes::default()
            .with_title(title.to_string())
            .with_surface_size(dioxus_native::winit::dpi::LogicalSize::new(size.0, size.1));
        // 主窓だけ前回の位置へ(別窓は既定)。
        if title == "Motolii" {
            if let Some(f) = load_window_frame() {
                attrs =
                    attrs.with_position(dioxus_native::winit::dpi::LogicalPosition::new(f.x, f.y));
            }
        }
        attrs
    })
}

/// 窓を増やせるようにするための薄い包み。頼みを先に食べて、残りは上流へ流す。
struct Windows {
    inner: BlitzApplication<DioxusNativeWindowRenderer>,
    asks: Receiver<Ask>,
    session: Session,
    host: Host,
    _catalog_watcher: Option<crate::render::engine::CatalogWatcher>,
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
    if view.window.title() == "Motolii" {
        crate::ui::window_frame::nudge_onto_screen(&*view.window, event_loop);
    }
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
                    .add_filter("Motolii Project", &["rrd"])
                    .set_file_name(crate::ui::project::default_file_name(&self.session))
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
        crate::ui::project::save_to(&self.session, out)
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
                    let mut view_message = message.clone();
                    view_message.jump_table = None;
                    let mut rust_applied = false;
                    if let Some(jump) = &message.jump_table {
                        let target_matches = message.for_pid == Some(std::process::id())
                            && message.for_build_id == Some(dioxus_cli_config::build_id());
                        let duplicate = self.host.last_patch.borrow().as_ref() == Some(&jump.lib);
                        if !target_matches || duplicate {
                            println!(
                                "MOTOLII_RELOAD {}",
                                serde_json::json!({
                                    "event":"ignored", "execution_class":"THIN_PATCH",
                                    "reason":if duplicate { "duplicate_patch" } else { "target_mismatch" },
                                    "pid":std::process::id(), "epoch":self.host.patch_epoch.get()
                                })
                            );
                        } else {
                            let mut incoming = jump.clone();
                            let aliases = unsafe { dioxus_devtools::subsecond::get_jump_table() }
                                .map_or(Ok(0), |previous| {
                                    carry_patch_aliases(
                                        &mut incoming,
                                        previous,
                                        dioxus_devtools::subsecond::aslr_reference() as u64,
                                    )
                                });
                            let aliases = match aliases {
                                Ok(aliases) => aliases,
                                Err(error) => {
                                    println!(
                                        "MOTOLII_RELOAD {}",
                                        serde_json::json!({
                                            "event":"failed", "execution_class":"THIN_PATCH", "reason":error,
                                            "pid":std::process::id()
                                        })
                                    );
                                    *self.session.project_notice.lock().unwrap() =
                                        format!("Reload failed: {error}");
                                    self.host.wake_all();
                                    return;
                                }
                            };
                            crate::ui::inspector::cancel_scrub(&self.session);
                            self.session.gesture.cancel();
                            self.session.close_field();
                            crate::ui::keymap::set_typing(false);
                            self.session.doc.lock().unwrap().clear_all_transients();
                            match unsafe { dioxus_devtools::subsecond::apply_patch(incoming) } {
                                Ok(()) => {
                                    if let Some(watcher) = &self._catalog_watcher {
                                        let runtime = watcher.runtime();
                                        dioxus_devtools::subsecond::HotFn::current(
                                            crate::render::engine::bind_catalog_runtime
                                                as fn(&crate::render::engine::CatalogRuntime),
                                        )
                                        .call((&runtime,));
                                        println!(
                                            "MOTOLII_RELOAD {}",
                                            serde_json::json!({
                                                "event":"catalog_runtime_rebound", "owner":runtime.identity(),
                                                "generation":runtime.generation()
                                            })
                                        );
                                    }
                                    self.host.clear_scope_callbacks();
                                    let epoch = self.host.patch_epoch.get() + 1;
                                    self.host.patch_epoch.set(epoch);
                                    *self.host.last_patch.borrow_mut() = Some(jump.lib.clone());
                                    rust_applied = true;
                                    println!(
                                        "MOTOLII_RELOAD {}",
                                        serde_json::json!({
                                            "event":"accepted", "execution_class":"THIN_PATCH", "epoch":epoch,
                                            "retained_function_aliases":aliases,
                                            "pid":std::process::id(), "build_id":dioxus_cli_config::build_id(),
                                            "document_owner":std::sync::Arc::as_ptr(&self.session.doc) as usize,
                                            "history":self.session.doc.lock().unwrap().history_depth(),
                                            "playing":self.session.clock.playing()
                                        })
                                    );
                                }
                                Err(error) => {
                                    println!(
                                        "MOTOLII_RELOAD {}",
                                        serde_json::json!({
                                            "event":"failed", "execution_class":"THIN_PATCH",
                                            "reason":error.to_string(), "pid":std::process::id()
                                        })
                                    );
                                    *self.session.project_notice.lock().unwrap() =
                                        format!("Reload failed: {error}");
                                    self.host.wake_all();
                                    return;
                                }
                            }
                        }
                    }
                    for (index, window) in self.inner.windows.values_mut().enumerate() {
                        let doc = window.downcast_doc_mut::<DioxusDocument>();
                        if let Some(watcher) = &self._catalog_watcher {
                            doc.vdom.provide_root_context(watcher.runtime());
                        }
                        if let Err(error) =
                            dioxus_devtools::try_apply_changes(&doc.vdom, &view_message)
                        {
                            *self.session.project_notice.lock().unwrap() =
                                format!("View reload failed: {error}");
                            continue;
                        }
                        if rust_applied {
                            doc.vdom
                                .runtime()
                                .in_scope(ScopeId::ROOT_ERROR_BOUNDARY, || {
                                    if let Some(errors) = dioxus_core::try_consume_context::<
                                        dioxus_core::ErrorContext,
                                    >() {
                                        if errors.error().is_some() {
                                            errors.clear_errors();
                                            println!(
                                                "MOTOLII_RELOAD {}",
                                                serde_json::json!({
                                                    "event":"view_error_retry", "window":index,
                                                    "epoch":self.host.patch_epoch.get()
                                                })
                                            );
                                        }
                                    }
                                });
                            doc.vdom.runtime().in_scope(ScopeId::ROOT, || {
                                dioxus_signals::get_global_context()
                                    .clear::<Signal<Option<dioxus_core::internal::HotReloadedTemplate>>>();
                                doc.vdom.runtime().force_all_dirty();
                            });
                        }
                        for asset in &message.assets {
                            if let Some(url) = asset.to_str() {
                                doc.inner.borrow_mut().reload_resource_by_href(url);
                            }
                        }
                        println!(
                            "MOTOLII_RELOAD {}",
                            serde_json::json!({
                                "event":"view_updated", "window":index,
                                "epoch":self.host.patch_epoch.get(), "rust_applied":rust_applied
                            })
                        );
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
                Ask::Open(panel) => {
                    if let Some(view) = self.detached.iter().find_map(|(id, existing)| {
                        (*existing == panel)
                            .then(|| self.inner.windows.get(id))
                            .flatten()
                    }) {
                        view.window.set_minimized(false);
                        view.window.focus_window();
                        view.window.request_redraw();
                        continue;
                    }
                    if self
                        .pending
                        .iter()
                        .any(|(_, existing)| *existing == Some(panel))
                    {
                        continue;
                    }
                    let mut contexts: Vec<Box<dyn std::any::Any>> = vec![
                        Box::new(self.session.clone()),
                        Box::new(self.host.clone()),
                        Box::new(panel),
                    ];
                    if let Some(watcher) = &self._catalog_watcher {
                        contexts.push(Box::new(watcher.runtime()));
                    }
                    self.pending.push((
                        window(
                            crate::ui::app::detached,
                            panel.label(),
                            panel.window_size(),
                            contexts,
                        ),
                        Some(panel),
                    ));
                }
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
        let call = dioxus_devtools::subsecond::call(|| {
            Self::dispatch_window_event
                as fn(&mut Self, &dyn ActiveEventLoop, WindowId, WindowEvent)
        });
        call(self, event_loop, window_id, event);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let call = dioxus_devtools::subsecond::call(|| {
            Self::dispatch_proxy_wake_up as fn(&mut Self, &dyn ActiveEventLoop)
        });
        call(self, event_loop);
    }
}

impl Windows {
    fn dispatch_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        dioxus_devtools::subsecond::HotFn::current(
            Self::run_window_event as fn(&mut Self, &dyn ActiveEventLoop, WindowId, WindowEvent),
        )
        .call((self, event_loop, window_id, event));
    }

    fn dispatch_proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        dioxus_devtools::subsecond::HotFn::current(
            Self::run_proxy_wake_up as fn(&mut Self, &dyn ActiveEventLoop),
        )
        .call((self, event_loop));
    }

    fn run_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.inner.windows.contains_key(&window_id) {
            self.reap_closed_window_state();
            return;
        }

        // Blitz は hover node が無い瞬間を `set_cursor(None)` として shell へ渡し、
        // shell は None を「既定」ではなく `set_cursor_visible(false)` にしている。
        // custom widget(Stage/Timeline)の空所や再layout直後でこれが起きるため、
        // Motolii 内を指しているnative eventの最後に可視性を必ず取り戻す。
        // このappは CSS cursor:none を使わないので、意図的な非表示との競合は無い。
        let restore_cursor = matches!(
            &event,
            WindowEvent::PointerMoved { .. }
                | WindowEvent::PointerEntered { .. }
                | WindowEvent::Focused(true)
        );
        if self.session.quit.load(std::sync::atomic::Ordering::Relaxed) {
            // ⌘Q でも枠を憶える(閉じるボタンだけだった)。主窓 = 別窓に登録されていない窓。
            if let Some(view) = self
                .inner
                .windows
                .iter()
                .find(|(id, _)| !self.detached.contains_key(id))
                .map(|(_, v)| v)
            {
                save_window_frame(&*view.window);
            }
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
            // Native fullscreen consumes Escape before document/menu commands.
            if key.state.is_pressed()
                && matches!(
                    key.logical_key,
                    dioxus_native::winit::keyboard::Key::Named(
                        dioxus_native::winit::keyboard::NamedKey::Escape
                    )
                )
            {
                if let Some(view) = self.inner.windows.get(&window_id) {
                    if view.window.fullscreen().is_some() {
                        view.window.set_fullscreen(None);
                        view.window.request_redraw();
                        println!(
                            "MOTOLII_RELOAD {}",
                            serde_json::json!({
                                "event":"fullscreen_exit_requested", "window":format!("{window_id:?}"),
                                "reason":"escape"
                            })
                        );
                        return;
                    }
                }
            }
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                let doc = view.downcast_doc_mut::<DioxusDocument>();
                aim_keystrokes(doc);
                if key.state.is_pressed() {
                    let logical = keyboard_types::Key::Character(
                        key.text.as_deref().unwrap_or("").to_string(),
                    );
                    let is_enter = matches!(
                        key.logical_key,
                        dioxus_native::winit::keyboard::Key::Named(
                            dioxus_native::winit::keyboard::NamedKey::Enter
                        )
                    );
                    let is_tab = matches!(
                        key.logical_key,
                        dioxus_native::winit::keyboard::Key::Named(
                            dioxus_native::winit::keyboard::NamedKey::Tab
                        )
                    );
                    let k = if is_enter {
                        keyboard_types::Key::Enter
                    } else if is_tab {
                        keyboard_types::Key::Tab
                    } else {
                        logical
                    };
                    if crate::ui::keys::step_focus_back(doc, &k, crate::ui::keymap::shift_held()) {
                        return;
                    }
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
                    .set_title(format!(
                        "Do you want to save the changes you made to {name}?"
                    ))
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
                let native = self.inner.windows.get(&window_id).map(|view| {
                    view.window.set_visible(false);
                    std::sync::Arc::downgrade(&view.window)
                });
                crate::ui::inspector::cancel_scrub(&self.session);
                self.session.gesture.cancel();
                let callbacks = self.retire_window_state(window_id);
                self.inner.window_event(event_loop, window_id, event);
                self.host.closed(panel);
                self.focus_main_after_child_close();
                println!(
                    "MOTOLII_RELOAD {}",
                    serde_json::json!({
                        "event":"child_window_closed", "window":format!("{window_id:?}"),
                        "callbacks_retired":callbacks, "remaining_windows":self.inner.windows.len(),
                        "retained_native_refs":native.map_or(0, |window| window.strong_count())
                    })
                );
                return;
            } else {
                // 主窓を閉じたら終わる。panel の別窓だけを残さない(Mac の document app)。
                if let Some(view) = self.inner.windows.get(&window_id) {
                    save_window_frame(&*view.window);
                }
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
            if restore_cursor {
                view.window.set_cursor_visible(true);
            }
            place_ime(view);
        }
        self.reflect_document(window_id);
    }

    fn retire_window_state(&mut self, window: WindowId) -> usize {
        self.cursor.remove(&window);
        self.seen_field.remove(&window);
        self.reflected.remove(&window);
        self.host.mounts.remove_window(window);
        self.host.retire_window_callbacks(window)
    }

    fn focus_main_after_child_close(&self) {
        if let Some((_, main)) = self
            .inner
            .windows
            .iter()
            .find(|(id, _)| !self.detached.contains_key(*id))
        {
            main.window.focus_window();
            main.window.request_redraw();
        }
        self.host.wake_all();
    }

    fn reap_closed_window_state(&mut self) {
        let closed: std::collections::HashSet<_> = self
            .cursor
            .keys()
            .chain(self.seen_field.keys())
            .chain(self.reflected.keys())
            .chain(self.detached.keys())
            .filter(|id| !self.inner.windows.contains_key(*id))
            .copied()
            .collect();
        if closed.is_empty() {
            return;
        }
        for window in closed {
            let callbacks = self.retire_window_state(window);
            if let Some(panel) = self.detached.remove(&window) {
                self.host.closed(panel);
            }
            println!(
                "MOTOLII_RELOAD {}",
                serde_json::json!({
                    "event":"closed_window_reaped", "window":format!("{window:?}"), "callbacks_retired":callbacks,
                    "remaining_windows":self.inner.windows.len()
                })
            );
        }
        self.focus_main_after_child_close();
    }

    fn run_proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.reap_closed_window_state();
        let retained_catalog = self
            ._catalog_watcher
            .as_ref()
            .map(|watcher| watcher.runtime());
        if let Some(runtime) = &retained_catalog {
            for window in self.inner.windows.values_mut() {
                window
                    .downcast_doc_mut::<DioxusDocument>()
                    .vdom
                    .provide_root_context(runtime.clone());
            }
            dioxus_devtools::subsecond::HotFn::current(
                crate::render::engine::bind_catalog_runtime
                    as fn(&crate::render::engine::CatalogRuntime),
            )
            .call((runtime,));
        }
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
                // 支援技術(VoiceOver)の押下。上流の blitz-shell は `ActionRequested` を捨てる(TODO)。
                BlitzShellEvent::Accessibility {
                    window_id,
                    ref data,
                } if crate::ui::keys::action_of(data).is_some() => {
                    if let (Some(req), Some(view)) = (
                        crate::ui::keys::action_of(data),
                        self.inner.windows.get_mut(&window_id),
                    ) {
                        crate::ui::keys::act(view.downcast_doc_mut::<DioxusDocument>(), req);
                    }
                }
                BlitzShellEvent::CloseWindow { window_id } => {
                    Self::dispatch_window_event(
                        self,
                        event_loop,
                        window_id,
                        WindowEvent::CloseRequested,
                    );
                }
                event => self.inner.handle_blitz_shell_event(event_loop, event),
            }
        }
        if wake_shared
            || retained_catalog
                .as_ref()
                .is_some_and(|runtime| runtime.is_dirty())
        {
            let catalog = if let Some(runtime) = &retained_catalog {
                dioxus_devtools::subsecond::HotFn::current(
                    crate::render::engine::refresh_effect_catalog_for
                        as fn(
                            &crate::render::engine::CatalogRuntime,
                        ) -> crate::render::engine::CatalogRefresh,
                )
                .call((runtime,))
            } else {
                dioxus_devtools::subsecond::HotFn::current(
                    crate::render::engine::refresh_effect_catalog
                        as fn() -> crate::render::engine::CatalogRefresh,
                )
                .call(())
            };
            if catalog.changed {
                crate::ui::inspector::cancel_scrub(&self.session);
                self.session.gesture.cancel();
                self.session.close_field();
                crate::ui::keymap::set_typing(false);
                self.session.doc.lock().unwrap().clear_all_transients();
            }
            let notice_changed = {
                let mut notice = self.session.project_notice.lock().unwrap();
                let next = if !catalog.errors.is_empty() {
                    Some(format!("Effect reload: {}", catalog.errors.join("; ")))
                } else if notice.starts_with("Effect reload: ") {
                    Some(String::new())
                } else {
                    None
                };
                next.is_some_and(|next| {
                    if *notice == next {
                        return false;
                    }
                    *notice = next;
                    true
                })
            };
            if catalog.changed || notice_changed {
                println!(
                    "MOTOLII_RELOAD {}",
                    serde_json::json!({"event":"catalog_updated", "generation":catalog.generation, "changed":catalog.changed, "errors":catalog.errors})
                );
                for window in self.inner.windows.values() {
                    window.window.request_redraw();
                }
            }
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

    let catalog_poke = crate::ui::poke::Poke(Some(proxy.clone()));
    let catalog_watcher = match crate::render::engine::watch_effect_catalog(move || {
        catalog_poke.poke()
    }) {
        Ok(watcher) => {
            if let (Some(path), Ok(session)) = (
                std::env::var_os("MOTOLII_RUNTIME_SOURCE_REGISTRATION"),
                std::env::var("MOTOLII_RELOAD_SESSION"),
            ) {
                let path = std::path::PathBuf::from(path);
                let temporary = path.with_extension("tmp");
                let data = serde_json::json!({"app_pid":std::process::id(), "session":session, "roots":crate::render::engine::catalog_source_roots()});
                if let Err(error) = std::fs::write(&temporary, data.to_string())
                    .and_then(|_| std::fs::rename(&temporary, &path))
                {
                    eprintln!(
                        "MOTOLII_RELOAD {}",
                        serde_json::json!({"event":"runtime_registration_failed", "reason":error.to_string()})
                    );
                }
            }

            println!(
                "MOTOLII_RELOAD {}",
                serde_json::json!({"event":"catalog_watcher_ready", "pid":std::process::id(), "owner":watcher.runtime().identity()})
            );
            Some(watcher)
        }
        Err(error) => {
            eprintln!(
                "MOTOLII_RELOAD {}",
                serde_json::json!({"event":"catalog_watcher_failed", "reason":error})
            );
            None
        }
    };
    let Loaded {
        doc,
        ui,
        duration_sec,
    } = load_fixture();
    // 普通のソフトは白紙で起動する。見本の作品は MOTOLII_FIXTURE=1 の時だけ(試験と実窓の検分用)。
    let (mut opened, argv_path, load_error) = crate::ui::project::open_from_argv();
    let from_argv = opened.is_some();
    let doc = match (opened.take(), std::env::var_os("MOTOLII_FIXTURE").is_some()) {
        (Some(loaded), _) => loaded,
        (None, true) => doc,
        (None, false) => crate::ui::blank_project(),
    };
    let session = Session::new(doc, duration_sec, ui);
    if let (Some(path), true) = (argv_path, from_argv) {
        let revision = session.doc.lock().unwrap().revision();
        session.mark_saved(path.clone(), revision);
        crate::ui::project::remember_recent(&path);
    }
    if let Some(e) = load_error {
        *session.project_notice.lock().unwrap() = e;
    }
    crate::ui::autosave::start(&session);
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
        mounts: Default::default(),
        patch_epoch: Default::default(),
        last_patch: Default::default(),
    };

    // 窓の枠は前回の続き(macOS の作法)。無ければ既定。
    let frame = load_window_frame();
    let mut contexts: Vec<Box<dyn std::any::Any>> =
        vec![Box::new(session.clone()), Box::new(host.clone())];
    if let Some(watcher) = &catalog_watcher {
        contexts.push(Box::new(watcher.runtime()));
    }
    let main = window(
        crate::ui::app::app,
        title,
        frame.map_or((1600, 1000), |f| (f.w, f.h)),
        contexts,
    );
    let inner = BlitzApplication::new(proxy, event_queue);

    event_loop
        .run_app(Windows {
            inner,
            asks,
            session,
            host,
            _catalog_watcher: catalog_watcher,
            pending: vec![(main, None)],
            detached: Default::default(),
            cursor: Default::default(),
            seen_field: Default::default(),
            reflected: Default::default(),
        })
        .unwrap();
}
pub(crate) use crate::ui::window_frame::{load_window_frame, save_window_frame};

#[cfg(test)]
#[test]
fn closing_one_window_retires_only_its_callbacks() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let host = Host::for_tests();
    let main = WindowId::from_raw(81);
    let child = WindowId::from_raw(82);
    let calls = Rc::new(RefCell::new(Vec::new()));
    for (window, tag) in [(main, "main-focus"), (child, "child-focus")] {
        let calls = calls.clone();
        host.on_focus_lost(window, move || calls.borrow_mut().push(tag));
    }
    for (window, tag) in [(main, "main-release"), (child, "child-release")] {
        let calls = calls.clone();
        host.on_primary_pointer_release(window, move |_, _, _| calls.borrow_mut().push(tag));
    }
    let live_calls = calls.clone();
    host.listen(move || live_calls.borrow_mut().push("main-wake"));
    assert_eq!(host.retire_window_callbacks(child), 2);
    assert_eq!(host.retire_window_callbacks(child), 0);
    host.focus_lost();
    host.primary_pointer_released(child, 0.0, 0.0, false);
    host.primary_pointer_released(main, 0.0, 0.0, false);
    host.wake_all();
    assert_eq!(
        &*calls.borrow(),
        &["main-focus", "main-release", "main-wake"]
    );
}
