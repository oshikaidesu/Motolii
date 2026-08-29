use std::sync::{Arc, Mutex};

use crate::playback::Clock;
use crate::session::Selection;
use crate::tokens;
use anyrender::{PaintRef, PaintScene, ResourceId};
use blitz_traits::events::UiEvent;
use dioxus_native::prelude::{Signal, WritableExt};
use keyboard_types::Modifiers;
use motolii_store::{property, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime, StoreView, Value};
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

/// どのハンドルを掴んだか。`bool`は「箱のmax側(x1/y1)を掴んだか」。
#[derive(Clone, Copy, Debug)]
enum GizmoMode {
    Move,
    ScaleCorner { sx: bool, sy: bool },
    ScaleEdge { axis_x: bool, positive: bool },
    Rotate,
}

struct GizmoDrag {
    layer: LayerId,
    mode: GizmoMode,
    /// ドラッグ開始点のcomp座標。
    grab: (f64, f64),
    orig_position: (f64, f64),
    orig_rotation: f64,
    anchor: (f64, f64),
    /// pre-scale local size(`selection_geom_in`参照)。
    natural: (f64, f64),
    /// ドラッグ開始時の箱(comp座標、回転無視——表示と同じ簡略化)。
    orig_box: (f64, f64, f64, f64),
}

pub struct StageWidget {
    state: State,
    frames: u64,
    clock: Arc<Clock>,
    doc: Arc<Mutex<Document>>,
    selection: Selection,
    selected_mirror: Signal<Option<LayerId>>,
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
        selected_mirror: Signal<Option<LayerId>>,
        revision: Signal<u32>,
    ) -> Self {
        Self {
            state: State::Suspended,
            frames: 0,
            clock,
            doc,
            selection,
            selected_mirror,
            fit: Fit::default(),
            drag: None,
            revision,
        }
    }

    fn current_rt(&self) -> RationalTime {
        let t_sec = self.clock.now_sec();
        RationalTime::try_new((t_sec * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO)
    }

    fn selection_geom(&self, layer: LayerId) -> Option<SelGeom> {
        let State::Active(active) = &self.state else {
            return None;
        };
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        selection_geom_in(&active.engine, &view, layer, self.current_rt())
    }

}

/// 選択層の位置/anchor/scale/rotation とcomp座標での箱(x, y, w, h)。
/// 箱は回転を無視した軸並行(表示の簡略化と同じ)——
/// world = position + scale*(local - anchor) から角を出すだけ(裁定58のaffineの逆)。
struct SelGeom {
    position: (f64, f64),
    anchor: (f64, f64),
    rotation: f64,
    /// pre-scale local size。板のサイズは`Engine::selected_layer_size`任せ
    /// ——front は layer の種類(`LayerSource`)を知らない。
    natural: (f64, f64),
    box_: (f64, f64, f64, f64),
}

fn vec2_at(view: &StoreView<'_>, layer: LayerId, name: &str, rt: RationalTime, default: (f64, f64)) -> (f64, f64) {
    let Ok(prop) = PropertyId::new(name) else { return default };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => default,
    }
}

fn f64_at(view: &StoreView<'_>, layer: LayerId, name: &str, rt: RationalTime, default: f64) -> f64 {
    let Ok(prop) = PropertyId::new(name) else { return default };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::F64(v)) => v,
        _ => default,
    }
}

/// timelineのbandが無い時刻ではNone(裁定どおり)。
fn selection_geom_in(
    engine: &Engine,
    view: &StoreView<'_>,
    layer: LayerId,
    rt: RationalTime,
) -> Option<SelGeom> {
    let meta = view.meta(layer).ok().flatten()?;
    let fps = view.composition().ok().flatten()?.fps;
    let frame = rt.try_to_frame_floor(fps).ok()?;
    if !meta.timing.covers(frame) {
        return None;
    }
    let position = vec2_at(view, layer, property::POSITION, rt, (0.0, 0.0));
    let anchor = vec2_at(view, layer, property::ANCHOR, rt, (0.0, 0.0));
    let scale = vec2_at(view, layer, property::SCALE, rt, (1.0, 1.0));
    let rotation = f64_at(view, layer, property::ROTATION, rt, 0.0);
    let [w0, h0] = engine.selected_layer_size(view, layer, rt)?;
    let natural = (w0 as f64, h0 as f64);
    let box_ = (
        position.0 - scale.0 * anchor.0,
        position.1 - scale.1 * anchor.1,
        scale.0 * natural.0,
        scale.1 * natural.1,
    );
    Some(SelGeom { position, anchor, rotation, natural, box_ })
}

/// 掴んだハンドルに応じて新しい`(scale, position)`を出す。固定点は対角(Alt=中心)。
/// Shift=等比(比率固定)。1ジェスチャで1回だけ呼ばれ、結果はtransient/確定の両方に使う。
fn compute_scale(
    orig_box: (f64, f64, f64, f64),
    natural: (f64, f64),
    anchor: (f64, f64),
    mode: GizmoMode,
    cur: (f64, f64),
    shift: bool,
    alt: bool,
) -> ((f64, f64), (f64, f64)) {
    let (bx, by, bw, bh) = orig_box;
    let (x0, y0, x1, y1) = (bx, by, bx + bw, by + bh);
    let (cx, cy) = cur;
    let (mut nx0, mut ny0, mut nx1, mut ny1) = (x0, y0, x1, y1);
    match mode {
        GizmoMode::ScaleCorner { sx, sy } => {
            if sx { nx1 = cx } else { nx0 = cx }
            if sy { ny1 = cy } else { ny0 = cy }
            if alt {
                let (ccx, ccy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
                let (hx, hy) = ((cx - ccx).abs(), (cy - ccy).abs());
                (nx0, nx1) = (ccx - hx, ccx + hx);
                (ny0, ny1) = (ccy - hy, ccy + hy);
            }
            if shift {
                let (fx, fy) = if alt {
                    ((x0 + x1) * 0.5, (y0 + y1) * 0.5)
                } else {
                    (if sx { x0 } else { x1 }, if sy { y0 } else { y1 })
                };
                let orig_w = (x1 - x0).abs().max(1e-6);
                let orig_h = (y1 - y0).abs().max(1e-6);
                let w = (nx1 - nx0).abs();
                let h = (ny1 - ny0).abs();
                let k = (w / orig_w).max(h / orig_h);
                let (new_w, new_h) = (orig_w * k, orig_h * k);
                if alt {
                    (nx0, nx1) = (fx - new_w * 0.5, fx + new_w * 0.5);
                    (ny0, ny1) = (fy - new_h * 0.5, fy + new_h * 0.5);
                } else {
                    if sx { nx1 = fx + new_w } else { nx0 = fx - new_w }
                    if sy { ny1 = fy + new_h } else { ny0 = fy - new_h }
                }
            }
        }
        GizmoMode::ScaleEdge { axis_x, positive } => {
            if axis_x {
                if positive { nx1 = cx } else { nx0 = cx }
                if alt {
                    let ccx = (x0 + x1) * 0.5;
                    let hx = (cx - ccx).abs();
                    (nx0, nx1) = (ccx - hx, ccx + hx);
                }
            } else {
                if positive { ny1 = cy } else { ny0 = cy }
                if alt {
                    let ccy = (y0 + y1) * 0.5;
                    let hy = (cy - ccy).abs();
                    (ny0, ny1) = (ccy - hy, ccy + hy);
                }
            }
        }
        GizmoMode::Move | GizmoMode::Rotate => {}
    }
    let (nbx, nby) = (nx0.min(nx1), ny0.min(ny1));
    let (nbw, nbh) = ((nx1 - nx0).abs().max(0.01), (ny1 - ny0).abs().max(0.01));
    let scale = (nbw / natural.0.max(0.01), nbh / natural.1.max(0.01));
    let position = (nbx + scale.0 * anchor.0, nby + scale.1 * anchor.1);
    (scale, position)
}

/// 回転の中心はanchorのworld座標——`world(anchor_local) = position`
/// (裁定58の affine で local=anchorを代入すると anchor 項が打ち消し合う)なので、
/// anchor値そのものを読まずに`position`を使ってよい。Shift=15度刻みへスナップ。
fn rotate_around(center: (f64, f64), angle_deg: f64, p: (f64, f64)) -> (f64, f64) {
    let a = angle_deg.to_radians();
    let (dx, dy) = (p.0 - center.0, p.1 - center.1);
    let (s, c) = a.sin_cos();
    (center.0 + dx * c - dy * s, center.1 + dx * s + dy * c)
}

fn compute_rotation(center: (f64, f64), grab: (f64, f64), cur: (f64, f64), orig_rotation: f64, shift: bool) -> f64 {
    let ang0 = (grab.1 - center.1).atan2(grab.0 - center.0);
    let ang1 = (cur.1 - center.1).atan2(cur.0 - center.0);
    let mut r = orig_rotation + (ang1 - ang0).to_degrees();
    if shift {
        r = (r / 15.0).round() * 15.0;
    }
    r
}

/// `SetTrack`用のIntent。track が無ければ`RationalTime::ZERO`(静的値)、
/// あれば`t`(プレイヘッド)——`write_key`(`inspector.rs`)と同じ法。
fn track_intent(doc: &Document, layer: LayerId, name: &str, value: Value, t: RationalTime) -> Option<Intent> {
    let prop = PropertyId::new(name).ok()?;
    let existing = doc.view().track(layer, &prop).ok().flatten();
    let is_new = existing.as_ref().map(|tr| tr.keys().is_empty()).unwrap_or(true);
    let mut track = existing.unwrap_or_else(KeyframeTrack::new);
    let key_t = if is_new { RationalTime::ZERO } else { t };
    track.insert(Keyframe { t: key_t, value, interp: Interp::Linear, spatial: None });
    Some(Intent::SetTrack { layer, property: prop, track })
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
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                if let Some(layer) = self.selection.get() {
                    if let Some(geom) = self.selection_geom(layer) {
                        // 見えている位置(回転済み)で掴めるように、どのハンドルかの判定だけ
                        // box_と同じ未回転(local)座標系で行う——カーソルをposition中心に逆回転。
                        // ドラッグ開始後の計算(grab/cur)は従来どおり生comp座標のまま。
                        let (lx, ly) = rotate_around(geom.position, -geom.rotation, (cx, cy));
                        let (bx, by, bw, bh) = geom.box_;
                        let (x0, y0, x1, y1) = (bx, by, bx + bw, by + bh);
                        let tol = 8.0 / self.fit.s.max(1e-6);
                        let near = |px: f64, py: f64| (lx - px).abs() <= tol && (ly - py).abs() <= tol;
                        let corners = [(x0, y0, false, false), (x1, y0, true, false), (x0, y1, false, true), (x1, y1, true, true)];
                        let mut mode = corners
                            .into_iter()
                            .find(|&(px, py, ..)| near(px, py))
                            .map(|(_, _, sx, sy)| GizmoMode::ScaleCorner { sx, sy });
                        if mode.is_none() {
                            let edges = [
                                ((x0 + x1) * 0.5, y0, GizmoMode::ScaleEdge { axis_x: false, positive: false }),
                                ((x0 + x1) * 0.5, y1, GizmoMode::ScaleEdge { axis_x: false, positive: true }),
                                (x0, (y0 + y1) * 0.5, GizmoMode::ScaleEdge { axis_x: true, positive: false }),
                                (x1, (y0 + y1) * 0.5, GizmoMode::ScaleEdge { axis_x: true, positive: true }),
                            ];
                            mode = edges.into_iter().find(|&(px, py, _)| near(px, py)).map(|(_, _, m)| m);
                        }
                        if mode.is_none() {
                            if lx >= bx && lx <= bx + bw && ly >= by && ly <= by + bh {
                                mode = Some(GizmoMode::Move);
                            } else {
                                let margin = 24.0 / self.fit.s.max(1e-6);
                                if lx >= bx - margin && lx <= bx + bw + margin && ly >= by - margin && ly <= by + bh + margin {
                                    mode = Some(GizmoMode::Rotate);
                                }
                            }
                        }
                        if let Some(mode) = mode {
                            self.drag = Some(GizmoDrag {
                                layer,
                                mode,
                                grab: (cx, cy),
                                orig_position: geom.position,
                                orig_rotation: geom.rotation,
                                anchor: geom.anchor,
                                natural: geom.natural,
                                orig_box: geom.box_,
                            });
                            return;
                        }
                    }
                }
                // ハンドルに当たらなかった——キャンバス上の層を直接拾う。手前(order大)が勝つ。
                // 何も当たらなければ選択はそのまま(Timelineの空白クリックと同じ文法)。
                let State::Active(active) = &self.state else { return };
                let doc = self.doc.lock().unwrap();
                let view = doc.view();
                let rt = self.current_rt();
                let Ok(layers) = view.resolved_layers(rt) else { return };
                let mut hit: Option<(i16, LayerId)> = None;
                for layer in &layers {
                    let Some(geom) = selection_geom_in(&active.engine, &view, layer.id, rt) else { continue };
                    let (bx, by, bw, bh) = geom.box_;
                    let (lx, ly) = rotate_around(geom.position, -geom.rotation, (cx, cy));
                    if lx < bx || lx > bx + bw || ly < by || ly > by + bh {
                        continue;
                    }
                    let order = layer.placement.order;
                    if hit.map(|(o, _)| order > o).unwrap_or(true) {
                        hit = Some((order, layer.id));
                    }
                }
                drop(view);
                drop(doc);
                if let Some((_, layer)) = hit {
                    if p.mods.contains(Modifiers::META) {
                        self.selection.toggle(layer);
                    } else {
                        self.selection.set(Some(layer));
                    }
                    self.selected_mirror.set(self.selection.get());
                }
            }
            UiEvent::PointerMove(p) => {
                let Some(drag) = self.drag.as_ref() else { return };
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                let shift = p.mods.contains(Modifiers::SHIFT);
                let alt = p.mods.contains(Modifiers::ALT);
                let mut doc = self.doc.lock().unwrap();
                match drag.mode {
                    GizmoMode::Move => {
                        let Ok(position_prop) = PropertyId::new(property::POSITION) else { return };
                        let (dx, dy) = (cx - drag.grab.0, cy - drag.grab.1);
                        let new_pos = (drag.orig_position.0 + dx, drag.orig_position.1 + dy);
                        doc.set_transient(drag.layer, position_prop, Value::Vec2([new_pos.0, new_pos.1]));
                    }
                    GizmoMode::ScaleCorner { .. } | GizmoMode::ScaleEdge { .. } => {
                        let Ok(scale_prop) = PropertyId::new(property::SCALE) else { return };
                        let Ok(position_prop) = PropertyId::new(property::POSITION) else { return };
                        let (new_scale, new_pos) =
                            compute_scale(drag.orig_box, drag.natural, drag.anchor, drag.mode, (cx, cy), shift, alt);
                        doc.set_transient(drag.layer, scale_prop, Value::Vec2([new_scale.0, new_scale.1]));
                        doc.set_transient(drag.layer, position_prop, Value::Vec2([new_pos.0, new_pos.1]));
                    }
                    GizmoMode::Rotate => {
                        let Ok(rotation_prop) = PropertyId::new(property::ROTATION) else { return };
                        let r = compute_rotation(drag.orig_position, drag.grab, (cx, cy), drag.orig_rotation, shift);
                        doc.set_transient(drag.layer, rotation_prop, Value::F64(r));
                    }
                }
            }
            UiEvent::PointerUp(p) => {
                let Some(drag) = self.drag.take() else { return };
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                let shift = p.mods.contains(Modifiers::SHIFT);
                let alt = p.mods.contains(Modifiers::ALT);
                let rt = self.current_rt();
                let mut doc = self.doc.lock().unwrap();
                let mut intents = Vec::new();
                let touched: &[&str] = match drag.mode {
                    GizmoMode::Move => {
                        let (dx, dy) = (cx - drag.grab.0, cy - drag.grab.1);
                        let new_pos = (drag.orig_position.0 + dx, drag.orig_position.1 + dy);
                        intents.extend(track_intent(&doc, drag.layer, property::POSITION, Value::Vec2([new_pos.0, new_pos.1]), rt));
                        &[property::POSITION]
                    }
                    GizmoMode::ScaleCorner { .. } | GizmoMode::ScaleEdge { .. } => {
                        let (new_scale, new_pos) =
                            compute_scale(drag.orig_box, drag.natural, drag.anchor, drag.mode, (cx, cy), shift, alt);
                        intents.extend(track_intent(&doc, drag.layer, property::SCALE, Value::Vec2([new_scale.0, new_scale.1]), rt));
                        intents.extend(track_intent(&doc, drag.layer, property::POSITION, Value::Vec2([new_pos.0, new_pos.1]), rt));
                        &[property::SCALE, property::POSITION]
                    }
                    GizmoMode::Rotate => {
                        let r = compute_rotation(drag.orig_position, drag.grab, (cx, cy), drag.orig_rotation, shift);
                        intents.extend(track_intent(&doc, drag.layer, property::ROTATION, Value::F64(r), rt));
                        &[property::ROTATION]
                    }
                };
                match doc.apply_all(intents) {
                    Ok(_) => {
                        *self.revision.write() += 1;
                        println!("PROBE room=write verdict=gizmo-{:?} layer={:?}", drag.mode, drag.layer);
                    }
                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                }
                for name in touched {
                    if let Ok(prop) = PropertyId::new(name) {
                        doc.clear_transient(drag.layer, &prop);
                    }
                }
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
        let primary_layer = self.selection.get();
        let selected_box = primary_layer
            .and_then(|layer| selection_geom_in(&active.engine, &view, layer, rt))
            .map(|geom| (geom.box_, geom.position, geom.rotation));
        let secondary_boxes: Vec<_> = self
            .selection
            .all()
            .into_iter()
            .filter(|l| Some(*l) != primary_layer)
            .filter_map(|l| selection_geom_in(&active.engine, &view, l, rt))
            .map(|geom| geom.box_)
            .collect();

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

        // 主選択以外の枠 — ハンドルもギズモも無い、細い縁だけ。
        for (bx, by, bw, bh) in &secondary_boxes {
            let (x0, y0) = (fx + bx * s, fy + by * s);
            let (x1, y1) = (fx + (bx + bw) * s, fy + (by + bh) * s);
            let th = 1.0;
            let edges = [
                Rect::from_origin_size((x0, y0), (x1 - x0, th)),
                Rect::from_origin_size((x0, y1 - th), (x1 - x0, th)),
                Rect::from_origin_size((x0, y0), (th, y1 - y0)),
                Rect::from_origin_size((x1 - th, y0), (th, y1 - y0)),
            ];
            for edge in &edges {
                scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, edge);
            }
        }

        if let Some(((bx, by, bw, bh), position, rotation)) = selected_box {
            let (x0, y0) = (fx + bx * s, fy + by * s);
            let (x1, y1) = (fx + (bx + bw) * s, fy + (by + bh) * s);
            // 回転はcomp/display両方でposition中心・同じ角度(等方scaleなので角度は不変)。
            // 枠の絵と当たり判定はともにpaint/PointerDownでこの中心を使う。
            let pivot = (fx + position.0 * s, fy + position.1 * s);
            let rot = Affine::translate(pivot)
                * Affine::rotate(rotation.to_radians())
                * Affine::translate((-pivot.0, -pivot.1));
            let th = 1.5;
            let edges = [
                Rect::from_origin_size((x0, y0), (x1 - x0, th)),
                Rect::from_origin_size((x0, y1 - th), (x1 - x0, th)),
                Rect::from_origin_size((x0, y0), (th, y1 - y0)),
                Rect::from_origin_size((x1 - th, y0), (th, y1 - y0)),
            ];
            for edge in &edges {
                scene.fill(Fill::NonZero, rot, PaintRef::Solid(c(tokens::ACCENT)), None, edge);
            }
            let hs = 6.0;
            let handles = [
                (x0, y0), (x1, y0), (x0, y1), (x1, y1),
                ((x0 + x1) * 0.5, y0), ((x0 + x1) * 0.5, y1),
                (x0, (y0 + y1) * 0.5), (x1, (y0 + y1) * 0.5),
            ];
            for (cx, cy) in handles {
                let handle_rect = Rect::from_origin_size((cx - hs * 0.5, cy - hs * 0.5), (hs, hs));
                scene.fill(Fill::NonZero, rot, PaintRef::Solid(c(tokens::ACCENT)), None, &handle_rect);
            }
        }

        scene
    }
}
