//! Blitz(HTML/CSS)を自前wgpuテクスチャへ描く共有面。
//!
//! Timeline／Inspector／Browserの各面がこれを1つずつ持つ。
//! 境界は「テクスチャを返す」「イベントを受け取る」の2本だけに保つ
//! (`docs/reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md` 1節)。
//!
//! 実測で踏んだ罠を構造で避ける:
//!   - `ImageManager` の画像キャッシュと texture binding は**フレームを跨いで保持する**。
//!     毎フレーム作り直すと atlas へ再確保が積み上がり `AtlasLimitReached` で panic する
//!     (`vello_hybrid` 内の `.unwrap()` なので捕捉できない)。
//!   - `BaseDocument::set_style_property` はレイアウトを無効化しない。
//!     属性更新は `mutate().set_attribute` を使う (`restyle` の項を参照)。

use std::sync::Arc;

use anyrender::ResourceId;
use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use atomic_refcell::AtomicRefCell;
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::Modifiers;
use rustc_hash::FxHashMap;
use vello_common::paint::ImageId;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

/// 面が受け取るポインタ位相。`motolii_input::InputPhase` へ写す前の生の区別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfacePointer {
    Move,
    Down,
    Up,
}

/// Blitz文書1つと、その描画先テクスチャ。
pub(crate) struct BlitzSurface {
    doc: HtmlDocument,
    renderer: Renderer,
    resources: Resources,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    device_handle: DeviceHandle,
    width: u32,
    height: u32,
    /// **フレームを跨いで保持する。** 毎フレーム作り直すと atlas が溢れる。
    image_cache: FxHashMap<u64, ImageId>,
    /// 同上。
    texture_bindings: FxHashMap<ResourceId, wgpu::TextureView>,
}

impl BlitzSurface {
    /// `html` は完全な文書。`device`/`queue` はホスト(eframe)のものをそのまま使う。
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adapter: &wgpu::Adapter,
        instance: &wgpu::Instance,
        width: u32,
        height: u32,
        html: &str,
    ) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        let texture = create_texture(device, width, height);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut surface = Self {
            doc: build_document(html, width, height),
            renderer: Renderer::new(
                device,
                &RenderTargetConfig {
                    format: TEXTURE_FORMAT,
                    width,
                    height,
                },
            ),
            resources: Resources::new(),
            texture,
            view,
            device_handle: DeviceHandle {
                instance: instance.clone(),
                adapter: adapter.clone(),
                device: device.clone(),
                queue: queue.clone(),
            },
            width,
            height,
            image_cache: FxHashMap::default(),
            texture_bindings: FxHashMap::default(),
        };
        surface.doc.resolve(0.0);
        surface
    }

    pub(crate) fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// 面の大きさが変わったら作り直す。テクスチャidは呼び出し側で再登録する。
    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) -> bool {
        let (width, height) = (width.max(1), height.max(1));
        if (self.width, self.height) == (width, height) {
            return false;
        }
        self.texture = create_texture(device, width, height);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer = Renderer::new(
            device,
            &RenderTargetConfig {
                format: TEXTURE_FORMAT,
                width,
                height,
            },
        );
        self.resources = Resources::new();
        self.image_cache.clear();
        self.texture_bindings.clear();
        self.width = width;
        self.height = height;
        self.doc.set_viewport(viewport(width, height));
        true
    }

    /// 文書ごと差し替える。構造が変わった時だけ使い、値だけの変更は
    /// [`Self::set_style`] を使う(再パースは実測で25msかかる)。
    pub(crate) fn replace_document(&mut self, html: &str) {
        self.doc = build_document(html, self.width, self.height);
        self.doc.resolve(0.0);
    }

    /// 既存ノードの`style`属性を差し替える。**レイアウトを無効化する正しい経路。**
    ///
    /// `BaseDocument::set_style_property` は属性を設定するが再レイアウトを起こさず、
    /// 「何もしていない速さ」が出る(実測で踏んだ)。必ずこちらを使う。
    pub(crate) fn set_style(&mut self, node_id: usize, style: &str) {
        let mut mutator = self.doc.mutate();
        mutator.set_attribute(node_id, blitz_dom::qual_name!("style"), style);
    }

    /// `id` 属性でノードを引く。`resolve` 前でも使える。
    pub(crate) fn node_by_id(&self, id: &str) -> Option<usize> {
        fn walk(doc: &HtmlDocument, node_id: usize, want: &str) -> Option<usize> {
            let node = doc.get_node(node_id)?;
            if let Some(element) = node.element_data() {
                if element.attr(blitz_dom::local_name!("id")) == Some(want) {
                    return Some(node_id);
                }
            }
            node.children
                .iter()
                .find_map(|child| walk(doc, *child, want))
        }
        walk(&self.doc, self.doc.root_element().id, id)
    }

    /// 面のローカル座標でヒット判定する。
    ///
    /// 「ポインタがこの面の上か、canvas面の上か」をMotolii側で振り分けるために使う。
    pub(crate) fn hit(&self, x: f32, y: f32) -> Option<usize> {
        self.doc.hit(x, y).map(|hit| hit.node_id)
    }

    /// ポインタイベントを注入する。ルーティングの決定権はMotolii側にある。
    pub(crate) fn send_pointer(&mut self, phase: SurfacePointer, x: f32, y: f32) {
        let event = pointer_event(x, y, phase);
        let ui_event = match phase {
            SurfacePointer::Move => UiEvent::PointerMove(event),
            SurfacePointer::Down => UiEvent::PointerDown(event),
            SurfacePointer::Up => UiEvent::PointerUp(event),
        };
        let mut driver = EventDriver::new(&mut *self.doc, NoopEventHandler);
        driver.handle_ui_event(ui_event);
    }

    /// スタイル解決とレイアウトを走らせる。毎フレーム呼ぶ。
    pub(crate) fn resolve(&mut self) {
        self.doc.resolve(0.0);
    }

    /// 自前テクスチャへ描く。`encoder` はホスト側のものを借りる。
    pub(crate) fn paint(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut scene = Scene::new(self.width as u16, self.height as u16);
        {
            let image_manager = ImageManager::new(
                &mut self.renderer,
                &mut self.resources,
                device,
                queue,
                encoder,
                &mut self.image_cache,
            );
            let mut painter = VelloHybridScenePainter::new(
                &mut scene,
                image_manager,
                &mut self.texture_bindings,
                &self.device_handle,
            );
            paint_scene(
                &mut painter,
                &mut self.doc,
                1.0,
                self.width,
                self.height,
                0,
                0,
            );
        }
        let _ = self.renderer.render(
            &scene,
            &mut self.resources,
            device,
            queue,
            encoder,
            &RenderSize {
                width: self.width,
                height: self.height,
            },
            &self.view,
            &TextureBindings::default(),
        );
    }
}

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("motolii-blitz-surface"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn viewport(width: u32, height: u32) -> Viewport {
    Viewport {
        window_size: (width, height),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    }
}

fn build_document(html: &str, width: u32, height: u32) -> HtmlDocument {
    // `StyleThreading::Parallel` は実測で3倍遅い(DOMが浅く広く、
    // スレッドプールの調整コストが1ノードあたりの仕事を上回るため)。
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            net_provider: Some(blitz_net::Provider::shared(None)),
            ..Default::default()
        },
    );
    doc.set_viewport(viewport(width, height));
    doc
}

fn pointer_event(x: f32, y: f32, phase: SurfacePointer) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: match phase {
            SurfacePointer::Down => MouseEventButtons::Primary,
            _ => MouseEventButtons::None,
        },
        mods: Modifiers::default(),
        details: PointerDetails {
            pressure: if matches!(phase, SurfacePointer::Down) {
                0.5
            } else {
                0.0
            },
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            altitude: 0.0,
            azimuth: 0.0,
        },
        element: Point { x, y },
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}
