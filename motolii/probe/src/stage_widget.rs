use std::sync::{Arc, Mutex};

use crate::playback::Clock;
use crate::session::Selection;
use crate::tokens;
use anyrender::{PaintRef, PaintScene, ResourceId};
use blitz_traits::events::UiEvent;
use dioxus_native::prelude::{Signal, WritableExt};
use motolii_store::{property, Document, Intent, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime, StoreView, Value};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use motolii_engine::Engine;
use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill, ImageBrush, ImageSampler};
use wgpu_context::DeviceHandle;

fn c(rgb: [u8; 3]) -> Color {
    Color::from_rgb8(rgb[0], rgb[1], rgb[2])
}

/// texture画素→表示pxの変換。paintが毎フレーム更新し、handle_eventが逆変換に使う。
/// 論理px単位(deviceで計算した値を`k`で割って揃える) — eventの座標系がこちら側。
#[derive(Clone, Copy)]
struct Fit {
    s: f64,
    fx: f64,
    fy: f64,
}

impl Default for Fit {
    fn default() -> Self {
        Self { s: 1.0, fx: 0.0, fy: 0.0 }
    }
}

impl Fit {
    fn to_comp(&self, x: f64, y: f64) -> (f64, f64) {
        ((x - self.fx) / self.s, (y - self.fy) / self.s)
    }
}

struct GizmoDrag {
    layer: LayerId,
    /// ドラッグ開始点のcomp座標。
    grab: (f64, f64),
    /// ドラッグ開始時のposition値。
    orig: (f64, f64),
}

pub struct StageWidget {
    state: State,
    frames: u64,
    clock: Arc<Clock>,
    doc: Arc<Mutex<Document>>,
    selection: Selection,
    fit: Fit,
    drag: Option<GizmoDrag>,
    revision: Signal<u32>,
}

enum State {
    Suspended,
    Active(Box<Active>),
}

struct Active {
    engine: Engine,
    displayed: Option<TexAndHandle>,
    next: Option<TexAndHandle>,
}

struct TexAndHandle {
    texture: wgpu::Texture,
    handle: ResourceId,
}

impl StageWidget {
    pub fn new(
        clock: Arc<Clock>,
        doc: Arc<Mutex<Document>>,
        selection: Selection,
        revision: Signal<u32>,
    ) -> Self {
        Self {
            state: State::Suspended,
            frames: 0,
            clock,
            doc,
            selection,
            fit: Fit::default(),
            drag: None,
            revision,
        }
    }

    fn current_rt(&self) -> RationalTime {
        let t_sec = self.clock.now_sec();
        RationalTime::try_new((t_sec * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO)
    }

    fn selection_box(&self, layer: LayerId) -> Option<(f64, f64, f64, f64)> {
        let State::Active(active) = &self.state else {
            return None;
        };
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        selection_box_in(&active.engine, &view, layer, self.current_rt())
    }

}

/// 選択層のcomp座標での枠(x, y, w, h)。timelineのbandが無い時刻ではNone(裁定どおり)。
/// 板のサイズは`Engine::selected_layer_size`任せ——front は layer の種類
/// (`LayerSource`)を知らない。
fn selection_box_in(
    engine: &Engine,
    view: &StoreView<'_>,
    layer: LayerId,
    rt: RationalTime,
) -> Option<(f64, f64, f64, f64)> {
    let meta = view.meta(layer).ok().flatten()?;
    let fps = view.composition().ok().flatten()?.fps;
    let frame = rt.try_to_frame_floor(fps).ok()?;
    if !meta.timing.covers(frame) {
        return None;
    }
    let position_prop = PropertyId::new(property::POSITION).ok()?;
    let (x, y) = match view.value_at(layer, &position_prop, rt).ok().flatten() {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => (0.0, 0.0),
    };
    let [w, h] = engine.selected_layer_size(view, layer, rt)?;
    Some((x, y, w as f64, h as f64))
}

fn create_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe-stage-target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // motolii-compositor::presentable::PRESENTABLE_FORMAT(render_frame_intoの
        // check_presentable_targetが要求する format)と同じ Rgba8UnormSrgb。
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // COPY_SRC: anyrender_velloは登録textureをcopyで取り込む。無いとsubmit全体が検証エラーで落ちる。
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

impl Widget for StageWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}

    fn can_create_surfaces(&mut self, render_ctx: &mut dyn anyrender::RenderContext) {
        let Some(ctx) = render_ctx.renderer_specific_context() else {
            println!("PROBE room=stage verdict=no-renderer-context");
            return;
        };
        let Ok(device_handle) = ctx.downcast::<DeviceHandle>() else {
            println!("PROBE room=stage verdict=non-wgpu-backend");
            return;
        };
        match Engine::with_device(device_handle.device.clone(), device_handle.queue.clone()) {
            Ok(engine) => {
                println!("PROBE room=stage verdict=engine-up");
                self.state = State::Active(Box::new(Active { engine, displayed: None, next: None }));
            }
            Err(e) => println!("PROBE room=stage verdict=engine-error {e}"),
        }
    }

    fn destroy_surfaces(&mut self) {
        println!("PROBE room=stage verdict=destroy-surfaces");
        self.state = State::Suspended;
    }

    fn requires_redraw(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent) {
        match event {
            UiEvent::PointerDown(p) => {
                let Some(layer) = self.selection.get() else { return };
                let Some((bx, by, bw, bh)) = self.selection_box(layer) else { return };
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                if cx >= bx && cx <= bx + bw && cy >= by && cy <= by + bh {
                    self.drag = Some(GizmoDrag { layer, grab: (cx, cy), orig: (bx, by) });
                }
            }
            UiEvent::PointerMove(p) => {
                let Some(drag) = self.drag.as_ref() else { return };
                let Ok(position_prop) = PropertyId::new(property::POSITION) else { return };
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                let (dx, dy) = (cx - drag.grab.0, cy - drag.grab.1);
                let (new_x, new_y) = (drag.orig.0 + dx, drag.orig.1 + dy);
                let mut doc = self.doc.lock().unwrap();
                doc.set_transient(drag.layer, position_prop, Value::Vec2([new_x, new_y]));
            }
            UiEvent::PointerUp(p) => {
                let Some(drag) = self.drag.take() else { return };
                let Ok(position_prop) = PropertyId::new(property::POSITION) else { return };
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                let (dx, dy) = (cx - drag.grab.0, cy - drag.grab.1);
                let (new_x, new_y) = (drag.orig.0 + dx, drag.orig.1 + dy);
                let rt = self.current_rt();
                let mut doc = self.doc.lock().unwrap();
                let existing = doc.view().track(drag.layer, &position_prop).ok().flatten();
                let is_new = existing.as_ref().map(|tr| tr.keys().is_empty()).unwrap_or(true);
                let mut track = existing.unwrap_or_else(KeyframeTrack::new);
                let key_t = if is_new { RationalTime::ZERO } else { rt };
                track.insert(Keyframe {
                    t: key_t,
                    value: Value::Vec2([new_x, new_y]),
                    interp: motolii_store::Interp::Linear,
                    spatial: None,
                });
                match doc.apply(Intent::SetTrack { layer: drag.layer, property: position_prop.clone(), track }) {
                    Ok(_) => {
                        *self.revision.write() += 1;
                        println!(
                        "PROBE room=write verdict=gizmo-move layer={:?} ({:.1},{:.1})->({:.1},{:.1})",
                            drag.layer, drag.orig.0, drag.orig.1, new_x, new_y
                        );
                    }
                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                }
                doc.clear_transient(drag.layer, &position_prop);
            }
            _ => {}
        }
    }

    fn paint(
        &mut self,
        render_ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> anyrender::Scene {
        let mut scene = anyrender::Scene::new();
        self.frames += 1;
        let first = self.frames == 1;
        let State::Active(active) = &mut self.state else {
            if first {
                println!("PROBE room=stage verdict=paint-while-suspended");
            }
            return scene;
        };
        if width == 0 || height == 0 {
            if first {
                println!("PROBE room=stage verdict=zero-size");
            }
            return scene;
        }

        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        let Some(composition) = view.composition().ok().flatten() else {
            if first {
                println!("PROBE room=stage verdict=no-composition");
            }
            return scene;
        };
        let (cw, ch) = (composition.width, composition.height);

        if first {
            println!("PROBE room=stage verdict=first-paint {}x{} comp={}x{}", width, height, cw, ch);
        }

        if active.next.as_ref().is_some_and(|t| t.texture.width() != cw || t.texture.height() != ch) {
            let handle = active.next.take().unwrap().handle;
            render_ctx.unregister_resource(handle);
        }
        let tex_and_handle = match &active.next {
            Some(next) => next,
            None => {
                let texture = create_target(active.engine.gpu_device(), cw, ch);
                let handle = render_ctx
                    .try_register_custom_resource(Box::new(texture.clone()))
                    .expect("wgpu backend accepts wgpu textures");
                active.next = Some(TexAndHandle { texture, handle });
                active.next.as_ref().unwrap()
            }
        };
        let target = tex_and_handle.texture.clone();
        let handle = tex_and_handle.handle;

        let t_sec = self.clock.now_sec();
        let rt = RationalTime::try_new((t_sec * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);

        if let Err(e) = active.engine.render_frame_into(&view, rt, &target) {
            println!("PROBE room=stage verdict=render-error {e}");
            return scene;
        }

        // viewを落とす前に読む。ドラッグ中の追随はtransient overlayがvalue_atへ
        // 優先して乗るので担う — ここに専用の分岐は要らない。
        let selected_box = self
            .selection
            .get()
            .and_then(|layer| selection_box_in(&active.engine, &view, layer, rt));

        drop(view);
        drop(doc);
        if first {
            println!("PROBE room=stage verdict=first-submit-ok");
        }

        std::mem::swap(&mut active.next, &mut active.displayed);

        let (w, h) = (width as f64, height as f64);
        let (cw, ch) = (target.width() as f64, target.height() as f64);
        let s = (w / cw).min(h / ch);
        let (fw, fh) = (cw * s, ch * s);
        let (fx, fy) = ((w - fw) * 0.5, (h - fh) * 0.5);
        // brush空間はtexture画素のまま — fit矩形へは brush_transform で写す。
        // 矩形だけ縮めるとcompを等倍で覗く穴になる。
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush { image: handle, sampler: ImageSampler::default() }),
            Some(Affine::translate((fx, fy)) * Affine::scale(s)),
            &Rect::from_origin_size((fx, fy), (fw, fh)),
        );

        // handle_eventはelement座標(論理px)で来るので、逆変換もその単位で揃える。
        let k = if scale > 0.0 { scale } else { 1.0 };
        self.fit = Fit { s: s / k, fx: fx / k, fy: fy / k };

        if let Some((bx, by, bw, bh)) = selected_box {
            let (x0, y0) = (fx + bx * s, fy + by * s);
            let (x1, y1) = (fx + (bx + bw) * s, fy + (by + bh) * s);
            let th = 1.5;
            let edges = [
                Rect::from_origin_size((x0, y0), (x1 - x0, th)),
                Rect::from_origin_size((x0, y1 - th), (x1 - x0, th)),
                Rect::from_origin_size((x0, y0), (th, y1 - y0)),
                Rect::from_origin_size((x1 - th, y0), (th, y1 - y0)),
            ];
            for edge in &edges {
                scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, edge);
            }
            let hs = 6.0;
            for (cx, cy) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
                let handle_rect = Rect::from_origin_size((cx - hs * 0.5, cy - hs * 0.5), (hs, hs));
                scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, &handle_rect);
            }
        }

        scene
    }
}
