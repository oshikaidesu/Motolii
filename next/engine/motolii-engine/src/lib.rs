//! wraps: motolii-store + motolii-compositor — **1フレームを出す唯一の経路**。
//!
//! 背骨2: preview も export も [`Engine::render_frame`] を呼ぶ。窓の有無だけが違う。
//! ここに「書き出し専用の速い道」を足さない — 足した瞬間に「見た絵 ≠ 出る絵」が生まれる。
//!
//! この crate 自身は意味を持たない。Document の意味は `motolii-store`、
//! 補間は `motolii-eval`、描画は `re_renderer` にある。ここは繋ぐだけ。

use std::collections::HashMap;

use motolii_compositor::{CompSpec, Compositor, CompositorError, Layer};
use motolii_store::{LayerSource, RationalTime, StoreView};
use motolii_compositor::GpuTexture2D;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Compositor(#[from] CompositorError),
}

pub struct Engine {
    compositor: Compositor,
    /// 素材 → GPU texture。同じ素材を毎フレーム上げ直さない。
    textures: HashMap<LayerSource, GpuTexture2D>,
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            textures: HashMap::new(),
        })
    }

    /// **唯一の評価経路**。Document のある時刻の姿を1枚の RGBA8 にする。
    pub fn render_frame(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<Vec<u8>, EngineError> {
        let resolved = view.resolved_layers(t);

        let mut layers = Vec::with_capacity(resolved.len());
        for layer in resolved {
            let texture = self.texture_for(&layer.source)?;
            layers.push(Layer {
                texture,
                top_left: layer.top_left,
                size: layer.size,
                order: layer.order,
                opacity: layer.opacity,
            });
        }

        Ok(self.compositor.render(comp, &layers)?)
    }

    fn texture_for(&mut self, source: &LayerSource) -> Result<GpuTexture2D, EngineError> {
        if let Some(texture) = self.textures.get(source) {
            return Ok(texture.clone());
        }

        let texture = match source {
            LayerSource::Solid {
                rgba,
                width,
                height,
            } => {
                let pixels: Vec<u8> = rgba
                    .iter()
                    .copied()
                    .cycle()
                    .take((width * height * 4) as usize)
                    .collect();
                self.compositor
                    .upload_rgba("solid", &pixels, *width, *height)?
            }
        };

        self.textures.insert(source.clone(), texture.clone());
        Ok(texture)
    }
}
