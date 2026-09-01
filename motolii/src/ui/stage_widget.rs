use std::sync::{Arc, Mutex};

use crate::ui::playback::Clock;
use crate::ui::session::Selection;
use crate::ui::tokens;
use anyrender::{PaintRef, PaintScene, ResourceId};
use blitz_traits::events::UiEvent;
use dioxus_native::prelude::{Signal, WritableExt};
use keyboard_types::Modifiers;
use crate::doc::store::{property, Document, Intent, Interp, Keyframe, LayerId, PropertyId, RationalTime, StoreView, Value};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use crate::render::engine::Engine;
use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill, ImageBrush, ImageSampler};
use wgpu_context::DeviceHandle;

fn c(rgb: [u8; 3]) -> Color {
    Color::from_rgb8(rgb[0], rgb[1], rgb[2])
}

/// 窓の点と世界の点の間の写像。**視点(User View)を通す**ので、
/// 画面を動かしても世界の座標は変わらない。
#[derive(Clone, Copy)]
struct Fit {
    /// 絵を窓へ収める倍率と余白(レターボックス)。
    s: f64,
    fx: f64,
    fy: f64,
    /// 世界 → 絵の中の画素。視点カメラが決める(上流の写像)。
    image_from_world: glam::Affine2,
    /// 別の奥行きで写像を作り直すための材料。
    comp: crate::doc::core::CompSpec,
    camera: crate::doc::core::ResolvedCamera,
    /// 視点を解くための素の値。**導き済みの `s`/`fx` から逆算しない** ——
    /// 逆算は単位と向きを取り違える(一晩で5回踏んだ)。
    widget: (f64, f64),
    image: (f64, f64),
    base: f64,
}

impl Fit {
    /// 指の下に在る、撮れた絵の座標。
    fn image_at(&self, sx: f64, sy: f64) -> (f64, f64) {
        let s = self.s.max(1e-9);
        ((sx - self.fx) / s, (sy - self.fy) / s)
    }

    /// `img`(絵の座標)が `screen` に来るような視点の移動量。
    /// 拡大率を変える時は変えた後の値を渡す。**素の値から組み立てるので、
    /// 単位も向きも推論しない。**
    fn pan_putting(&self, img: (f64, f64), screen: (f64, f64), zoom: f64) -> [f32; 2] {
        let s = (self.base * zoom).max(1e-9);
        [
            (((self.widget.0 - self.image.0 * s) * 0.5 + img.0 * s - screen.0) / s) as f32,
            (((self.widget.1 - self.image.1 * s) * 0.5 + img.1 * s - screen.1) / s) as f32,
        ]
    }

    /// **その奥行きの面**での写像。奥に在る物は小さく、位置もずれて見えるので、
    /// 取っ手を絵の上に置くにはこちらを使う。
    fn at_z(&self, z: f64) -> Fit {
        Fit {
            image_from_world: crate::doc::core::camera_screen_from_world_at_z(
                self.comp,
                self.camera,
                z as f32,
            ),
            ..*self
        }
    }
}

impl Default for Fit {
    fn default() -> Self {
        Self {
            s: 1.0,
            fx: 0.0,
            fy: 0.0,
            image_from_world: glam::Affine2::IDENTITY,
            comp: crate::doc::core::CompSpec { width: 1, height: 1 },
            camera: crate::doc::core::ResolvedCamera::default(),
            widget: (1.0, 1.0),
            image: (1.0, 1.0),
            base: 1.0,
        }
    }
}

impl Fit {
    fn to_comp(&self, x: f64, y: f64) -> (f64, f64) {
        let image = glam::vec2(
            ((x - self.fx) / self.s) as f32,
            ((y - self.fy) / self.s) as f32,
        );
        let world = self.image_from_world.inverse().transform_point2(image);
        (world.x as f64, world.y as f64)
    }

    /// 世界の点を窓の点へ。ギズモはこれを通して描く。
    fn to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        let image = self
            .image_from_world
            .transform_point2(glam::vec2(wx as f32, wy as f32));
        (
            self.fx + image.x as f64 * self.s,
            self.fy + image.y as f64 * self.s,
        )
    }
}

#[derive(Clone, Copy, Debug)]
enum GizmoMode {
    Move,
    ScaleCorner { sx: bool, sy: bool },
    ScaleEdge { axis_x: bool, positive: bool },
    Rotate,
    /// 3D: 横で rotation.y、縦で rotation.x を回す。
    Orbit { axis_x: bool },
    /// 3D: 縦で position.z を動かす。
    Depth,
}

struct CameraDrag {
    grab: (f64, f64),
    orig_center: (f64, f64),
    /// 世界を見る側(視点)か、書き出しの枠(Document のカメラ)か。
    export_frame: bool,
    /// 最後に動かした先。動かしていなければ None(→ 何も書かない)。
    last: Option<(f64, f64)>,
    /// 掴んだ時に指の下に在った、**撮れた絵の座標**。視点はこれを指に付けて動かす。
    grab_image: (f64, f64),
}

struct GizmoDrag {
    layer: LayerId,
    mode: GizmoMode,
    grab: (f64, f64),
    orig_position: (f64, f64),
    orig_rotation: f64,
    orig_rotation_xy: (f64, f64),
    orig_z: f64,
    anchor: (f64, f64),
    natural: (f64, f64),
    orig_box: (f64, f64, f64, f64),
    /// 掴んだ時の奥行き。掴んでいる間、写像はこの面に固定する
    /// (奥行きを動かしている最中に写像まで動くと、指と絵が食い違う)。
    fit_z: f64,
    /// 最後に**動かした**先。動かしていなければ None。
    /// 離した時の座標から差分を取り直すと、掴んでいる間に要素の座標系がずれた分だけ
    /// 勝手に動く。動かしていないなら1画素も動かさない。
    last: Option<(f64, f64)>,
}

pub(super) struct StageWidget {
    state: State,
    frames: u64,
    clock: Arc<Clock>,
    doc: Arc<Mutex<Document>>,
    selection: Selection,
    selected_mirror: Signal<Option<LayerId>>,
    fit: Fit,
    drag: Option<GizmoDrag>,
    /// 何も掴んでいない所からのドラッグ = カメラを動かす。
    camera_drag: Option<CameraDrag>,
    revision: Signal<u32>,
    selected_size: Arc<Mutex<Option<[f32; 2]>>>,
    view_camera: Arc<Mutex<crate::render::engine::ObservationCamera>>,
    rings: Arc<std::sync::atomic::AtomicBool>,
    frame_dim: Arc<std::sync::atomic::AtomicU32>,
    cancel: Arc<std::sync::atomic::AtomicU32>,
    gesture_active: Arc<std::sync::atomic::AtomicBool>,
    seen_cancel: u32,
    /// 最後に指が居た所(窓の点)。拡縮を**指の下**で行うために覚える。
    cursor: Option<(f64, f64)>,
    /// **出す物だけを映す。** 書き出しカメラで撮り、取っ手も枠も描かず、触れない。
    /// Stage が世界を見る場になった以上、出力を確かめる場が別に要る。
    output_only: bool,
}

enum State {
    Suspended,
    Active(Box<Active>),
}

struct Active {
    engine: Engine,
    displayed: Option<TexAndHandle>,
    next: Option<TexAndHandle>,
    /// 画角を広げて撮った物。**枠の外**を見せるためだけに使う。
    /// 目玉は動かさず画角だけ `zoom/k` に広げるので、中心基準で k 倍に
    /// 拡げると**全ての奥行きで**元の絵と重なる。
    wide: Option<TexAndHandle>,
}

struct TexAndHandle {
    texture: wgpu::Texture,
    handle: ResourceId,
}

impl StageWidget {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        clock: Arc<Clock>,
        doc: Arc<Mutex<Document>>,
        selection: Selection,
        selected_mirror: Signal<Option<LayerId>>,
        revision: Signal<u32>,
        selected_size: Arc<Mutex<Option<[f32; 2]>>>,
        view_camera: Arc<Mutex<crate::render::engine::ObservationCamera>>,
        rings: Arc<std::sync::atomic::AtomicBool>,
        frame_dim: Arc<std::sync::atomic::AtomicU32>,
        cancel: Arc<std::sync::atomic::AtomicU32>,
        gesture_active: Arc<std::sync::atomic::AtomicBool>,
        output_only: bool,
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
            camera_drag: None,
            revision,
            selected_size,
            view_camera,
            rings,
            frame_dim,
            cancel,
            gesture_active,
            seen_cancel: 0,
            cursor: None,
            output_only,
        }
    }

    fn current_rt(&self) -> RationalTime {
        let t_sec = self.clock.now_sec();
        RationalTime::try_new((t_sec * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO)
    }

    /// 書き出しカメラの中心。
    fn export_center(&self, rt: RationalTime) -> (f64, f64) {
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        match PropertyId::camera(property::CAMERA_CENTER)
            .ok()
            .and_then(|p| view.camera_value_at(&p, rt).ok().flatten())
        {
            Some(Value::Vec2([x, y])) => (x, y),
            _ => (0.0, 0.0),
        }
    }

    /// 掴んでいる間は履歴を汚さず、離した時に1つだけ残す(層のドラッグと同じ流儀)。
    /// 掴みを終える。**離した事が届かなかった時もここを通す** —— 捨てると
    /// 離した所までの編集が失われる(規格の pointer capture が保証している物の、
    /// 届く範囲での代わり)。
    fn finish_drag(&mut self, shift: bool, alt: bool) {
        self.gesture_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
                if let Some(cam) = self.camera_drag.take() {
                    if let (true, Some(at)) = (cam.export_frame, cam.last) {
                        let next = camera_center_for(&cam, at);
                        self.write_export_center(next, self.current_rt(), true);
                        self.revision += 1;
                    }
                    return;
                }
                let Some(drag) = self.drag.take() else { return };
                let Some((cx, cy)) = drag.last else {
                    println!("PROBE room=write verdict=gizmo-noop reason=never-moved");
                    return;
                };
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
                    GizmoMode::Orbit { axis_x } => {
                        let (rx, ry) = orbit_axis(drag.orig_rotation_xy, drag.grab, (cx, cy), axis_x, self.fit.s);
                        intents.extend(track_intent(&doc, drag.layer, property::ROTATION_X, Value::F64(rx), rt));
                        intents.extend(track_intent(&doc, drag.layer, property::ROTATION_Y, Value::F64(ry), rt));
                        &[property::ROTATION_X, property::ROTATION_Y]
                    }
                    GizmoMode::Depth => {
                        let z = drag.orig_z + (cy - drag.grab.1);
                        intents.extend(track_intent(&doc, drag.layer, property::POSITION_Z, Value::F64(z), rt));
                        &[property::POSITION_Z]
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

    /// 掴んだ物を**書かずに**手放す。窓が背面へ回った時や `Esc` で通る。
    fn cancel_drag(&mut self) {
        if let Some(drag) = self.drag.take() {
            let mut doc = self.doc.lock().unwrap();
            for name in [
                property::POSITION,
                property::SCALE,
                property::ROTATION,
                property::ROTATION_X,
                property::ROTATION_Y,
                property::POSITION_Z,
            ] {
                if let Ok(prop) = PropertyId::new(name) {
                    doc.clear_transient(drag.layer, &prop);
                }
            }
        }
        self.camera_drag = None;
        self.gesture_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.revision += 1;
    }


    fn write_export_center(&self, center: (f64, f64), rt: RationalTime, commit: bool) {
        let Ok(property) = PropertyId::camera(property::CAMERA_CENTER) else { return };
        let value = Value::Vec2([center.0, center.1]);
        let mut doc = self.doc.lock().unwrap();
        if !commit {
            doc.set_camera_transient(property, value);
            return;
        }
        doc.clear_camera_transient(&property);
        let mut track = doc.view().camera_track(&property).ok().flatten().unwrap_or_default();
        track.insert(crate::doc::store::Keyframe {
            t: rt,
            value,
            interp: crate::doc::store::Interp::Linear,
            spatial: None,
        });
        if let Err(e) = doc.apply(Intent::SetCameraTrack { property, track }) {
            println!("PROBE room=write verdict=apply-error {e}");
        }
    }

    /// 書き出しの枠の縁を掴んでいるか(世界の座標で見る)。
    fn near_export_frame(&self, wx: f64, wy: f64) -> bool {
        let State::Active(active) = &self.state else { return false };
        let Some(target) = active.displayed.as_ref().or(active.next.as_ref()) else {
            return false;
        };
        let comp = crate::doc::core::CompSpec {
            width: target.texture.width(),
            height: target.texture.height(),
        };
        let rt = self.current_rt();
        let camera = {
            let doc = self.doc.lock().unwrap();
            doc.view().resolve_camera(rt).unwrap_or_default()
        };
        let image = crate::doc::core::camera_screen_from_world_z0(comp, camera)
            .transform_point2(glam::vec2(wx as f32, wy as f32));
        let (w, h) = (comp.width as f32, comp.height as f32);
        let tol = (8.0 / self.fit.s.max(1e-6)) as f32;
        let inside = image.x >= -tol && image.x <= w + tol && image.y >= -tol && image.y <= h + tol;
        let core = image.x > tol && image.x < w - tol && image.y > tol && image.y < h - tol;
        inside && !core
    }

    fn layer_f64(&self, layer: LayerId, name: &str) -> f64 {
        let rt = self.current_rt();
        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        match PropertyId::new(name)
            .ok()
            .and_then(|p| view.value_at(layer, &p, rt).ok().flatten())
        {
            Some(Value::F64(v)) => v,
            _ => 0.0,
        }
    }

    fn rotation_xy(&self, layer: LayerId) -> (f64, f64) {
        (
            self.layer_f64(layer, property::ROTATION_X),
            self.layer_f64(layer, property::ROTATION_Y),
        )
    }

    fn depth(&self, layer: LayerId) -> f64 {
        self.layer_f64(layer, property::POSITION_Z)
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

struct SelGeom {
    z: f64,
    rotation_x: f64,
    rotation_y: f64,
    position: (f64, f64),
    anchor: (f64, f64),
    rotation: f64,
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
    let z = f64_at(view, layer, property::POSITION_Z, rt, 0.0);
    let rotation_x = f64_at(view, layer, property::ROTATION_X, rt, 0.0);
    let rotation_y = f64_at(view, layer, property::ROTATION_Y, rt, 0.0);
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
    Some(SelGeom { z, rotation_x, rotation_y, position, anchor, rotation, natural, box_ })
}

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
        GizmoMode::Move | GizmoMode::Rotate | GizmoMode::Orbit { .. } | GizmoMode::Depth => {}
    }
    let (nbx, nby) = (nx0.min(nx1), ny0.min(ny1));
    let (nbw, nbh) = ((nx1 - nx0).abs().max(0.01), (ny1 - ny0).abs().max(0.01));
    let scale = (nbw / natural.0.max(0.01), nbh / natural.1.max(0.01));
    let position = (nbx + scale.0 * anchor.0, nby + scale.1 * anchor.1);
    (scale, position)
}

/// 画面の1pxを0.5°に読む。掴んだ板が指の動きに素直に付いてくる速さ。
/// 向きの輪の大きさ。枠の内側に収め、潰した側が軸の向きを示す。
const RING_FLAT: f64 = 0.28;

/// 奥行きの取っ手。**真ん中は動かすための場所**なので、そこには置かない。
/// 中心から画面上で一定の長さだけ左上へ伸ばした先に置く(AE や Blender の
/// 軸の矢印と同じ考え)。層が小さくても真ん中と食い合わない。
fn depth_handle(mx: f64, my: f64, scale: f64) -> (f64, f64) {
    let d = 24.0 / scale.max(1e-6);
    (mx - d, my - d)
}

/// 向きの輪は**箱の外**に置く。中は動かすための場所なので明け渡す。
/// 枠のすぐ外(24px)は画面に対して回す帯なので、輪はその先へ置く。
/// 近すぎると、回す帯が輪に食われて掴めなくなる。
fn ring_radii(bw: f64, bh: f64, scale: f64) -> (f64, f64) {
    let m = 56.0 / scale.max(1e-6);
    (bw.abs() * 0.5 + m, bh.abs() * 0.5 + m)
}

/// 画面で 1px 引いたら何度回るか。**世界の単位で数えると、拡大率と作品の大きさで
/// 同じ手つきの結果が変わる。** 指の動いた長さで決める。
const ORBIT_DEGREES_PER_PIXEL: f64 = 0.5;

fn orbit_angles(orig: (f64, f64), grab: (f64, f64), now: (f64, f64), scale: f64) -> (f64, f64) {
    let k = ORBIT_DEGREES_PER_PIXEL * scale;
    (
        orig.0 - (now.1 - grab.1) * k,
        orig.1 + (now.0 - grab.0) * k,
    )
}

/// 掴んだ輪だけが回る。もう一方の軸は掴まれていないので据え置く。
fn orbit_axis(orig: (f64, f64), grab: (f64, f64), now: (f64, f64), axis_x: bool, scale: f64) -> (f64, f64) {
    let (rx, ry) = orbit_angles(orig, grab, now, scale);
    if axis_x {
        (rx, orig.1)
    } else {
        (orig.0, ry)
    }
}

/// タップがドラッグになるまでに指が動ける長さ(窓の点)。
/// 外の規格が必須として挙げている物(Android の touch slop)。
const DRAG_SLOP_PIXELS: f64 = 3.0;

/// 掴んだ物が指について来るように、カメラの中心を出す。
///
/// **視点は世界を掴んでいる**ので、世界を右へ引くならカメラは左へ動く。
/// **書き出しの枠は枠そのものを掴んでいる**ので、枠を右へ引けば枠が右へ動く。
/// 同じ式を使うと、掴んだ物が指と逆へ逃げる。
fn camera_center_for(cam: &CameraDrag, at: (f64, f64)) -> (f64, f64) {
    let (dx, dy) = (at.0 - cam.grab.0, at.1 - cam.grab.1);
    let sign = if cam.export_frame { 1.0 } else { -1.0 };
    (cam.orig_center.0 + sign * dx, cam.orig_center.1 + sign * dy)
}

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

fn track_intent(doc: &Document, layer: LayerId, name: &str, value: Value, t: RationalTime) -> Option<Intent> {
    let prop = PropertyId::new(name).ok()?;
    // キーが無い間は**値を置くだけ**。◇ を押すまで時間の世界へ入れない。
    let Some(mut track) = doc.view().track(layer, &prop).ok().flatten() else {
        return Some(Intent::SetConstant { layer, property: prop, value });
    };
    track.insert(Keyframe { t, value, interp: Interp::Linear, spatial: None });
    Some(Intent::SetTrack { layer, property: prop, track })
}

fn create_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe-stage-target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
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
                self.state = State::Active(Box::new(Active { engine, displayed: None, next: None, wide: None }));
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
        if self.output_only {
            // 出力を映す窓。ここは**見るだけ**で、触っても何も起きない。
            return;
        }
        let cancel = self.cancel.load(std::sync::atomic::Ordering::Relaxed);
        if cancel != self.seen_cancel {
            self.seen_cancel = cancel;
            self.cancel_drag();
            return;
        }
        match event {
            UiEvent::Wheel(wheel) => {
                let dy = match wheel.delta {
                    blitz_traits::events::BlitzWheelDelta::Pixels(_, y) => y,
                    blitz_traits::events::BlitzWheelDelta::Lines(_, y) => y * 20.0,
                };
                if dy == 0.0 {
                    return;
                }
                // **指の下を動かさない。** 中心を基準に拡げると、見たい物が
                // 画面の外へ逃げ、拡げるたびに動かし直す手間が付く。
                let anchor = self
                    .cursor
                    .map(|(x, y)| ((x, y), self.fit.image_at(x, y)));
                let mut view = self.view_camera.lock().unwrap();
                view.zoom = (view.zoom * (1.0 - dy as f32 * 0.002)).clamp(0.05, 40.0);
                if let Some((screen, img)) = anchor {
                    view.pan = self.fit.pan_putting(img, screen, view.zoom as f64);
                }
            }
            UiEvent::PointerDown(p) => {
                let (cx, cy) = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                if let Some(layer) = self.selection.get() {
                    if let Some(geom) = self.selection_geom(layer) {
                        // 傾いた層は台形に見える。**見えている所で掴めるように**、
                        // 層の面と窓の間の写像を通して箱の中の位置へ戻す。
                        let map = plane_map(&self.fit, &geom);
                        let (bxx, byy, bww, bhh) = geom.box_;
                        let (u, v) = map.to_uv(p.element.x as f64, p.element.y as f64);
                        if !u.is_finite() || !v.is_finite() {
                            return;
                        }
                        let (lx, ly) = (bxx + u * bww, byy + v * bhh);
                        let (cx, cy) = rotate_around(geom.position, geom.rotation, (lx, ly));
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
                        let rings = self.rings.load(std::sync::atomic::Ordering::Relaxed);
                        let (mx, my) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
                        let (dx, dy) = depth_handle(mx, my, self.fit.s);
                        if rings && mode.is_none() && near(dx, dy) {
                            mode = Some(GizmoMode::Depth);
                        }
                        let inside = lx >= bx && lx <= bx + bw && ly >= by && ly <= by + bh;
                        // **箱の中は必ず動かす。** 輪が中を横切っても、そこは掴めない。
                        if rings && mode.is_none() && !inside {
                            let (ra, rb) = ring_radii(bw, bh, self.fit.s);
                            // 輪までの距離は**中心から見た半径の差**で測る。
                            // `(f-1)*min(a,b)` で近似すると、平たい楕円の短径側で
                            // 距離を大きく見誤り、輪から遠い所まで輪として掴んでしまう。
                            let on = |a: f64, b: f64| {
                                if a <= 1e-6 || b <= 1e-6 {
                                    return false;
                                }
                                let (px, py) = (lx - mx, ly - my);
                                let f = ((px / a).powi(2) + (py / b).powi(2)).sqrt();
                                if f < 1e-9 {
                                    return false;
                                }
                                let reach = (px * px + py * py).sqrt();
                                (reach - reach / f).abs() <= tol
                            };
                            if on(ra, rb * RING_FLAT) {
                                mode = Some(GizmoMode::Orbit { axis_x: false });
                            } else if on(ra * RING_FLAT, rb) {
                                mode = Some(GizmoMode::Orbit { axis_x: true });
                            }
                        }
                        if mode.is_none() {
                            if inside {
                                mode = Some(GizmoMode::Move);
                            } else {
                                let margin = 24.0 / self.fit.s.max(1e-6);
                                if lx >= bx - margin && lx <= bx + bw + margin && ly >= by - margin && ly <= by + bh + margin {
                                    mode = Some(GizmoMode::Rotate);
                                }
                            }
                        }
                        println!(
                            "PROBE room=input verdict=gizmo-hit mode={mode:?} at=({lx:.0},{ly:.0})                              box=({bx:.0},{by:.0},{bw:.0},{bh:.0}) depth=({dx:.0},{dy:.0}) tol={tol:.0}"
                        );
                        if let Some(mode) = mode {
                            self.gesture_active
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            self.drag = Some(GizmoDrag {
                                layer,
                                mode,
                                grab: (cx, cy),
                                orig_position: geom.position,
                                orig_rotation: geom.rotation,
                                orig_rotation_xy: self.rotation_xy(layer),
                                orig_z: self.depth(layer),
                                anchor: geom.anchor,
                                natural: geom.natural,
                                orig_box: geom.box_,
                                fit_z: geom.z,
                                last: None,
                            });
                            return;
                        }
                    }
                }
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
                match hit {
                    Some((_, layer)) => {
                        if p.mods.contains(Modifiers::META) {
                            self.selection.toggle(layer);
                        } else {
                            self.selection.set(Some(layer));
                        }
                        self.selected_mirror.set(self.selection.get());
                    }
                    None => {
                        // 枠の縁を掴んだら書き出しカメラ、それ以外は視点。
                        // 錠が掛かっている間は枠を掴めない(誤って掴むのを止める)。
                        self.gesture_active
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        if self.near_export_frame(cx, cy) && !crate::ui::fixture::camera_locked() {
                            let rt = self.current_rt();
                            self.camera_drag = Some(CameraDrag {
                                grab: (cx, cy),
                                orig_center: self.export_center(rt),
                                export_frame: true,
                                last: None,
                                grab_image: self.fit.image_at(p.element.x as f64, p.element.y as f64),
                            });
                        } else {
                            let pan = self.view_camera.lock().unwrap().pan;
                            self.camera_drag = Some(CameraDrag {
                                grab: (cx, cy),
                                orig_center: (pan[0] as f64, pan[1] as f64),
                                export_frame: false,
                                last: None,
                                grab_image: self.fit.image_at(p.element.x as f64, p.element.y as f64),
                            });
                        }
                    }
                }
            }
            UiEvent::PointerMove(p) => {
                self.cursor = Some((p.element.x as f64, p.element.y as f64));
                // 枠の外で離すと、離した事がここへ届かない。掴んだままの絵が残る。
                if p.buttons.is_empty() && (self.drag.is_some() || self.camera_drag.is_some()) {
                    println!("PROBE room=input verdict=drag-finished reason=release-not-seen");
                    let shift = p.mods.contains(Modifiers::SHIFT);
                    let alt = p.mods.contains(Modifiers::ALT);
                    self.finish_drag(shift, alt);
                    return;
                }
                if self.camera_drag.is_some() {
                    let at = self.fit.to_comp(p.element.x as f64, p.element.y as f64);
                    let cam = self.camera_drag.as_mut().expect("直前に居ることを見た");
                    cam.last = Some(at);
                    let cam = &*cam;
                    if cam.export_frame {
                        let next = camera_center_for(cam, at);
                        self.write_export_center(next, self.current_rt(), false);
                        self.revision += 1;
                    } else {
                        // 掴んだ絵の点が指について来るように動かす。
                        let screen = (p.element.x as f64, p.element.y as f64);
                        let img = cam.grab_image;
                        let mut view = self.view_camera.lock().unwrap();
                        let zoom = view.zoom as f64;
                        view.pan = self.fit.pan_putting(img, screen, zoom);
                    }
                    return;
                }
                let (cx, cy) = {
                    let z = self.drag.as_ref().map(|d| d.fit_z);
                    let Some(z) = z else { return };
                    let at = self.fit.at_z(z).to_comp(p.element.x as f64, p.element.y as f64);
                    let slop = DRAG_SLOP_PIXELS / self.fit.s.max(1e-9);
                    let Some(drag) = self.drag.as_mut() else { return };
                    // **押しただけでは動かない。** 指が少し動くまでは掴んだ事にしない
                    // (押すつもりが値を変える、が起きる)。
                    if drag.last.is_none() {
                        let (dx, dy) = (at.0 - drag.grab.0, at.1 - drag.grab.1);
                        if (dx * dx + dy * dy).sqrt() < slop {
                            return;
                        }
                    }
                    drag.last = Some(at);
                    at
                };
                let Some(drag) = self.drag.as_ref() else { return };
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
                    GizmoMode::Orbit { axis_x } => {
                        let (rx, ry) = orbit_axis(drag.orig_rotation_xy, drag.grab, (cx, cy), axis_x, self.fit.s);
                        if let Ok(prop) = PropertyId::new(property::ROTATION_X) {
                            doc.set_transient(drag.layer, prop, Value::F64(rx));
                        }
                        if let Ok(prop) = PropertyId::new(property::ROTATION_Y) {
                            doc.set_transient(drag.layer, prop, Value::F64(ry));
                        }
                    }
                    GizmoMode::Depth => {
                        let Ok(prop) = PropertyId::new(property::POSITION_Z) else { return };
                        doc.set_transient(drag.layer, prop, Value::F64(drag.orig_z + (cy - drag.grab.1)));
                    }
                }
            }
            UiEvent::PointerUp(p) => {
                let shift = p.mods.contains(Modifiers::SHIFT);
                let alt = p.mods.contains(Modifiers::ALT);
                self.finish_drag(shift, alt);
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
        // 面が 0 の時は描かない(timeline_widget と同じ)。ここを通すと下の
        // `s = (w/cw).min(h/ch)` が 0 になり、退化した Affine で vello を回すことになる。
        if width == 0 || height == 0 {
            return scene;
        }
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

        // 見るのは視点カメラ、書き出しは Document のカメラ。同じ世界を通る。
        let observation = if self.output_only {
            crate::render::engine::ObservationCamera::default()
        } else {
            *self.view_camera.lock().unwrap()
        };
        let export_camera = view.resolve_camera(rt).unwrap_or_default();
        // **撮るのは常に書き出しのカメラ。** 視点は撮れた絵を2Dで動かすだけなので、
        // preview と export が食い違いようがない(裁定 2026-09-01)。
        let rendered = active.engine.render_frame_into_with_camera(
            &view,
            rt,
            &target,
            export_camera,
            self.output_only,
        );
        if let Err(e) = rendered {
            println!("PROBE room=stage verdict=render-error {e}");
            return scene;
        }

        // 枠の外を見せるための、画角を広げた1枚。**目玉は動かさず画角だけ**
        // `zoom/k` に広げる。距離を変えると遠近そのものが変わってしまう。
        // 窓が枠より広い時だけ撮る(枠が窓を覆っていれば外は見えない)。
        let wide = if self.output_only {
            None
        } else {
            let base = ((width as f64) / cw as f64).min((height as f64) / ch as f64);
            let on_screen = base * observation.zoom as f64;
            let need = ((width as f64) / (cw as f64 * on_screen))
                .max((height as f64) / (ch as f64 * on_screen));
            let k = need.clamp(1.0, 4.0);
            if k <= 1.02 {
                None
            } else {
                if active.wide.is_none() {
                    let texture = create_target(active.engine.gpu_device(), cw, ch);
                    let handle = render_ctx
                        .try_register_custom_resource(Box::new(texture.clone()))
                        .expect("wgpu backend accepts wgpu textures");
                    active.wide = Some(TexAndHandle { texture, handle });
                }
                let wide = active.wide.as_ref().expect("直前に用意した");
                let camera = crate::doc::core::ResolvedCamera {
                    zoom: export_camera.zoom / k as f32,
                    ..export_camera
                };
                match active.engine.render_frame_into_with_camera(
                    &view,
                    rt,
                    &wide.texture,
                    camera,
                    false,
                ) {
                    Ok(()) => Some((wide.handle, k)),
                    Err(e) => {
                        println!("PROBE room=stage verdict=wide-render-error {e}");
                        None
                    }
                }
            }
        };

        let primary_layer = self.selection.get();
        let primary_geom =
            primary_layer.and_then(|layer| selection_geom_in(&active.engine, &view, layer, rt));
        *self.selected_size.lock().unwrap() =
            primary_geom.as_ref().map(|g| [g.natural.0 as f32, g.natural.1 as f32]);
        let selected_box = primary_geom;
        let secondary_boxes: Vec<_> = self
            .selection
            .all()
            .into_iter()
            .filter(|l| Some(*l) != primary_layer)
            .filter_map(|l| selection_geom_in(&active.engine, &view, l, rt))
            .collect();

        drop(view);
        drop(doc);
        if first {
            println!("PROBE room=stage verdict=first-submit-ok");
        }

        std::mem::swap(&mut active.next, &mut active.displayed);

        let (w, h) = (width as f64, height as f64);
        let (cw, ch) = (target.width() as f64, target.height() as f64);
        // 窓に収める倍率。ここへ**視点の拡大**を掛け、**視点の移動**を足す。
        // 視点は撮れた絵に対する2Dの動きで、カメラには触らない。
        let base = (w / cw).min(h / ch);
        let s = base * observation.zoom as f64;
        let (fw, fh) = (cw * s, ch * s);
        let (fx, fy) = (
            (w - fw) * 0.5 - observation.pan[0] as f64 * s,
            (h - fh) * 0.5 - observation.pan[1] as f64 * s,
        );

        let k = if scale > 0.0 { scale } else { 1.0 };
        let comp = crate::doc::core::CompSpec {
            width: target.width(),
            height: target.height(),
        };
        self.fit = Fit {
            s: s / k,
            fx: fx / k,
            fy: fy / k,
            image_from_world: crate::doc::core::camera_screen_from_world_z0(comp, export_camera),
            comp,
            camera: export_camera,
            widget: (w / k, h / k),
            image: (cw, ch),
            base: base / k,
        };

        // 描く時は物理 px、掴む時は論理 px。写像を2つ持つ。
        let draw = Fit { s, fx, fy, ..self.fit };

        // 書き出しの枠。地色は枠の中だけに敷く(視界全体を塗ると枠の意味が消える)。
        let frame = {
            let comp = crate::doc::core::CompSpec {
                width: target.width(),
                height: target.height(),
            };
            let image_from_world =
                crate::doc::core::camera_screen_from_world_z0(comp, export_camera);
            let world_from_image = image_from_world.inverse();
            let corner = |ix: f32, iy: f32| {
                let w = world_from_image.transform_point2(glam::vec2(ix, iy));
                let (sx, sy) = draw.to_screen(w.x as f64, w.y as f64);
                peniko::kurbo::Point::new(sx, sy)
            };
            let (iw, ih) = (comp.width as f32, comp.height as f32);
            let mut frame = peniko::kurbo::BezPath::new();
            frame.move_to(corner(0.0, 0.0));
            frame.line_to(corner(iw, 0.0));
            frame.line_to(corner(iw, ih));
            frame.line_to(corner(0.0, ih));
            frame.close_path();
            frame
        };

        let bg = composition.background;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Solid(Color::new([bg[0], bg[1], bg[2], bg[3]])),
            None,
            &frame,
        );
        // 枠の外。**中身は本物のまま、外側を暗い膜で落とす。**
        // 形を塗りつぶすと動画に効かないので、広く撮った絵をそのまま敷く。
        if let Some((wide_handle, k)) = wide {
            let (wx, wy) = (fx + fw * 0.5, fy + fh * 0.5);
            let (ww, wh) = (fw * k, fh * k);
            let rect = Rect::from_origin_size((wx - ww * 0.5, wy - wh * 0.5), (ww, wh));
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                PaintRef::Resource(ImageBrush {
                    image: wide_handle,
                    sampler: ImageSampler::default(),
                }),
                Some(
                    Affine::translate((wx - ww * 0.5, wy - wh * 0.5))
                        * Affine::scale(s * k),
                ),
                &rect,
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                PaintRef::Solid(Color::from_rgba8(
                    0x1a,
                    0x1a,
                    0x1a,
                    (self.frame_dim.load(std::sync::atomic::Ordering::Relaxed).min(100) * 255 / 100)
                        as u8,
                )),
                None,
                &rect,
            );
        }

        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush { image: handle, sampler: ImageSampler::default() }),
            Some(Affine::translate((fx, fy)) * Affine::scale(s)),
            &Rect::from_origin_size((fx, fy), (fw, fh)),
        );
        if self.output_only {
            // 出す物だけ。枠も取っ手も描かない。
            return scene;
        }
        scene.stroke(
            &peniko::kurbo::Stroke::new(1.0).with_dashes(0.0, [4.0, 4.0]),
            Affine::IDENTITY,
            PaintRef::Solid(c(tokens::INK3)),
            None,
            &frame,
        );

        // 副次の選択も**同じ写像**で描く。長方形で描くと、回した層や奥に在る層で
        // 枠だけが別の場所に残る。

        for geom in &secondary_boxes {
            let (_, _, bw, bh) = geom.box_;
            if bw.abs() < 1e-9 || bh.abs() < 1e-9 {
                continue;
            }
            let map = plane_map(&draw, geom);
            let p = |u: f64, v: f64| {
                let (x, y) = map.to_screen(u, v);
                peniko::kurbo::Point::new(x, y)
            };
            let mut outline = peniko::kurbo::BezPath::new();
            outline.move_to(p(0.0, 0.0));
            outline.line_to(p(1.0, 0.0));
            outline.line_to(p(1.0, 1.0));
            outline.line_to(p(0.0, 1.0));
            outline.close_path();
            scene.stroke(
                &peniko::kurbo::Stroke::new(1.0),
                Affine::IDENTITY,
                PaintRef::Solid(c(tokens::ACCENT)),
                None,
                &outline,
            );
        }

        if let Some(geom) = &selected_box {
            let (bx, by, bw, bh) = geom.box_;
            if bw.abs() > 1e-9 && bh.abs() > 1e-9 {
                // 局所(箱の中)で組んでから、層の面の写像で窓へ落とす。
                // 掴む側と同じ座標系なので、見えている所がそのまま掴める所になる。
                let map = plane_map(&draw, geom);
                let l2s = |lx: f64, ly: f64| {
                    let (x, y) = map.to_screen((lx - bx) / bw, (ly - by) / bh);
                    peniko::kurbo::Point::new(x, y)
                };
                let stroke = peniko::kurbo::Stroke::new(1.5);
                let thin = peniko::kurbo::Stroke::new(1.0);

                // アンカー(回転と拡大の支点)
                let a = l2s(geom.position.0, geom.position.1);
                let arm = 6.0;
                for line in [
                    peniko::kurbo::Line::new((a.x - arm, a.y), (a.x + arm, a.y)),
                    peniko::kurbo::Line::new((a.x, a.y - arm), (a.x, a.y + arm)),
                ] {
                    scene.stroke(&thin, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, &line);
                }

                // 枠
                let mut outline = peniko::kurbo::BezPath::new();
                outline.move_to(l2s(bx, by));
                outline.line_to(l2s(bx + bw, by));
                outline.line_to(l2s(bx + bw, by + bh));
                outline.line_to(l2s(bx, by + bh));
                outline.close_path();
                scene.stroke(&stroke, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, &outline);

                // 四隅と辺の取っ手
                let hs = 6.0;
                let (mx, my) = (bx + bw * 0.5, by + bh * 0.5);
                for (lx, ly) in [
                    (bx, by), (bx + bw, by), (bx, by + bh), (bx + bw, by + bh),
                    (mx, by), (mx, by + bh), (bx, my), (bx + bw, my),
                ] {
                    let p = l2s(lx, ly);
                    let r = Rect::from_origin_size((p.x - hs * 0.5, p.y - hs * 0.5), (hs, hs));
                    scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, &r);
                }

                if self.rings.load(std::sync::atomic::Ordering::Relaxed) {
                    let (ra, rb) = ring_radii(bw, bh, self.fit.s);
                    for (a, b) in [(ra, rb * RING_FLAT), (ra * RING_FLAT, rb)] {
                        let mut ring = peniko::kurbo::BezPath::new();
                        for i in 0..=64 {
                            let t = i as f64 / 64.0 * std::f64::consts::TAU;
                            let p = l2s(mx + a * t.cos(), my + b * t.sin());
                            if i == 0 { ring.move_to(p) } else { ring.line_to(p) }
                        }
                        scene.stroke(&thin, Affine::IDENTITY, PaintRef::Solid(c(tokens::INK3)), None, &ring);
                    }
                    let (hx, hy) = depth_handle(mx, my, self.fit.s);
                    let end = l2s(hx, hy);
                    let stem = peniko::kurbo::Line::new(l2s(mx, my), end);
                    scene.stroke(&thin, Affine::IDENTITY, PaintRef::Solid(c(tokens::INK3)), None, &stem);
                    let dot = peniko::kurbo::Circle::new(end, 3.5);
                    scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(c(tokens::ACCENT)), None, &dot);
                }
            }
        }

        scene
    }
}

#[cfg(test)]
mod orbit_tests {
    use super::orbit_angles;

    #[test]
    fn dragging_right_turns_the_plate_to_face_right_and_down_tips_it_back() {
        let (rx, ry) = orbit_angles((0.0, 0.0), (100.0, 100.0), (120.0, 140.0), 1.0);
        assert!(ry > 0.0, "右へ引いたのに rotation.y が正にならない");
        assert!(rx < 0.0, "下へ引いたのに rotation.x が負にならない");
    }

    #[test]
    fn no_movement_keeps_the_original_angles() {
        assert_eq!(orbit_angles((12.0, -5.0), (7.0, 7.0), (7.0, 7.0), 1.0), (12.0, -5.0));
    }
}

// ---- 傾いた層の面と、窓の間の写像 --------------------------------------
//
// 向きを付けた層は台形に見える。層の面は平面なので、4隅を投影すれば
// 平面と窓の間は**射影写像**1つで書ける。描くのも掴むのもこれを通せば、
// 見えている所と掴める所が必ず一致する。

/// 単位正方形 (0,0)(1,0)(1,1)(0,1) を、与えた4点へ送る写像。
fn homography_from_unit_square(p: [glam::DVec2; 4]) -> glam::DMat3 {
    let (x0, y0) = (p[0].x, p[0].y);
    let (x1, y1) = (p[1].x, p[1].y);
    let (x2, y2) = (p[2].x, p[2].y);
    let (x3, y3) = (p[3].x, p[3].y);
    let sx = x0 - x1 + x2 - x3;
    let sy = y0 - y1 + y2 - y3;

    let (g, h) = if sx.abs() < 1e-9 && sy.abs() < 1e-9 {
        (0.0, 0.0)
    } else {
        let (dx1, dx2) = (x1 - x2, x3 - x2);
        let (dy1, dy2) = (y1 - y2, y3 - y2);
        let den = dx1 * dy2 - dx2 * dy1;
        if den.abs() < 1e-12 {
            (0.0, 0.0)
        } else {
            ((sx * dy2 - dx2 * sy) / den, (dx1 * sy - sx * dy1) / den)
        }
    };

    glam::DMat3::from_cols(
        glam::dvec3(x1 - x0 + g * x1, y1 - y0 + g * y1, g),
        glam::dvec3(x3 - x0 + h * x3, y3 - y0 + h * y3, h),
        glam::dvec3(x0, y0, 1.0),
    )
}

fn apply_h(m: &glam::DMat3, x: f64, y: f64) -> (f64, f64) {
    let q = *m * glam::dvec3(x, y, 1.0);
    if q.z.abs() < 1e-12 {
        return (f64::NAN, f64::NAN);
    }
    (q.x / q.z, q.y / q.z)
}

/// 層の面と窓の間。`uv` は箱の中の割合(0..1)。
struct PlaneMap {
    screen_from_uv: glam::DMat3,
    uv_from_screen: glam::DMat3,
}

impl PlaneMap {
    fn to_screen(&self, u: f64, v: f64) -> (f64, f64) {
        apply_h(&self.screen_from_uv, u, v)
    }

    fn to_uv(&self, sx: f64, sy: f64) -> (f64, f64) {
        apply_h(&self.uv_from_screen, sx, sy)
    }
}

/// 層の4隅を窓の点へ落とし、写像を組む。上流が絵を置くのと同じ式で隅を出す
/// (`tilted_corners`)ので、取っ手は必ず絵の上に乗る。
fn plane_map(fit: &Fit, geom: &SelGeom) -> PlaneMap {
    let (bx, by, bw, bh) = geom.box_;
    let pivot = glam::vec2(geom.position.0 as f32, geom.position.1 as f32);
    let transform = glam::Affine2::from_translation(pivot)
        * glam::Affine2::from_angle((geom.rotation as f32).to_radians())
        * glam::Affine2::from_translation(-pivot);

    let (corner, u, v) = crate::render::compositor::tilted_corners(
        transform,
        glam::vec2(bx as f32, by as f32),
        glam::vec2(bw as f32, bh as f32),
        geom.z as f32,
        geom.rotation_x as f32,
        geom.rotation_y as f32,
    );

    let projection = crate::doc::core::camera_projection(fit.comp, fit.camera);
    let clip_from_world = projection.projection_matrix() * projection.view_matrix();
    let to_screen = |p: glam::Vec3| -> glam::DVec2 {
        let clip = clip_from_world * glam::Vec4::new(p.x, p.y, p.z, 1.0);
        let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
        let ndc = glam::vec2(clip.x / w, clip.y / w);
        let image = glam::vec2(
            (ndc.x + 1.0) * 0.5 * fit.comp.width as f32,
            (1.0 - ndc.y) * 0.5 * fit.comp.height as f32,
        );
        glam::dvec2(
            fit.fx + image.x as f64 * fit.s,
            fit.fy + image.y as f64 * fit.s,
        )
    };

    let screen_from_uv = homography_from_unit_square([
        to_screen(corner),
        to_screen(corner + u),
        to_screen(corner + u + v),
        to_screen(corner + v),
    ]);
    PlaneMap {
        uv_from_screen: screen_from_uv.inverse(),
        screen_from_uv,
    }
}

#[cfg(test)]
mod plane_tests {
    use super::*;

    #[test]
    fn the_map_sends_the_corners_where_they_were_put_and_comes_back() {
        let p = [
            glam::dvec2(10.0, 10.0),
            glam::dvec2(110.0, 20.0),
            glam::dvec2(90.0, 80.0),
            glam::dvec2(20.0, 70.0),
        ];
        let m = homography_from_unit_square(p);
        let inv = m.inverse();
        for (i, (u, v)) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].iter().enumerate() {
            let (sx, sy) = apply_h(&m, *u, *v);
            assert!((sx - p[i].x).abs() < 1e-6 && (sy - p[i].y).abs() < 1e-6, "隅 {i} がずれる");
            let (bu, bv) = apply_h(&inv, sx, sy);
            assert!((bu - u).abs() < 1e-6 && (bv - v).abs() < 1e-6, "隅 {i} から戻れない");
        }
    }

    /// 隅だけ合っていても、**内側がずれていれば掴めない**。描く側と掴む側は
    /// この1枚を共有しているので、任意の傾きの任意の点で往復できなければ、
    /// 絵と当たり判定が別の場所に居る。
    ///
    /// 生成する四角形は角度を回り順にして凸に保つ。潰れた四角形では写像が
    /// 定義できないので、面積に下限を置く。
    proptest::proptest! {
        #[test]
        fn any_point_on_any_tilted_plane_comes_back_where_it_started(
            cx in -500.0f64..500.0,
            cy in -500.0f64..500.0,
            radii in proptest::array::uniform4(30.0f64..400.0),
            wobble in proptest::array::uniform4(-0.6f64..0.6),
            probes in proptest::collection::vec((0.0f64..1.0, 0.0f64..1.0), 1..12),
        ) {
            let corner = |i: usize| {
                let a = std::f64::consts::FRAC_PI_4 + i as f64 * std::f64::consts::FRAC_PI_2 + wobble[i];
                glam::dvec2(cx + radii[i] * a.cos(), cy + radii[i] * a.sin())
            };
            let p = [corner(0), corner(1), corner(2), corner(3)];

            let area: f64 = (0..4)
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % 4]);
                    a.x * b.y - b.x * a.y
                })
                .sum::<f64>()
                .abs()
                * 0.5;
            proptest::prop_assume!(area > 2_000.0);

            let m = homography_from_unit_square(p);
            let inv = m.inverse();
            for (u, v) in probes {
                let (sx, sy) = apply_h(&m, u, v);
                proptest::prop_assume!(sx.is_finite() && sy.is_finite());
                let (bu, bv) = apply_h(&inv, sx, sy);
                proptest::prop_assert!(
                    (bu - u).abs() < 1e-6 && (bv - v).abs() < 1e-6,
                    "({u:.3},{v:.3}) が ({bu:.3},{bv:.3}) で戻ってきた。隅={p:?}"
                );
            }
        }
    }
}
