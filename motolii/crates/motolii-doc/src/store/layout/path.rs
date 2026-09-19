//! 道の上に置く — Offset Path を辿って、その位置と接線の向きを返す。
//! 角丸の矩形の閉じた道もここ(箱を道に変えるのはこの一口だけ)。

use super::*;

/// 角丸の矩形の閉じた道(時計回り、角は 3 次 Bézier の円弧近似)。`to` で各点を写す(接線は向きだけ写す)。
pub(crate) fn rounded_rect_path(b: [f32; 4], radius: f32, to: glam::Affine2) -> crate::doc::eval::Path {
    use crate::doc::eval::{Path, PathVertex};
    let r = radius.min((b[2] - b[0]) * 0.5).min((b[3] - b[1]) * 0.5).max(0.0);
    let k = r * 0.552_284_8;
    let mut vertices = Vec::new();
    let mut push = |p: [f32; 2], inn: [f32; 2], out: [f32; 2]| {
        let point = to.transform_point2(glam::Vec2::from(p));
        let (i, o) = (to.transform_vector2(glam::Vec2::from(inn)), to.transform_vector2(glam::Vec2::from(out)));
        vertices.push(PathVertex { point: [point.x as f64, point.y as f64], in_tangent: [i.x as f64, i.y as f64], out_tangent: [o.x as f64, o.y as f64] });
    };
    let (l, top, rt, bot) = (b[0], b[1], b[2], b[3]);
    push([l + r, top], [-k, 0.0], [0.0, 0.0]);
    push([rt - r, top], [0.0, 0.0], [k, 0.0]);
    push([rt, top + r], [0.0, -k], [0.0, 0.0]);
    push([rt, bot - r], [0.0, 0.0], [0.0, k]);
    push([rt - r, bot], [k, 0.0], [0.0, 0.0]);
    push([l + r, bot], [0.0, 0.0], [-k, 0.0]);
    push([l, bot - r], [0.0, k], [0.0, 0.0]);
    push([l, top + r], [0.0, 0.0], [0.0, -k]);
    Path { vertices, closed: true }
}

impl StoreView<'_> {
    /// Offset Path が Border Box なら、親の箱の輪郭の上の点(親の素材座標)と、その向き(度)。
    pub(crate) fn on_offset_path(&self, layer: LayerId, t: RationalTime) -> Result<Option<([f32; 2], f32)>, StoreError> {
        if self.choice(layer, OFFSET_PATH, t)? != 1 {
            return Ok(None);
        }
        let (b, radius) = match self.attrs(layer)?.unwrap_or_default().parent {
            Some(parent) => {
                if self.display(parent, t)? == 0 {
                    return Ok(None);
                }
                let Some(size) = self.group_size(parent, t)? else { return Ok(None) };
                ([CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]], self.number(parent, BORDER_RADIUS, 0.0, t)?.max(0.0) as f32)
            }
            None => {
                let Some(comp) = self.composition()? else { return Ok(None) };
                ([0.0, 0.0, comp.width as f32, comp.height as f32], 0.0)
            }
        };
        let fraction = (self.number(layer, OFFSET_DISTANCE, 0.0, t)? as f32 / 100.0).rem_euclid(1.0);
        let (w, h) = (b[2] - b[0], b[3] - b[1]);
        let r = radius.min(w * 0.5).min(h * 0.5);
        let (sw, sh) = (w - 2.0 * r, h - 2.0 * r);
        let arc = std::f32::consts::FRAC_PI_2 * r;
        let total = 2.0 * (sw + sh) + 4.0 * arc;
        if total <= 1e-3 {
            return Ok(Some(([b[0], b[1]], 0.0)));
        }
        let mut d = fraction * total;
        // 上の辺 → 右上の角 → 右の辺 → 右下 → 下の辺 → 左下 → 左の辺 → 左上。
        let corners = [[b[2] - r, b[1] + r], [b[2] - r, b[3] - r], [b[0] + r, b[3] - r], [b[0] + r, b[1] + r]];
        let starts = [[b[0] + r, b[1]], [b[2], b[1] + r], [b[2] - r, b[3]], [b[0], b[3] - r]];
        let dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
        let lengths = [sw, sh, sw, sh];
        for side in 0..4 {
            if d <= lengths[side] {
                let p = [starts[side][0] + dirs[side][0] * d, starts[side][1] + dirs[side][1] * d];
                return Ok(Some((p, [0.0f32, 90.0, 180.0, 270.0][side])));
            }
            d -= lengths[side];
            if d <= arc {
                let angle = -std::f32::consts::FRAC_PI_2 + side as f32 * std::f32::consts::FRAC_PI_2 + if r > 0.0 { d / r } else { 0.0 };
                let c = corners[side];
                let p = [c[0] + r * angle.cos(), c[1] + r * angle.sin()];
                return Ok(Some((p, angle.to_degrees() + 90.0)));
            }
            d -= arc;
        }
        Ok(Some(([b[0] + r, b[1]], 0.0)))
    }

    /// 道の向きに回る分(Offset Rotate = Auto)。
    pub(crate) fn offset_rotation(&self, layer: LayerId, t: RationalTime) -> Result<f32, StoreError> {
        if self.choice(layer, OFFSET_ROTATE, t)? != 0 {
            return Ok(0.0);
        }
        Ok(self.on_offset_path(layer, t)?.map_or(0.0, |(_, angle)| angle))
    }
}
