//! RN Browser chrome を載せた Blitz media panel。
//!
//! `ui/motolii-rn/src/Browser.tsx` の MEDIA / GRID 見た目を、既存
//! `media_library` の画像投影と縮小実体へ重ねる。走査・種別判定・path 解決は
//! `media_library` が owner のままで、選択・drag・配置intent・file dialog は持たない。
//! 実画像は元解像度でなく `thumbnail.rs` の縮小実体を使う。

mod library_view;
mod markup;
pub mod render;
mod theme;
mod thumbnail;

use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use std::path::Path;

pub use library_view::{BrowserItem, DEFAULT_MAX_ITEMS, image_items};

/// パネル生成の失敗。
#[derive(Debug, thiserror::Error)]
pub enum BrowserBlitzError {
    /// `blitz_net::Provider` は Tokio reactor が無いと panic するため、
    /// runtime を張れない時点で作らずに返す。
    #[error("tokio runtime for blitz-net is unavailable")]
    Runtime(#[source] std::io::Error),
}

/// Browser パネル一面。Document / image cache / Tokio reactor を表示中ずっと保持する。
pub struct BrowserBlitzPanel {
    document: HtmlDocument,
    surface: render::BlitzSurface,
    runtime: tokio::runtime::Runtime,
    items: Vec<BrowserItem>,
    width: u32,
    height: u32,
    scale: f64,
}

impl BrowserBlitzPanel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scale: f64,
        title: impl Into<String>,
        items: Vec<BrowserItem>,
    ) -> Result<Self, BrowserBlitzError> {
        let runtime = tokio::runtime::Runtime::new().map_err(BrowserBlitzError::Runtime)?;
        let title = title.into();
        // 縮小実体は Document を組む前に用意する。2回目以降は cache path を使うだけ。
        let thumbnails = thumbnail::prepare(&items);
        let document = {
            let _guard = runtime.enter();
            build_document(width, height, scale, &title, &items, &thumbnails)
        };
        let mut surface = render::BlitzSurface::new(
            device,
            adapter,
            queue,
            format,
            ((width as f64) * scale).round() as u32,
            ((height as f64) * scale).round() as u32,
        );
        surface.set_scale(scale);
        Ok(Self {
            document,
            surface,
            runtime,
            items,
            width,
            height,
            scale,
        })
    }

    /// 寸法変更は既存 Document の viewport と既存 surface へ反映する。
    /// runtime / image cache / thumbnail は作り直さない。
    pub fn resize(&mut self, width: u32, height: u32, scale: f64) {
        if self.width == width && self.height == height && self.scale == scale {
            return;
        }
        let size_changed = self.width != width || self.height != height;
        self.width = width;
        self.height = height;
        self.scale = scale;
        if size_changed {
            self.surface.resize(
                ((width as f64) * scale).round() as u32,
                ((height as f64) * scale).round() as u32,
            );
        }
        self.surface.set_scale(scale);
        self.document.set_viewport(viewport(width, height, scale));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_folder(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scale: f64,
        dir: &Path,
        max_items: usize,
    ) -> Result<Self, BrowserBlitzError> {
        let items = image_items(dir, max_items);
        let title = format!("Browser — {}", dir.display());
        Self::new(
            device, adapter, queue, format, width, height, scale, title, items,
        )
    }

    pub fn items(&self) -> &[BrowserItem] {
        &self.items
    }

    pub fn set_scale(&mut self, scale: f64) {
        if self.scale == scale {
            return;
        }
        self.scale = scale;
        self.surface.set_scale(scale);
        self.document
            .set_viewport(viewport(self.width, self.height, scale));
    }

    pub fn cached_image_count(&self) -> usize {
        self.surface.cached_image_count()
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) {
        let _guard = self.runtime.enter();
        self.document.resolve(0.0);
        self.surface
            .render(&mut self.document, device, queue, target);
    }
}

fn build_document(
    width: u32,
    height: u32,
    scale: f64,
    title: &str,
    items: &[BrowserItem],
    thumbnails: &[Option<std::path::PathBuf>],
) -> HtmlDocument {
    let mut document = HtmlDocument::from_html(
        &markup::build_html(title, items, thumbnails),
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            net_provider: Some(blitz_net::Provider::shared(None)),
            ..Default::default()
        },
    );
    document.set_viewport(viewport(width, height, scale));
    document.resolve(0.0);
    document
}

fn viewport(width: u32, height: u32, scale: f64) -> Viewport {
    Viewport {
        window_size: (width, height),
        hidpi_scale: scale as f32,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_projects_media_library_as_rn_browser_chrome() {
        let items = vec![
            BrowserItem {
                id: "root-0:a.png".into(),
                name: "a.png".into(),
                kind: "image",
                path: "/tmp/mocks/a.png".into(),
                asset_type: "image/png",
            },
            BrowserItem {
                id: "root-0:b.png".into(),
                name: "b.png".into(),
                kind: "image",
                path: "/tmp/mocks/b.png".into(),
                asset_type: "image/png",
            },
        ];
        let html = markup::build_html(
            "Browser — mocks",
            &items,
            &[Some("/tmp/thumbs/a.png".into()), None],
        );

        for fragment in [
            r#"class="panelHeader""#,
            r#"class="tabRow""#,
            r#"Search media"#,
            r#"class="sourceRail""#,
            r#"class="resultsHeader""#,
            r#"class="browserCard""#,
            r#"file:///tmp/thumbs/a.png"#,
            r#"<span class="effectName">b.png</span>"#,
            r#"<span class="effectTags">image</span>"#,
        ] {
            assert!(html.contains(fragment), "{fragment} が出ていない");
        }
        assert!(!html.contains("file:///tmp/mocks/a.png"));
    }

}
