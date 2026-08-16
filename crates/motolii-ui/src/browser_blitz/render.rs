//! Blitz(vello_hybrid)描画面。wgpu textureへ出す。
//!
//! ## 罠1(実測): `ImageManager` の `cache` はフレームを跨いで保持する
//! `img_cache` / `tex_bindings` を毎フレーム新規生成すると、全画像がatlasへ
//! 再確保され続け数秒で `AtlasLimitReached` に達する。しかも
//! `vello_hybrid/src/render/wgpu.rs:596` はライブラリ内 `.unwrap()` なので**捕捉できない**。
//! よって両者は本structのフィールドとして保持し、`render` は `&mut self` 経由で貸すだけ。
//! (`docs/reviews/2026-08-15-blitz-ui-runtime-probe.md:246-257`)

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::BaseDocument;
use blitz_paint::paint_scene;
use rustc_hash::FxHashMap;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

/// プロセスで1つだけ作る `wgpu::Instance`。
/// `anyrender` の `DeviceHandle` が要求するので持つが、**中身は使い回す**。
fn shared_instance() -> &'static wgpu::Instance {
    static INSTANCE: std::sync::OnceLock<wgpu::Instance> = std::sync::OnceLock::new();
    INSTANCE
        .get_or_init(|| wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()))
}

pub struct BlitzSurface {
    renderer: Renderer,
    resources: Resources,
    device_handle: DeviceHandle,
    width: u32,
    height: u32,
    /// **フレームを跨いで保持する。** 毎フレーム作り直すと罠1でpanicする。
    image_cache: FxHashMap<u64, vello_common::paint::ImageId>,
    /// 同上。`texture_bindings` もフレームを跨いで保持する。
    texture_bindings: FxHashMap<anyrender::ResourceId, wgpu::TextureView>,
    /// paint scale。**1 CSS px を何物理px で描くか。**
    /// 既定は1.0で、これは「CSS px = 物理px」を意味する。hidpi の面へ等倍で描くなら
    /// `set_scale` で `pixels_per_point` を渡す。渡さないと Retina で全部が半分に見える。
    /// 文書側の `Viewport::hidpi_scale` と**同じ値**にすること(片方だけだとレイアウトと
    /// 描画の縮尺がずれる)。
    scale: f64,
}

impl BlitzSurface {
    pub fn new(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            renderer: Renderer::new(
                device,
                &RenderTargetConfig {
                    format,
                    width,
                    height,
                },
            ),
            resources: Resources::new(),
            device_handle: DeviceHandle {
                // **使い回す。** `wgpu::Instance::new` はバックエンドとアダプタを
                // 列挙するので安くない。面の大きさが変わるたびに `BlitzSurface` を
                // 作り直す呼び手(ドッキングのドラッグ中は毎フレーム)がいるため、
                // ここで毎回作ると драг が目に見えて重くなる。
                instance: shared_instance().clone(),
                adapter: adapter.clone(),
                device: device.clone(),
                queue: queue.clone(),
            },
            width,
            height,
            image_cache: FxHashMap::default(),
            texture_bindings: FxHashMap::default(),
            scale: 1.0,
        }
    }

    /// paint scale を差し替える。`Viewport::hidpi_scale` と同じ値を渡すこと。
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// 面の大きさを変える。**`Renderer` も `Resources` も作り直さない。**
    ///
    /// `vello_hybrid::Renderer::render` は毎回 `RenderSize` を受け取り、内部の
    /// `maybe_update_config_buffer` が「前回と寸法が違うとき」だけ config buffer を
    /// 書き換えて depth texture を作り直す
    /// (`vello_hybrid-0.0.9/src/render/wgpu.rs:2302-2334`)。構築時に pipeline へ
    /// 焼き込まれるのは `format` だけ(`:1317`, `:1404`, `:1572`)なので、
    /// **寸法が変わっただけで `BlitzSurface` を作り直す必要は無い**。
    ///
    /// 作り直す方が高くつく: `image_cache` と `Resources`(グリフatlasを持つ)を
    /// 捨てることになり、次のフレームで全部載せ直しになる。
    /// 上流 Blitz の `examples/wgpu_texture` も、リサイズ時に作り直すのは
    /// テクスチャだけで pipeline と device はそのままにしている。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// 1フレーム描いて `target` へ書く。GPU resourceはloop内で作らない
    /// (renderer/resources/cacheはすべて再利用)。
    pub fn render(
        &mut self,
        document: &mut BaseDocument,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) {
        let (width, height) = (self.width, self.height);
        let mut scene = Scene::new(width as u16, height as u16);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let image_manager = ImageManager::new(
                &mut self.renderer,
                &mut self.resources,
                device,
                queue,
                &mut encoder,
                &mut self.image_cache,
            );
            let mut painter = VelloHybridScenePainter::new(
                &mut scene,
                image_manager,
                &mut self.texture_bindings,
                &self.device_handle,
            );
            paint_scene(&mut painter, document, self.scale, width, height, 0, 0);
        }
        let _ = self.renderer.render(
            &scene,
            &mut self.resources,
            device,
            queue,
            &mut encoder,
            &RenderSize { width, height },
            target,
            &TextureBindings::default(),
        );
        queue.submit([encoder.finish()]);
    }

    /// atlasへ載っている画像の枚数。罠1の再発をテスト/観測から見るための窓。
    pub fn cached_image_count(&self) -> usize {
        self.image_cache.len()
    }
}
