//! owns: パス演算子(trim-path / repeater / rounded-corners)。上流のラスタライザは
//!       パスを**塗る**ことしか知らず、「パスを弧長で切る」「アフィンの実数冪でコピーを
//!       並べる」「角を fillet へ置換する」を持っている 2D crate は無い。旧 workspace
//!       `crates/motolii-doc/src/pathgeom.rs` からの**移植**であって再実装ではない(裁定10)。
//!
//! **ラスタライズは自前で持っていない** — `raster.rs` は `tiny-skia` を包んだだけで、
//! そちらの marker は module の doc が持つ(crate の根しか見ない `check.sh` の
//! 空振りを避ける形は裁定34 と同じ)。選定理由も `raster.rs` に書いてある。
//!
//! # 出口は1本
//!
//! [`render`] だけ。**図形の記述 → premultiplied RGBA8** で、それ以外の公開口を
//! 置かない(裁定18 と同じ趣旨 — 第二経路が作れる口があると、片方だけ直る欠陥が生まれる)。
//! 出た RGBA は合成器の既存 `TexturedRect` 経路へそのまま流す想定で、
//! **この crate は `motolii-compositor` を引かない**(背骨2)。
//!
//! # 形(裁定73)
//!
//! 持つのは「**1つのパス源 + 順序付き演算子スタック + fill/stroke 各1**」。
//! Lottie の `group.it`(兄弟順スコープ)は採らない — 修飾子が「同じ配列の手前の
//! 兄弟」に暗黙に効く模型は、**並べ替えが結果を黙って変える**(裁定66 と同型)。
//!
//! その代わり [`PathSource::Bezier`] が**輪郭の列**を持つ。Lottie では複数輪郭は
//! `group.it` に `path` を並べて作るが、ここでは1つのパス源が最初から複数輪郭を持てる。
//! **中マドは [`FillRule::EvenOdd`] が開ける**ので、パスブーリアンは要らない(裁定74)。

mod geom;
mod ops;
mod raster;

// パスの器は**入力の語彙**なので公開する。`rect` / `ellipse` は公開しない —
// 同じパスが `PathSource::Rectangle` からも作れることになり、口が2本になる(軸4)。
pub use geom::{Contour, Path, Point, Vertex};

use geom::{ellipse, rect};
use ops::Instance;

// ---------------------------------------------------------------------------
// 図形の記述
// ---------------------------------------------------------------------------

/// 図形1つ。`shape-1` が扱う全語彙がここに集まっている。
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// パス源(`path.ks` / `rectangle.s` / `ellipse.s`)。
    pub source: PathSource,
    /// 順序付き演算子スタック。**手前から順に**掛かる。
    pub ops: Vec<ShapeOp>,
    /// `fill`。層が持てるのは1つ(裁定73)。
    pub fill: Option<Fill>,
    /// `stroke`。同じく1つ。
    pub stroke: Option<Stroke>,
}

impl Shape {
    /// 演算子もスタイルも無い裸のパス。**何も描かれない** — スタイルが無い図形は
    /// 「透明な図形」ではなく「まだ塗り方が決まっていない図形」なので、既定色を
    /// 発明しない(裁定20 の「キーを打っていない property は静止値」とは別の話で、
    /// ここは静止値そのものが Lottie に無い)。
    pub fn new(source: PathSource) -> Self {
        Self {
            source,
            ops: Vec::new(),
            fill: None,
            stroke: None,
        }
    }
}

/// パス源。`shapes/path` `shapes/rectangle` `shapes/ellipse` の3つ。
///
/// 位置・回転を持たないのは裁定74 — 層の transform と repeater の anchor で賄える。
#[derive(Debug, Clone, PartialEq)]
pub enum PathSource {
    /// `path.ks` — **ベジェパスの正本**。SVG・テキストアウトライン・primitive は
    /// すべてこの型へ落ちる。輪郭が複数持てるのが `group.it` を落とした代わり。
    Bezier(Path),
    /// `rectangle.s` — パスを生成するパラメータであって素材の寸法ではない(裁定59 とは別物)。
    /// scale と違い、あとから線幅・角丸が伸びない。
    Rectangle { size: Point },
    /// `ellipse.s` — 同上。
    Ellipse { size: Point },
}

impl PathSource {
    fn to_path(&self) -> Path {
        match self {
            PathSource::Bezier(p) => p.clone(),
            PathSource::Rectangle { size } => rect(*size),
            PathSource::Ellipse { size } => ellipse(*size),
        }
    }
}

/// 演算子スタックの1段。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeOp {
    /// `graphic-element.hd` — この1段だけの有効/無効。**層の visible とは別**で、
    /// 消すのではなく黙らせる(設定を失わずに切れる)。
    pub hidden: bool,
    /// `graphic-element.ty` は Rust の enum variant tag として自然に出る。
    pub kind: OpKind,
}

impl ShapeOp {
    pub fn new(kind: OpKind) -> Self {
        Self {
            hidden: false,
            kind,
        }
    }
}

/// 演算子。`shape-1` の範囲は3つ。
#[derive(Debug, Clone, PartialEq)]
pub enum OpKind {
    /// `trim-path` — MV で最も使う語彙。`start`/`end` は 0..1、`offset` は窓の回転。
    TrimPath {
        /// `trim-path.s`
        start: f64,
        /// `trim-path.e`
        end: f64,
        /// `trim-path.o` — 切り出し窓の回転。ぐるぐる回る線はこれ。
        offset: f64,
        /// `trim-path.m`
        multiple: TrimMultiple,
    },
    /// `repeater` — trim-path と並ぶ MV の主語彙。
    Repeater {
        /// `repeater.c`
        copies: f64,
        /// `repeater.o` — 開始インデックスのずらし。
        offset: f64,
        /// `repeater.tr` — **anchor を持つので放射状配置がここで成立する**。
        transform: RepeaterTransform,
        /// `repeater.m`
        composite: Composite,
        /// `repeater-transform.so`
        start_opacity: f64,
        /// `repeater-transform.eo`
        end_opacity: f64,
    },
    /// `rounded-corners` — `rectangle.r` と `polystar.roundness` を落とした受け皿(裁定74)。
    RoundedCorners {
        /// `rounded-corners.r`
        radius: f64,
    },
}

/// `repeater.tr`。旧 `pathgeom::ResolvedTransform` の移植。
///
/// 適用順序は `T(position) · R(rotation) · S(scale) · T(-anchor)` で、裁定58 と同じ。
/// `rotation` は**度**(裁定58「度のまま。人が読める」)、`scale` は 1.0 基準。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepeaterTransform {
    pub anchor: Point,
    pub position: Point,
    pub scale: Point,
    pub rotation: f64,
}

impl RepeaterTransform {
    pub const IDENTITY: RepeaterTransform = RepeaterTransform {
        anchor: Point::ZERO,
        position: Point::ZERO,
        scale: Point { x: 1.0, y: 1.0 },
        rotation: 0.0,
    };
}

impl Default for RepeaterTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ---------------------------------------------------------------------------
// スタイル
// ---------------------------------------------------------------------------

/// 直線(非 premultiplied)RGB。**alpha を持たない** — 不透明度は `shape-style.o` が
/// 1本で持つ。両方持つと同じことを言う正本が2つになる(裁定59 の形)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
}

/// `fill`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    /// `fill.c`
    pub color: Rgb,
    /// `fill.r` — **merge-paths を落とした分の中マドをここで賄う**(裁定74)。
    pub rule: FillRule,
    /// `shape-style.o` — 1.0 基準(パーセントは採らない。裁定65)。
    pub opacity: f64,
    /// `graphic-element.hd`
    pub hidden: bool,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            color: Rgb::BLACK,
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }
    }
}

/// `stroke` + `base-stroke`。
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// `stroke.c`
    pub color: Rgb,
    /// `base-stroke.w`
    pub width: f64,
    /// `base-stroke.lc`
    pub cap: LineCap,
    /// `base-stroke.lj`
    pub join: LineJoin,
    /// `base-stroke.ml2` — miter limit はこの1本(Lottie の deprecated な `ml` は採らない)。
    pub miter_limit: f64,
    /// `base-stroke.d`
    pub dash: Option<Dash>,
    /// `shape-style.o`
    pub opacity: f64,
    /// `graphic-element.hd`
    pub hidden: bool,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: Rgb::BLACK,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
            dash: None,
            opacity: 1.0,
            hidden: false,
        }
    }
}

/// `base-stroke.d`(Dashes)。
///
/// Lottie は `[{n:"d",v:..},{n:"g",v:..},{n:"o",v:..}]` という**種別つきの列**で持つが、
/// 採らない — 同じ破線が何通りにも書けてしまい(`d,g,d,g` と `d,g` の繰り返しが同義)、
/// 正本が1つにならない。ラスタライザが受け取る平坦形(長さの列 + 位相)で持つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Dash {
    /// 線・間隙の長さが交互に並ぶ。**偶数個**でないと上流が破線として受け付けない。
    pub pattern: Vec<f64>,
    /// 位相のずらし。
    pub offset: f64,
}

/// `constants/fill-rule`。**中マドを賄う**(裁定74)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// `constants/line-cap`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// `constants/line-join`。`offset-path`(shape-2)と共有する enum。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// `constants/composite` — repeater の Above/Below。**描く順**を決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Composite {
    /// コピー0 が先に描かれ、後のコピーが上に載る。
    #[default]
    Above,
    /// 逆順。
    Below,
}

/// `constants/trim-multiple-shapes` — trim の複数輪郭モード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrimMultiple {
    /// 輪郭ごとに**同じ窓**で切る。全部が同時に伸びる。
    #[default]
    Simultaneously,
    /// 輪郭を連結した**1つの長さ空間**として切る。1本ずつ順に描かれる。
    Individually,
}

// ---------------------------------------------------------------------------
// 出口
// ---------------------------------------------------------------------------

/// 描く板。`origin` は図形の局所原点が載る画素座標。
///
/// **層の transform は持たない** — 位置・回転・拡大は層が持ち(裁定58/69)、
/// この crate はそれより内側に居る。ここにあるのは板の framing だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
}

impl Canvas {
    /// 局所原点を板の中央へ置く。図形源が原点中央に作られるので、これが自然な既定。
    pub fn centered(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            origin_x: (width / 2) as i32,
            origin_y: (height / 2) as i32,
        }
    }
}

/// 出た画。**premultiplied RGBA8**、行優先・隙間なし(`width * height * 4` バイト)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub premultiplied_rgba8: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VectorError {
    /// 板が作れない大きさ。**空の画を静かに返さない** — 「無い」と「読めない」を
    /// 同義にすると、壊れた入力が黙って既定値へ落ちる(裁定37)。
    #[error("canvas size {width}x{height} cannot be rasterized")]
    CanvasSize { width: u32, height: u32 },
}

/// **この crate の唯一の公開口**: 図形の記述 → premultiplied RGBA8。
///
/// 同じ記述からは何度呼んでも byte 一致する(上流も演算子も CPU の純関数)。
/// 試験 `the_same_description_renders_byte_identical_twice` がそれを縛る。
pub fn render(shape: &Shape, canvas: &Canvas) -> Result<Raster, VectorError> {
    let mut pixmap = raster::new_pixmap(canvas)?;
    let origin = Point {
        x: canvas.origin_x as f64,
        y: canvas.origin_y as f64,
    };
    for instance in resolve(shape) {
        raster::draw(
            &mut pixmap,
            &instance.path,
            origin,
            shape.fill.as_ref(),
            shape.stroke.as_ref(),
            instance.opacity,
        );
    }
    Ok(raster::finish(pixmap, canvas))
}

/// 演算子スタックを畳んで、描くべきパスと重みの列にする。
///
/// スタックが**順序付き**であるとはここが `for` で回ることに他ならない。
/// 「手前の兄弟に暗黙に効く」模型(`group.it`)を採らなかったので、
/// 段の意味は「1つ前の出力を受け取って次へ渡す」以上でも以下でもない。
fn resolve(shape: &Shape) -> Vec<Instance> {
    let mut instances = vec![Instance {
        path: shape.source.to_path(),
        opacity: 1.0,
    }];
    for op in &shape.ops {
        if op.hidden {
            continue;
        }
        instances = match &op.kind {
            OpKind::TrimPath {
                start,
                end,
                offset,
                multiple,
            } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::trim(&i.path, *start, *end, *offset, *multiple),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::RoundedCorners { radius } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::round_corners(&i.path, *radius),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::Repeater {
                copies,
                offset,
                transform,
                composite,
                start_opacity,
                end_opacity,
            } => ops::repeater(
                &instances,
                *copies,
                *offset,
                transform,
                *composite,
                *start_opacity,
                *end_opacity,
            ),
        };
    }
    instances
}

/// 幾何だけを見たい試験はここに置く。
///
/// 「角丸で頂点が増える」は画素からは観測できないが、それを見るために
/// `pub fn debug_*` を生やすと**公開口が2本になる**。crate の中なら
/// `resolve` へ直に触れるので、口を増やさずに同じことが確かめられる。
#[cfg(test)]
mod geometry_tests {
    use super::*;

    fn square() -> PathSource {
        PathSource::Rectangle {
            size: Point { x: 100.0, y: 100.0 },
        }
    }

    fn vertex_count(shape: &Shape) -> usize {
        resolve(shape)
            .iter()
            .flat_map(|i| i.path.iter())
            .map(|c| c.vertices.len())
            .sum()
    }

    /// 丸めは「角を弧へ置換する」ので輪郭の点が増える。
    #[test]
    fn rounded_corners_add_vertices() {
        let before = vertex_count(&Shape::new(square()));
        let after = vertex_count(&Shape {
            ops: vec![ShapeOp::new(OpKind::RoundedCorners { radius: 20.0 })],
            ..Shape::new(square())
        });
        assert!(
            after > before,
            "角丸で頂点が増えていない({before} → {after})"
        );
    }

    /// 角丸は**閉じた輪郭**のまま。開いてしまうと fill が別物になる。
    #[test]
    fn rounded_corners_keep_the_contour_closed() {
        let out = resolve(&Shape {
            ops: vec![ShapeOp::new(OpKind::RoundedCorners { radius: 20.0 })],
            ..Shape::new(square())
        });
        assert!(out.iter().flat_map(|i| i.path.iter()).all(|c| c.closed));
    }

    /// trim は閉路を**開いた**輪郭へ落とす(切り口が端になるため)。
    #[test]
    fn trim_opens_the_contour() {
        let out = resolve(&Shape {
            ops: vec![ShapeOp::new(OpKind::TrimPath {
                start: 0.0,
                end: 0.5,
                offset: 0.0,
                multiple: TrimMultiple::Simultaneously,
            })],
            ..Shape::new(square())
        });
        let contours: Vec<_> = out.iter().flat_map(|i| i.path.iter()).collect();
        assert!(!contours.is_empty(), "trim で全部消えた");
        assert!(contours.iter().all(|c| !c.closed));
    }
}
