//! owns: パス演算子(trim-path / repeater / rounded-corners / pucker-bloat / zig-zag /
//!       offset-path / twist)と星形のパス源。上流のラスタライザはパスを**塗る**ことしか
//!       知らず、「パスを弧長で切る」「アフィンの実数冪でコピーを並べる」「角を fillet へ
//!       置換する」「頂点を重心へ寄せる」「辺を法線方向へ振る」「輪郭を太らせる」を
//!       持っている 2D crate は無い。旧 workspace `crates/motolii-doc/src/pathgeom.rs`
//!       からの**移植**であって再実装ではない(裁定10)。
//!
//! OWNS-JUSTIFICATION(D): 立証不足 — パス演算子7種の**実装側**の借り先
//! (`kurbo`/`lyon_algorithms` 等)を具体的に検討した記録が無い。裁定10(移植は
//! 再実装より優先)は「旧 `pathgeom.rs` をコピーしてよい理由」であって「自前で
//! 持つべき理由」の代わりにはならない(裁定215 棚卸し 2026-08-23 #16、発注書が
//! 名指しした2件の1つ)。**捏造しない正直な記録**: `kurbo`(弧長計算・オフセット
//! 曲線)・`lyon_algorithms`(弧長ウォーク)はどちらも既に Cargo.lock の
//! dependency graph に(他crate経由で)存在し、新規依存追加なしで検討可能——
//! ただし repeater/pucker-bloat/zig-zag/rounded-corners 一式をまとめて持つ2D
//! crateがある見込みは薄い(部分借用止まりの可能性が高い、同棚卸し §4)。動いて
//! いる実装は教義のために壊さない・借り換えない。
//!
//! **ラスタライズは自前で持っていない** — `raster.rs` は `tiny-skia` を包んだだけで、
//! そちらの marker は module の doc が持つ(crate の根しか見ない `check.sh` の
//! 空振りを避ける形は裁定34 と同じ)。選定理由も `raster.rs` に書いてある。
//!
//! # 出口はラスタライズ経路1本(裁定173 H4 で2つの口に精密化)
//!
//! [`render`](1つの `Shape`)と [`render_tree`](入れ子の [`ShapeNode`] 列)。
//! **後者が唯一の実体**で、前者は `render_tree(&[ShapeNode::Leaf(shape.clone())],
//! canvas)` と等価な糖衣 — ラスタライズする経路(`raster::new_pixmap`/`draw`/
//! `finish`)は依然として1本のまま(裁定18 の趣旨は「同じ仕事を2箇所に書かない」
//! であって「関数が1個だけ」ではない、と精密化)。**図形の記述 → premultiplied
//! RGBA8** 以外の公開口は置かない。出た RGBA は合成器の既存 `TexturedRect` 経路へ
//! そのまま流す想定で、**この crate は `motolii-compositor` を引かない**(背骨2)。
//!
//! # 形(裁定73・裁定173 H4)
//!
//! `Shape` 1つが持つのは「**1つのパス源 + 順序付き演算子スタック + fill/stroke
//! 各1**」——ここは裁定73 のまま1バイトも変えていない(`next/engine/
//! motolii-engine/src/mask.rs` 等の既存呼び手が struct literal で直に組んでいる
//! ため)。Lottie の `group.it`(兄弟順スコープ)は採らない — 修飾子が「同じ配列の
//! 手前の兄弟」に暗黙に効く模型は、**並べ替えが結果を黙って変える**(裁定66 と同型)。
//!
//! その代わり [`PathSource::Bezier`] が**輪郭の列**を持つ。Lottie では複数輪郭は
//! `group.it` に `path` を並べて作るが、ここでは1つのパス源が最初から複数輪郭を持てる。
//! **中マドは [`FillRule::EvenOdd`] が開ける**ので、パスブーリアンは要らない(裁定74)。
//!
//! 複数の `Shape` を**入れ子**にする話(shape 間の共有 transform)は裁定73 の外に
//! [`ShapeNode`]/[`ShapeGroup`]/[`flatten`] として足した(裁定173 H4、`group.rs`
//! module doc 参照)。「1つの `Shape` の形」自体は変えていない。
//!
//! # 編集
//!
//! [`PathSource::Bezier`] の中身(`values/bezier` の `v`/`i`/`o`/`c` — Lottie 公式
//! スキーマ、`reference/lottie-coverage.tsv` 参照。全4フィールド採用済)への
//! 頂点単位の読み書き(追加/削除/移動・ハンドル設定・閉路の開閉・セグメント分割)は
//! [`edit`] に集めた。UI 側のジェスチャがどれをどう呼ぶかは次波(`edit.rs`
//! module doc「UI 側への口」参照)。
//! # 保存(layer-meta 束)
//!
//! `Shape` とその内側の型に `serde` を足してある。**中身の語彙は増やしていない**
//! (derive を足しただけ)。`motolii-store` の shape-layer は `Vec<ShapeNode>` を
//! そのまま Document の component として持つ(裁定173 H4 — 旧 `Vec<Shape>` は
//! `ShapeNode::Leaf` の列として後方互換に読める)。「1つのパス源+演算子スタック」の
//! 正本はここ1つのままで、store 側に別のパス表現を発明させない(裁定10 と同じ考え方)。
//! `Canvas` / `Raster` / `VectorError` は描画専用の値なので保存の対象外(derive していない)。

/// `values/bezier`(Lottie 公式スキーマ、全4フィールド採用済)に対する頂点編集。
/// `insert_vertex`/`move_vertex`/`split_segment` 等は `coverage` と同じ理由
/// (「1つの図形の記述」とは別の関心事 — こちらは**編集**であって**記述**ではない)で、
/// crate 根の `pub use` には混ぜず独立した名前空間として公開する。
/// module doc(`edit.rs`)参照。
pub mod edit;
/// `Shape.ops`(演算子スタック — `shapes/*` modifier)に対する編集。`edit.rs` が
/// `values/bezier` を編集単位にしたのと同じ理由(**編集**であって**記述**ではない)
/// で、独立した名前空間として公開する。module doc(`stack_edit.rs`)参照。
pub mod stack_edit;
mod geom;
/// シェイプ内の入れ子グループ(裁定173 H4)。`ShapeNode`/`ShapeGroup`/`flatten`/
/// `render_tree` は crate 根から `pub use` で直に公開する — `coverage` と違い
/// 「1つの図形の記述」という同じ関心事(`Shape` の隣)なので、独立した名前空間には
/// 分けない(module doc の「出口」節参照)。
mod group;
mod ops;
mod raster;

/// raster 後処理の被覆(coverage)合成代数(MK2)。crate 根の `pub use` には混ぜず、
/// 独立した名前空間として公開する — module doc(`coverage.rs`)参照。
pub mod coverage;

/// 字形→輪郭(裁定190 切片1)。`coverage` と同じ理由で独立した名前空間 —
/// module doc(`text.rs`)参照。cosmic-text 0.19 を direct 依存として引く
/// (Cargo.toml のコメント参照、依存増分ゼロ)。
pub mod text;

use serde::{Deserialize, Serialize};

// パスの器は**入力の語彙**なので公開する。`rect` / `ellipse` は公開しない —
// 同じパスが `PathSource::Rectangle` からも作れることになり、口が2本になる(軸4)。
pub use geom::{Contour, Path, Point, Vertex};
/// 入れ子グループ(裁定173 H4)。module doc(`group.rs`)参照。
pub use group::{flatten, render_tree, ShapeGroup, ShapeNode};

use geom::{ellipse, polystar, rect};
use ops::Instance;

// ---------------------------------------------------------------------------
// 図形の記述
// ---------------------------------------------------------------------------

/// 図形1つ。`shape-1` が扱う全語彙がここに集まっている。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// パス源。`shapes/path` `shapes/rectangle` `shapes/ellipse` `shapes/polystar` の4つ。
///
/// 位置・回転を持たないのは裁定74 — 層の transform と repeater の anchor で賄える。
/// `polystar` の `is`/`os`(Inner/Outer Roundness)も同じ理由で持たない
/// ([`OpKind::RoundedCorners`] と同じ見た目に2つ目の口を作ることになる)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSource {
    /// `path.ks` — **ベジェパスの正本**。SVG・テキストアウトライン・primitive は
    /// すべてこの型へ落ちる。輪郭が複数持てるのが `group.it` を落とした代わり。
    Bezier(Path),
    /// `rectangle.s` — パスを生成するパラメータであって素材の寸法ではない(裁定59 とは別物)。
    /// scale と違い、あとから線幅・角丸が伸びない。
    Rectangle { size: Point },
    /// `ellipse.s` — 同上。
    Ellipse { size: Point },
    /// `polystar` — 星と正多角形。
    PolyStar {
        /// `polystar.pt` — 星なら腕の数(頂点は 2倍)、多角形なら辺の数。
        points: f64,
        /// `polystar.or`
        outer_radius: f64,
        /// `polystar.ir` — **[`StarType::Star`] のときだけ意味を持つ**。
        inner_radius: f64,
        /// `polystar.sy`
        star_type: StarType,
    },
}

impl PathSource {
    fn to_path(&self) -> Path {
        match self {
            PathSource::Bezier(p) => p.clone(),
            PathSource::Rectangle { size } => rect(*size),
            PathSource::Ellipse { size } => ellipse(*size),
            PathSource::PolyStar {
                points,
                outer_radius,
                inner_radius,
                star_type,
            } => polystar(*points, *outer_radius, *inner_radius, *star_type),
        }
    }
}

/// 演算子スタックの1段。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 演算子。`shape-1` で3つ、`shape-2` で3つ、`shape-3` で1つ。
///
/// **どれも「パス → パス」**(repeater だけが1枚を複数枚へ増やす)。
/// 段の意味は「1つ前の出力を受け取って次へ渡す」以上でも以下でもない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// `pucker-bloat` — 頂点を重心へ寄せる(+)/遠ざける(-)。
    PuckerBloat {
        /// `pucker-bloat.a` — `0` が恒等、`+1` で重心へ潰れる。
        amount: f64,
    },
    /// `zig-zag` — 辺を法線方向へ交互に振る。
    ZigZag {
        /// `zig-zag.s` — 山谷の振幅。
        amplitude: f64,
        /// `zig-zag.r` — **辺あたり**の山数。
        frequency: f64,
        /// `zig-zag.pt`
        point_type: PointType,
    },
    /// `offset-path` — 輪郭を法線方向へ平行移動して太らせる/痩せさせる。
    ///
    /// **v1 は閉路限定**。開いた輪郭が来たら [`VectorError::OpenPathOffset`] で、
    /// 静かに恒等へ落とさない(裁定37)。
    OffsetPath {
        /// `offset-path.a` — 正で外側、負で内側。
        amount: f64,
        /// `offset-path.lj` — `base-stroke.lj` と**同じ enum を共有する**。
        join: LineJoin,
        /// `offset-path.ml`
        miter_limit: f64,
    },
    /// `twist` — 中心ほど強くねじる。輪郭自身の外縁でねじれがゼロになる
    /// (半径パラメータを持たない = AE Twist と同じ自己正規化)。
    Twist {
        /// `twist.a` — **度**(裁定58)。
        angle: f64,
        /// `twist.c`
        center: Point,
    },
}

/// `repeater.tr`。旧 `pathgeom::ResolvedTransform` の移植。
///
/// 適用順序は `T(position) · R(rotation) · S(scale) · T(-anchor)` で、裁定58 と同じ。
/// `rotation` は**度**(裁定58「度のまま。人が読める」)、`scale` は 1.0 基準。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

/// 塗り方。`fill` と `gradient-fill`、`stroke` と `gradient-stroke` の違いは**これ1つ**。
///
/// Lottie は4つの型を並べるが、採らない — `gradient-fill` は `fill` に
/// `fill-rule` も `opacity` も `hidden` も全部持っており、違うのは「単色か色停止列か」
/// だけである。型を割ると**同じ意味の property が2箇所に生える**(裁定59 の形)。
/// Brush は fill/stroke に**直交**するので、線にも同じ塗りがそのまま乗る。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Brush {
    /// `fill.c` / `stroke.c`
    Solid(Rgb),
    /// `gradient-fill` / `gradient-stroke` の `g`(= `base-gradient`)。
    Gradient(Gradient),
}

impl Default for Brush {
    fn default() -> Self {
        Brush::Solid(Rgb::BLACK)
    }
}

/// `shapes/base-gradient`。
///
/// **停止点は alpha を持たない** — [`Rgb`] と同じ理由で、不透明度の正本は
/// `shape-style.o` 1本である(裁定59 の形を作らない)。Lottie は `g` の中に
/// 色停止と不透明度停止を混ぜて置けるが、それは1枚 JSON へ畳むための都合であって
/// 編集器の意味ではない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    /// `base-gradient.t`
    pub kind: GradientType,
    /// `base-gradient.s` — linear なら始点、radial なら**中心**。
    pub start: Point,
    /// `base-gradient.e` — linear なら終点、radial なら**外周上の1点**(半径を決める)。
    pub end: Point,
    /// `base-gradient.g` — 色停止列。
    pub stops: Vec<GradientStop>,
}

/// 色停止1つ。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// 0..1。
    pub offset: f64,
    pub color: Rgb,
}

/// `constants/gradient-type`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GradientType {
    #[default]
    Linear,
    Radial,
}

/// `fill` / `gradient-fill`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    /// `fill.c` か `base-gradient`。
    pub brush: Brush,
    /// `fill.r` / `gradient-fill.r` — **merge-paths を落とした分の中マドをここで賄う**(裁定74)。
    pub rule: FillRule,
    /// `shape-style.o` — 1.0 基準(パーセントは採らない。裁定65)。
    pub opacity: f64,
    /// `graphic-element.hd`
    pub hidden: bool,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            brush: Brush::default(),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }
    }
}

/// `stroke` / `gradient-stroke` + `base-stroke`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// `stroke.c` か `base-gradient`。
    pub brush: Brush,
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
            brush: Brush::default(),
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dash {
    /// 線・間隙の長さが交互に並ぶ。**偶数個**でないと上流が破線として受け付けない。
    pub pattern: Vec<f64>,
    /// 位相のずらし。
    pub offset: f64,
}

/// `constants/fill-rule`。**中マドを賄う**(裁定74)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// `constants/line-cap`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// `constants/line-join`。`offset-path`(shape-2)と共有する enum。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// `constants/star-type`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StarType {
    /// 外半径と内半径が交互に並ぶ。
    #[default]
    Star,
    /// 外半径だけ。`inner_radius` は読まれない。
    Polygon,
}

/// `zig-zag.pt` — 山の頂点をコーナーにするか滑らかにするか。
///
/// `constants` に独立した行を持たない(Lottie では zig-zag だけが使う語彙)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PointType {
    #[default]
    Corner,
    Smooth,
}

/// `constants/composite` — repeater の Above/Below。**描く順**を決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Composite {
    /// コピー0 が先に描かれ、後のコピーが上に載る。
    #[default]
    Above,
    /// 逆順。
    Below,
}

/// `constants/trim-multiple-shapes` — trim の複数輪郭モード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    /// `offset-path` は v1 では閉路しか扱わない。開いた輪郭を**黙って素通しにしない** —
    /// 素通しにすると「演算子を足したのに何も起きない」という、**利用者から見て
    /// 何も壊れていないように見える欠陥**になる(裁定37 と同じ話)。
    ///
    /// `trim-path` は閉路を開くので、`trim` → `offset` の並びがここに来る。
    #[error("offset-path is closed-contour only; an open contour reached it")]
    OpenPathOffset,
}

/// 図形1つの記述 → premultiplied RGBA8。**単一 `Shape` 用の糖衣**(裁定173 H4) —
/// 中身は [`render_tree`]`(&[ShapeNode::Leaf(shape.clone())], canvas)` と文字通り
/// 同じ呼び出しで、ラスタライズする経路は2箇所に書かれていない。既存呼び手
/// (`next/engine/motolii-engine/src/mask.rs` 等)はシグネチャ・挙動とも無改修。
///
/// 同じ記述からは何度呼んでも byte 一致する(上流も演算子も CPU の純関数)。
/// 試験 `the_same_description_renders_byte_identical_twice` がそれを縛る。
pub fn render(shape: &Shape, canvas: &Canvas) -> Result<Raster, VectorError> {
    render_tree(&[ShapeNode::Leaf(shape.clone())], canvas)
}

/// 演算子スタックを畳んで、描くべきパスと重みの列にする。
///
/// スタックが**順序付き**であるとはここが `for` で回ることに他ならない。
/// 「手前の兄弟に暗黙に効く」模型(`group.it`)を採らなかったので、
/// 段の意味は「1つ前の出力を受け取って次へ渡す」以上でも以下でもない。
fn resolve(shape: &Shape) -> Result<Vec<Instance>, VectorError> {
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
            OpKind::PuckerBloat { amount } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::pucker_bloat(&i.path, *amount),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::ZigZag {
                amplitude,
                frequency,
                point_type,
            } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::zigzag(&i.path, *amplitude, *frequency, *point_type),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::OffsetPath {
                amount,
                join,
                miter_limit,
            } => instances
                .into_iter()
                .map(|i| {
                    Ok(Instance {
                        path: ops::offset_path(&i.path, *amount, *join, *miter_limit)?,
                        opacity: i.opacity,
                    })
                })
                .collect::<Result<Vec<_>, VectorError>>()?,
            OpKind::Twist { angle, center } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::twist(&i.path, *angle, *center),
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
    Ok(instances)
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
            .expect("この試験の入力に offset-path は無い")
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
        })
        .expect("この試験の入力に offset-path は無い");
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
        })
        .expect("この試験の入力に offset-path は無い");
        let contours: Vec<_> = out.iter().flat_map(|i| i.path.iter()).collect();
        assert!(!contours.is_empty(), "trim で全部消えた");
        assert!(contours.iter().all(|c| !c.closed));
    }
}
