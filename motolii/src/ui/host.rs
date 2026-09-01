
use std::sync::mpsc::{channel, Receiver, Sender};

use blitz_shell::{create_default_event_loop, BlitzShellEvent, BlitzShellProxy, WindowConfig};
use dioxus_native::prelude::VirtualDom;
use dioxus_native::winit::application::ApplicationHandler;
use dioxus_native::winit::event::{StartCause, WindowEvent};
use dioxus_native::winit::event_loop::ActiveEventLoop;
use dioxus_native::winit::window::{WindowAttributes, WindowId};
use dioxus_native::prelude::{provide_context, ScopeId};
use dioxus_native::{DioxusDocument, DioxusNativeWindowRenderer, RendererOptions};
use blitz_shell::{BlitzApplication, View};

use blitz_dom::{Document, DocumentConfig};

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
    /// 窓を起こす線。窓ごとに1本、自分の runtime を包んで置く。
    /// 状態は全窓で1つなので、誰かが書いたら他の窓も描き直す必要がある。
    wakers: std::rc::Rc<std::cell::RefCell<Vec<(u64, std::rc::Rc<dyn Fn()>)>>>,
    next_id: std::rc::Rc<std::cell::Cell<u64>>,
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
        let wakers: Vec<_> = self.wakers.borrow().iter().map(|(_, w)| w.clone()).collect();
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

    fn closed(&self, panel: Panel) {
        let f = self.on_close.borrow().clone();
        if let Some(f) = f {
            f(panel);
        }
    }

    /// 窓を開けずに動かす時用。頼みは誰も受け取らない。
    #[cfg(test)]
    pub(crate) fn for_tests() -> Self {
        let (tx, rx) = channel();
        std::mem::forget(rx);
        Self {
            tx,
            proxy: None,
            on_close: Default::default(),
            wakers: Default::default(),
            next_id: Default::default(),
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
            net_provider: Some(std::sync::Arc::new(blitz_shell::DataUriNetProvider::new(None))),
            ..Default::default()
        },
    );
    let renderer = DioxusNativeWindowRenderer::with_options(RendererOptions::default());
    WindowConfig::with_attributes(
        Box::new(doc) as _,
        renderer,
        WindowAttributes::default().with_title(title.to_string()).with_surface_size(
            dioxus_native::winit::dpi::LogicalSize::new(size.0, size.1),
        ),
    )
}

/// 打鍵をどこへ配るかを決める。**窓の側と試験の側で同じ規則を通す** —— 分けると、
/// 利用者が歩く道(欄を開けて打つ)の試験が書けない。
pub(crate) fn aim_keystrokes(doc: &mut DioxusDocument) {
    let field = doc.inner().query_selector("input").ok().flatten();
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
                Ask::Open(panel) => self.pending.push((window(
                    crate::ui::app::detached,
                    panel.label(),
                    panel.window_size(),
                    vec![
                        Box::new(self.session.clone()),
                        Box::new(self.host.clone()),
                        Box::new(panel),
                    ],
                ), Some(panel))),
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
        if matches!(event, WindowEvent::KeyboardInput { .. }) {
            if let Some(view) = self.inner.windows.get_mut(&window_id) {
                aim_keystrokes(view.downcast_doc_mut::<DioxusDocument>());
            }
        }
        if let WindowEvent::PointerMoved { position, .. } = &event {
            self.cursor.insert(window_id, (position.x as f32, position.y as f32));
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
        // Finder から落ちてきた素材を棚へ入れる。窓の外の出来事なので、
        // 入れたあと自分で全ての窓を起こす。
        if let WindowEvent::DragDropped { paths, .. } = &event {
            let admitted = {
                let mut doc = self.session.doc.lock().unwrap();
                paths
                    .iter()
                    .filter(|p| crate::ui::fixture::admit_path(&mut doc, p))
                    .count()
            };
            println!("PROBE room=browser verdict=drop admitted={admitted} of={}", paths.len());
            if admitted > 0 {
                self.host.wake_all();
            }
        }
        if matches!(event, WindowEvent::CloseRequested) {
            if let Some(panel) = self.detached.remove(&window_id) {
                self.host.closed(panel);
            }
        }
        self.inner.window_event(event_loop, window_id, event);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        // 窓の外(書き出しの糸など)で状態が変わっている。全ての窓を描き直す。
        self.host.wake_all();
        self.serve_asks();
        self.realise_pending(event_loop);
        self.inner.proxy_wake_up(event_loop);
    }
}

pub fn launch(title: &str) {
    let event_loop = create_default_event_loop();
    let (proxy, event_queue) = BlitzShellProxy::new(event_loop.create_proxy());

    let Loaded { doc, ui, duration_sec } = load_fixture();
    let session = Session::new(doc, duration_sec, ui);
    let (tx, asks) = channel();
    let host = Host {
        tx,
        proxy: Some(proxy.clone()),
        on_close: Default::default(),
        wakers: Default::default(),
        next_id: Default::default(),
    };

    let main = window(
        crate::ui::app::app,
        title,
        (1600, 1000),
        vec![
            Box::new(session.clone()),
            Box::new(host.clone()),
        ],
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
        })
        .unwrap();
}
