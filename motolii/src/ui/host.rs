
use blitz_shell::{create_default_event_loop, BlitzShellProxy, WindowConfig};
use dioxus_native::prelude::VirtualDom;
use dioxus_native::{
    DioxusDocument, DioxusNativeApplication, DioxusNativeWindowRenderer, RendererOptions,
};
use dioxus_native::winit::window::WindowAttributes;

use blitz_dom::DocumentConfig;

/// 窓を1枚作る。`dioxus_native::launch_cfg` は窓を1枚だけ作って握ったまま返らないので、
/// 窓を増やす口が要る Motolii は blitz の embedder の形をそのまま持つ。
fn window(
    root: fn() -> dioxus_native::prelude::Element,
    title: &str,
) -> WindowConfig<DioxusNativeWindowRenderer> {
    let vdom = VirtualDom::new(root);
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
        WindowAttributes::default().with_title(title.to_string()),
    )
}

pub fn launch(root: fn() -> dioxus_native::prelude::Element, title: &str) {
    let event_loop = create_default_event_loop();
    let (proxy, event_queue) = BlitzShellProxy::new(event_loop.create_proxy());

    let application = DioxusNativeApplication::new(proxy, event_queue, window(root, title));
    event_loop.run_app(application).unwrap();
}
