//! **ここは上流(`tiny-skia`)を包んでいるだけ**。中身を知りたければ tiny-skia を読む。
//!
//! crate の marker が `owns:` なのは演算子(`ops.rs`)のためで、
//! **この module は `wraps:` である**。`check.sh` は crate の根しか見ないので、
//! 境界はこの doc コメントが持つ(裁定34 と同じ形)。
//!
//! ## なぜ Vello + usvg ではなく tiny-skia か
//!
//! `docs/concept.md` は Vello + usvg と書いているが、この crate では採らなかった。
//! 理由は3つ:
//!
//! 1. **依存が1本も増えない**。`tiny-skia 0.11.4` は既に `next/Cargo.lock` に居る —
//!    `iced_tiny_skia`(iced 自身の software renderer)が引いている。**front が既に
//!    抱えている物**をそのまま使うので、保守対象は増えない(軸4)
//! 2. **GPU を要求しない**ので試験が速く回る。R1 が「GPU 試験は単独で走らせないと
//!    予算が倍にぶれる」を実測している以上、GPU を1つ増やすのは試験の質を下げる
//! 3. **usvg が要らない**。usvg は SVG の**文字列**を読むための物で、ここの入力は
//!    既に解決済みの頂点列。SVG を読む日が来たら usvg を足せばよく、
//!    それは今日の判断ではない(軸4「一時的にを作らない」)
//!
//! 出る物は `TexturedRect` へ流す RGBA なので、**この選択は合成器から見えない**。
//! GPU 側の理由で差し替える日が来ても、この module の外は動かない。
//!
//! ## premultiplied であること
//!
//! `tiny_skia::Pixmap` は**そのまま premultiplied RGBA8**(上流 doc「Byteorder: RGBA」+
//! `PremultipliedColorU8`)。だから変換を1行も書いていない — 書くと丸めが1つ増える。

use tiny_skia::{
    FillRule as TsFillRule, LineCap as TsCap, LineJoin as TsJoin, Paint, PathBuilder, Pixmap,
    Stroke as TsStroke, StrokeDash, Transform,
};

use crate::geom::{is_straight, Contour, Path, Point};
use crate::{Canvas, Fill, FillRule, LineCap, LineJoin, Raster, Stroke, VectorError};

/// 輪郭列を tiny-skia のパスへ。`origin` はここで足し込む —
/// `Transform` で渡すと stroke 幅まで一緒に変換されうるので、頂点に畳んでおく。
fn to_tiny_skia(path: &Path, origin: Point) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    let mut any = false;
    for c in path {
        if c.vertices.len() < 2 {
            continue;
        }
        any = true;
        emit_contour(&mut b, c, origin);
    }
    if !any {
        return None;
    }
    b.finish()
}

fn emit_contour(b: &mut PathBuilder, c: &Contour, origin: Point) {
    let at = |p: Point| ((p.x + origin.x) as f32, (p.y + origin.y) as f32);
    let n = c.vertices.len();
    let (x0, y0) = at(c.vertices[0].point);
    b.move_to(x0, y0);
    let edges = if c.closed { n } else { n - 1 };
    for i in 0..edges {
        let v0 = &c.vertices[i];
        let v1 = &c.vertices[(i + 1) % n];
        let (x, y) = at(v1.point);
        if is_straight(v0, v1) {
            b.line_to(x, y);
        } else {
            let (cx1, cy1) = at(v0.point.add(v0.out_tangent));
            let (cx2, cy2) = at(v1.point.add(v1.in_tangent));
            b.cubic_to(cx1, cy1, cx2, cy2, x, y);
        }
    }
    if c.closed {
        b.close();
    }
}

fn paint_for(r: f64, g: f64, b: f64, alpha: f64) -> Paint<'static> {
    // `clamp01` が 0..1 の有限値を保証しているので上流の `None` は起きない。
    // それでも unwrap しないのは、panic する経路を1本も置かないため。
    let color = tiny_skia::Color::from_rgba(
        clamp01(r) as f32,
        clamp01(g) as f32,
        clamp01(b) as f32,
        clamp01(alpha) as f32,
    )
    .unwrap_or(tiny_skia::Color::TRANSPARENT);
    Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        anti_alias: true,
        ..Paint::default()
    }
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

impl From<FillRule> for TsFillRule {
    fn from(r: FillRule) -> Self {
        match r {
            FillRule::NonZero => TsFillRule::Winding,
            FillRule::EvenOdd => TsFillRule::EvenOdd,
        }
    }
}

impl From<LineCap> for TsCap {
    fn from(c: LineCap) -> Self {
        match c {
            LineCap::Butt => TsCap::Butt,
            LineCap::Round => TsCap::Round,
            LineCap::Square => TsCap::Square,
        }
    }
}

impl From<LineJoin> for TsJoin {
    fn from(j: LineJoin) -> Self {
        match j {
            // 上流は `MiterClip` も持つが、Lottie の `line-join` は3値しかない。
            // 4つ目を勝手に足すと、地図に無い語彙が Document へ入る。
            LineJoin::Miter => TsJoin::Miter,
            LineJoin::Round => TsJoin::Round,
            LineJoin::Bevel => TsJoin::Bevel,
        }
    }
}

pub(crate) fn new_pixmap(canvas: &Canvas) -> Result<Pixmap, VectorError> {
    Pixmap::new(canvas.width, canvas.height).ok_or(VectorError::CanvasSize {
        width: canvas.width,
        height: canvas.height,
    })
}

/// 1インスタンスを塗って線を引く。順は AE と同じく **fill が下、stroke が上**。
pub(crate) fn draw(
    pixmap: &mut Pixmap,
    path: &Path,
    origin: Point,
    fill: Option<&Fill>,
    stroke: Option<&Stroke>,
    weight: f64,
) {
    let Some(ts_path) = to_tiny_skia(path, origin) else {
        return;
    };
    if let Some(f) = fill.filter(|f| !f.hidden) {
        let paint = paint_for(f.color.r, f.color.g, f.color.b, f.opacity * weight);
        pixmap.fill_path(&ts_path, &paint, f.rule.into(), Transform::identity(), None);
    }
    if let Some(s) = stroke.filter(|s| !s.hidden && s.width > 0.0) {
        let paint = paint_for(s.color.r, s.color.g, s.color.b, s.opacity * weight);
        let ts_stroke = TsStroke {
            width: s.width as f32,
            miter_limit: s.miter_limit as f32,
            line_cap: s.cap.into(),
            line_join: s.join.into(),
            // 破線は上流の `StrokeDash` そのまま。`None` を返す入力(奇数長・負値)は
            // 「破線でない」へ落ちる — 上流の判定を写し取らない。
            dash: s.dash.as_ref().and_then(|d| {
                StrokeDash::new(
                    d.pattern.iter().map(|v| *v as f32).collect(),
                    d.offset as f32,
                )
            }),
        };
        pixmap.stroke_path(&ts_path, &paint, &ts_stroke, Transform::identity(), None);
    }
}

pub(crate) fn finish(pixmap: Pixmap, canvas: &Canvas) -> Raster {
    Raster {
        width: canvas.width,
        height: canvas.height,
        premultiplied_rgba8: pixmap.take(),
    }
}
