//! Browserパネル(Blitz): フォルダを参照してサムネイル格子を出し、項目を掴んでドラッグする。
//!
//! `N-MEDIA-PICK` / `N-PROJECT-ENTRY`(ファイルを選ぶ入口が無い)への代替。
//! 出発点は実証済みの `spikes/blitz-probe/src/bin/browser_panel.rs`。
//!
//! ## 再実装しないこと
//! フォルダ参照・メディア一覧・種別判定・path解決の意味は
//! `crate::media_library`(553行)が既に持っている。本moduleはその投影を
//! HTML/CSSで描くだけで、走査も拡張子表も持たない(`library_view.rs`)。
//! `browser_host.rs` / `browser_host_runtime.rs` は wry WebView 側のhostであり、
//! 読むだけで触らない。
//!
//! ## このmoduleが決めないこと
//! - **配置intentの意味**: 離した先でDocumentに何が起きるかは未決。
//!   `DomainIntent` / `Document` schema へは触らない。release は観測を返すだけ。
//! - **native file dialog**: 作らない。走査するpathは呼び手が渡す。
//! - **色・寸法・間隔**: `theme.rs` は timeline_egui/theme.rs と probe からの写しだけ。
//!
//! ## 実装が踏んでいる罠(実測記録: docs/reviews/2026-08-15-blitz-ui-runtime-probe.md)
//! 1. `ImageManager` の `cache` / `texture_bindings` はフレームを跨いで保持する → `render.rs`
//! 2. `blitz_net::Provider` は Tokio reactor を要求する → 本fileが runtime を張る
//! 3. 画像はCSS表示サイズではなく**元解像度**でatlasを消費する → `thumbnail.rs` で縮小実体を
//!    先に作り、`<img>` はそれを指す(枚数上限のロジックは `library_view.rs` に残してある)
//!
//! ## 合成側の実測(P12: spikes/blitz-probe/src/bin/alpha_composite.rs)
//! - `body` に背景色を置くと viewport 全面が不透明で塗り潰される → 地色は要素側へ(`markup.rs`)
//! - 合成先textureは `Rgba8Unorm`。`Rgba8UnormSrgb` にすると値が浮く

mod interaction;
mod library_view;
mod markup;
pub mod render;
mod theme;
mod thumbnail;

use atomic_refcell::AtomicRefCell;
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::Modifiers;
use std::path::Path;
use std::sync::Arc;

pub use interaction::{BrowserDrag, BrowserDragRelease};
pub use library_view::{image_items, BrowserItem, DEFAULT_MAX_ITEMS};

/// パネル生成の失敗。
#[derive(Debug, thiserror::Error)]
pub enum BrowserBlitzError {
    /// `blitz_net::Provider` は Tokio reactor が無いとpanicするため、
    /// runtime を張れない時点で作らずに返す(罠2)。
    #[error("tokio runtime for blitz-net is unavailable")]
    Runtime(#[source] std::io::Error),
}

/// Browserパネル1面。wgpu textureへ描く。
pub struct BrowserBlitzPanel {
    document: HtmlDocument,
    surface: render::BlitzSurface,
    /// blitz-net の fetch が走る reactor。パネルより長く生かす必要がある(罠2)。
    runtime: tokio::runtime::Runtime,
    title: String,
    items: Vec<BrowserItem>,
    /// `items` と同じ長さ・同じ並びの縮小実体path(`thumbnail.rs`)。
    /// `<img>` はこちらを指す。作れなかった項目は `None` で、画像なしで描かれる。
    thumbnails: Vec<Option<std::path::PathBuf>>,
    highlight: markup::GridHighlight,
    drag: Option<BrowserDrag>,
    width: u32,
    height: u32,
    dirty: bool,
}

impl BrowserBlitzPanel {
    /// 走査済みの項目からパネルを作る。`title` はヘッダ帯に出す文字。
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
        let highlight = markup::GridHighlight::default();
        // 縮小実体はDocumentを組む前に用意する(2回目以降はディスクのキャッシュを掴むだけ)。
        let thumbnails = thumbnail::prepare(&items);
        let document = {
            let _guard = runtime.enter();
            build_document(width, height, &title, &items, &thumbnails, highlight)
        };
        Ok(Self {
            document,
            surface: {
                // `width`/`height` は **CSS px**(ポインタ判定もこの空間)。
                // texture は物理px で要る。hidpiの面へ等倍で描くための倍率がここ。
                let mut surface = render::BlitzSurface::new(
                    device,
                    adapter,
                    queue,
                    format,
                    ((width as f64) * scale).round() as u32,
                    ((height as f64) * scale).round() as u32,
                );
                surface.set_scale(scale);
                surface
            },
            runtime,
            title,
            items,
            thumbnails,
            highlight,
            drag: None,
            width,
            height,
            dirty: false,
        })
    }

    /// フォルダを参照してパネルを作る。走査と種別判定は `media_library` が持つ。
    /// native file dialog は使わない(C6 NON-GOALS)。
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

    /// ドラッグ中の項目。ドラッグ中の絵はホスト側が描く(probe P6の原則)。
    pub fn drag(&self) -> Option<BrowserDrag> {
        self.drag
    }

    pub fn selected(&self) -> Option<usize> {
        self.highlight.selected
    }

    /// paint scale。hidpiの面へ等倍で描くときに `pixels_per_point` を渡す。
    /// 文書側の `Viewport::hidpi_scale` と同じ値にすること。
    /// **この面は文書の大きさを後から変えられない**ので、倍率を変えるだけでは
    /// レイアウトのCSS px幅は変わらない。呼び出し側はパネルごと作り直すこと。
    pub fn set_scale(&mut self, scale: f64) {
        self.surface.set_scale(scale);
    }

    /// 罠1の再発観測窓。フレームを跨いで保持できていれば同じフォルダで増え続けない。
    pub fn cached_image_count(&self) -> usize {
        self.surface.cached_image_count()
    }

    /// pointer移動。Blitz側の `:hover` はこの転送で効く。
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        {
            let mut driver = EventDriver::new(&mut *self.document, NoopEventHandler);
            driver.handle_ui_event(UiEvent::PointerMove(pointer_at(x as f32, y as f32)));
        }
        if let Some(drag) = self.drag.as_mut() {
            drag.pointer = (x, y);
        }
    }

    /// 押した。項目の上なら選択してドラッグを開始する。
    pub fn pointer_down(&mut self, x: f64, y: f64) {
        let hit = interaction::index_at(x, y, self.items.len());
        if self.highlight.selected != hit {
            self.highlight.selected = hit;
            self.dirty = true;
        }
        if let Some(index) = hit {
            self.drag = Some(BrowserDrag {
                index,
                pointer: (x, y),
            });
            if self.highlight.dragging != Some(index) {
                self.highlight.dragging = Some(index);
                self.dirty = true;
            }
        }
    }

    /// 離した。**ここで配置は確定しない。** 何が起きるべきかは未決(C6のRETURN事項)。
    pub fn pointer_up(&mut self, x: f64, y: f64) -> Option<BrowserDragRelease> {
        let drag = self.drag.take()?;
        if self.highlight.dragging.is_some() {
            self.highlight.dragging = None;
            self.dirty = true;
        }
        let item = self.items.get(drag.index)?.clone();
        Some(BrowserDragRelease {
            item,
            pointer: (x, y),
            inside_panel: x >= 0.0 && y >= 0.0 && x < self.width as f64 && y < self.height as f64,
        })
    }

    /// ドラッグを取り消す。pointer captureを失った時などに呼ぶ。
    pub fn cancel_drag(&mut self) {
        if self.drag.take().is_some() {
            self.highlight.dragging = None;
            self.dirty = true;
        }
    }

    /// 1フレーム描く。状態が変わっていた時だけDOMを組み直す。
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) {
        let _guard = self.runtime.enter();
        if self.dirty {
            self.document = build_document(
                self.width,
                self.height,
                &self.title,
                &self.items,
                &self.thumbnails,
                self.highlight,
            );
            self.dirty = false;
        }
        self.document.resolve(0.0);
        self.surface
            .render(&mut self.document, device, queue, target);
    }
}

fn build_document(
    width: u32,
    height: u32,
    title: &str,
    items: &[BrowserItem],
    thumbnails: &[Option<std::path::PathBuf>],
    highlight: markup::GridHighlight,
) -> HtmlDocument {
    let mut document = HtmlDocument::from_html(
        &markup::build_html(width, height, title, items, thumbnails, highlight),
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            // file スキームを std::fs::read で処理する経路。reactor必須(罠2)。
            net_provider: Some(blitz_net::Provider::shared(None)),
            ..Default::default()
        },
    );
    document.set_viewport(Viewport {
        window_size: (width, height),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    document.resolve(0.0);
    document
}

fn pointer_at(x: f32, y: f32) -> BlitzPointerEvent {
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
        buttons: MouseEventButtons::None,
        mods: Modifiers::default(),
        details: PointerDetails {
            pressure: 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_hit_maps_to_row_major_index() {
        // 6列(theme::COLS)。2行目の先頭は index 6。
        let y = theme::TOP + theme::CELL_H + theme::PAD + 1.0;
        assert_eq!(interaction::index_at(theme::PAD + 1.0, y, 45), Some(6));
    }

    #[test]
    fn grid_hit_rejects_header_band() {
        assert_eq!(interaction::index_at(20.0, 4.0, 45), None);
    }

    #[test]
    fn grid_hit_rejects_index_beyond_items() {
        let y = theme::TOP + 4.0 * (theme::CELL_H + theme::PAD) + 1.0;
        assert_eq!(interaction::index_at(theme::PAD + 1.0, y, 3), None);
    }

    /// `<img>` は縮小実体だけを指す。作れなかった項目は**元画像へ戻さず**画像なしで描く。
    #[test]
    fn markup_points_at_thumbnails_and_never_at_the_source() {
        let items = vec![
            BrowserItem {
                id: "root-0:a.png".into(),
                name: "a.png".into(),
                kind: "image",
                path: "/tmp/source-a.png".into(),
                asset_type: "image/png",
            },
            BrowserItem {
                id: "root-0:b.png".into(),
                name: "b.png".into(),
                kind: "image",
                path: "/tmp/source-b.png".into(),
                asset_type: "image/png",
            },
        ];
        let html = markup::build_html(
            900,
            520,
            "t",
            &items,
            // 2枚目は生成に失敗した項目。
            &[Some("/tmp/thumbs/aaaa-248x168.png".into()), None],
            markup::GridHighlight::default(),
        );
        assert!(html.contains("file:///tmp/thumbs/aaaa-248x168.png"));
        assert!(!html.contains("source-a.png"));
        assert!(!html.contains("source-b.png"));
        // 画像なしのcardが1枚。`<img>` は1個だけ。
        assert_eq!(html.matches("<img").count(), 1);
        assert!(html.contains(r#"<div class="nm">b.png</div>"#));
    }

    #[test]
    fn markup_uses_only_probe_verified_css_properties() {
        // spikes/blitz-probe で実際に効いたプロパティ以外を混ぜない。
        const ALLOWED: [&str; 17] = [
            "margin",
            "padding",
            "width",
            "height",
            "background",
            "font-family",
            "color",
            "position",
            "left",
            "top",
            "border",
            "border-bottom",
            "border-color",
            "overflow",
            "white-space",
            "font-size",
            "object-fit",
        ];
        let html = markup::build_html(
            900,
            520,
            "t",
            &[BrowserItem {
                id: "root-0:a.png".into(),
                name: "a.png".into(),
                kind: "image",
                path: "/tmp/a.png".into(),
                asset_type: "image/png",
            }],
            &[Some("/tmp/a-thumb.png".into())],
            markup::GridHighlight::default(),
        );
        let style = html
            .split("<style>")
            .nth(1)
            .and_then(|rest| rest.split("</style>").next())
            .expect("style block");
        // セレクタ(`.card:hover` 等)を先に落としてから宣言を見る。
        for block in style.split('}') {
            let Some((_selector, declarations)) = block.split_once('{') else {
                continue;
            };
            for declaration in declarations.split(';') {
                let Some((property, _)) = declaration.split_once(':') else {
                    continue;
                };
                let property = property.trim();
                if property.is_empty() {
                    continue;
                }
                assert!(
                    ALLOWED.contains(&property),
                    "probe未検証のCSSプロパティ: {property}"
                );
            }
        }
    }
}
