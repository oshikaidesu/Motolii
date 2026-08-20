//! `shape-2`(+ `shape-3` の twist)の受入条件。**先に落ちる試験として書いた**(実装より前)。
//!
//! `shape_1.rs` と同じ流儀 — 見るのは**出た画素**であって内部状態ではない。
//! 演算子の実装を差し替えても「膨らむ」「ぎざぎざになる」「輪郭が太る」
//! 「星の腕が伸びる」「色が端から端へ変わる」「内側だけねじれる」が
//! 観測できる限り緑であってほしいため。
//!
//! 例外は `offset-path` の開路拒否だけで、これは**画ではなく Err** が観測対象になる
//! (裁定37「無い」と「読めない」を区別する)。

use motolii_vector::{
    render, Brush, Canvas, Contour, Fill, FillRule, Gradient, GradientStop, GradientType, LineJoin,
    OpKind, PathSource, Point, PointType, Raster, Rgb, Shape, ShapeOp, StarType, Stroke,
    VectorError,
};

// ---------------------------------------------------------------------------
// 器具 — 画素を数える/拾うだけ。ここに幾何を持ち込むと試験が実装の写しになる。
// ---------------------------------------------------------------------------

fn painted(r: &Raster) -> usize {
    r.premultiplied_rgba8
        .chunks_exact(4)
        .filter(|p| p[3] != 0)
        .count()
}

fn alpha_at(r: &Raster, x: u32, y: u32) -> u8 {
    r.premultiplied_rgba8[((y * r.width + x) * 4 + 3) as usize]
}

/// premultiplied のまま R/G/B を返す。**色の比較にしか使わない**。
fn rgb_at(r: &Raster, x: u32, y: u32) -> (u8, u8, u8) {
    let i = ((y * r.width + x) * 4) as usize;
    (
        r.premultiplied_rgba8[i],
        r.premultiplied_rgba8[i + 1],
        r.premultiplied_rgba8[i + 2],
    )
}

fn canvas() -> Canvas {
    Canvas::centered(200, 200)
}

fn square() -> PathSource {
    PathSource::Rectangle {
        size: Point { x: 100.0, y: 100.0 },
    }
}

fn black_fill() -> Fill {
    Fill {
        brush: Brush::Solid(Rgb::BLACK),
        ..Fill::default()
    }
}

fn black_stroke(width: f64) -> Stroke {
    Stroke {
        brush: Brush::Solid(Rgb::BLACK),
        width,
        ..Stroke::default()
    }
}

fn filled(source: PathSource, ops: Vec<ShapeOp>) -> Raster {
    render(
        &Shape {
            ops,
            fill: Some(black_fill()),
            ..Shape::new(source)
        },
        &canvas(),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// pucker-bloat — `a` は頂点を重心へ寄せる(+)/遠ざける(-)
// ---------------------------------------------------------------------------

fn pucker_bloat(amount: f64) -> ShapeOp {
    ShapeOp::new(OpKind::PuckerBloat { amount })
}

/// `pucker-bloat.a`(Amount)。**正で bloat / 負で pucker**(AE・Lottie と同じ向き)。
///
/// 正体は「頂点が重心へ寄り、**ハンドルが逆向きへ伸びる**」の同時進行なので、
/// 角が抜けて辺が膨らむ。頂点の移動だけを見ると向きを取り違えるので、
/// **角の内側と辺の外側の2点**を見る。
#[test]
fn pucker_bloat_amount_pulls_the_vertices_in_and_bulges_the_edges() {
    let base = filled(square(), vec![]);
    let bloat = filled(square(), vec![pucker_bloat(0.5)]);

    // 100x100 の正方形は板の [50,150] を占める。角のすぐ内側。
    assert_ne!(alpha_at(&base, 53, 53), 0, "元の正方形に角が無い");
    assert_eq!(
        alpha_at(&bloat, 53, 53),
        0,
        "bloat で頂点が重心へ寄っていない(角が残っている)"
    );
    // 上辺の**外側**。膨らんでいれば埋まる。
    assert_eq!(alpha_at(&base, 100, 45), 0, "元の正方形が辺の外へ出ている");
    assert_ne!(
        alpha_at(&bloat, 100, 45),
        0,
        "bloat で辺が膨らんでいない(ハンドルが伸びていない)"
    );
}

/// `pucker-bloat.a` は連続量。-0.75 → +0.75 で面積が単調に増える。
#[test]
fn pucker_bloat_amount_moves_the_area_monotonically() {
    let mut previous = 0usize;
    let mut areas = Vec::new();
    for step in 0..=6 {
        let amount = -0.75 + 0.25 * step as f64;
        let n = painted(&filled(square(), vec![pucker_bloat(amount)]));
        assert!(
            n >= previous,
            "amount={amount} で面積が減った({previous} → {n})。\
             pucker→bloat は単調に増えなければならない"
        );
        previous = n;
        areas.push(n);
    }
    assert!(
        areas[6] > areas[0],
        "端から端で面積が動いていない: {areas:?}"
    );
}

/// `pucker-bloat.ty`。演算子スタックの1段として乗り、`0` は恒等。
#[test]
fn pucker_bloat_is_identity_at_zero() {
    assert_eq!(
        filled(square(), vec![pucker_bloat(0.0)]).premultiplied_rgba8,
        filled(square(), vec![]).premultiplied_rgba8,
        "amount=0 が恒等になっていない"
    );
}

// ---------------------------------------------------------------------------
// zig-zag — `s`(振幅)/`r`(山数)/`pt`(角の型)
// ---------------------------------------------------------------------------

/// 原点を通る水平の開いた線。zigzag の観測台。
fn horizontal_line() -> PathSource {
    PathSource::Bezier(vec![Contour::open([
        Point { x: -80.0, y: 0.0 },
        Point { x: 80.0, y: 0.0 },
    ])])
}

fn zigzag(amplitude: f64, frequency: f64, point_type: PointType) -> ShapeOp {
    ShapeOp::new(OpKind::ZigZag {
        amplitude,
        frequency,
        point_type,
    })
}

fn stroked(source: PathSource, ops: Vec<ShapeOp>) -> Raster {
    render(
        &Shape {
            ops,
            stroke: Some(black_stroke(3.0)),
            ..Shape::new(source)
        },
        &canvas(),
    )
    .unwrap()
}

/// 画素が乗っている行の範囲(上端, 下端)。
fn ink_rows(r: &Raster) -> (u32, u32) {
    let mut lo = r.height;
    let mut hi = 0;
    for y in 0..r.height {
        if (0..r.width).any(|x| alpha_at(r, x, y) != 0) {
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    (lo, hi)
}

/// `zig-zag.s`(Amplitude)。線が baseline の上下へ振れて、縦の広がりが増える。
#[test]
fn zigzag_amplitude_pushes_the_line_off_its_baseline() {
    let flat = stroked(horizontal_line(), vec![]);
    let wavy = stroked(
        horizontal_line(),
        vec![zigzag(20.0, 4.0, PointType::Corner)],
    );
    let (flat_lo, flat_hi) = ink_rows(&flat);
    let (wavy_lo, wavy_hi) = ink_rows(&wavy);
    // 直線は太さ 3 の帯にしか居ない。振幅 20 なら ±20 に広がる。
    assert!(
        flat_hi - flat_lo <= 6,
        "元の線が baseline を離れている({flat_lo}..{flat_hi})"
    );
    assert!(
        wavy_hi - wavy_lo >= 30,
        "amplitude が画に出ていない(縦の広がりが {wavy_lo}..{wavy_hi} しかない)"
    );
    assert!(
        painted(&wavy) > painted(&flat),
        "ぎざぎざの方が線が長いはずなのに画素が増えていない"
    );
}

/// `zig-zag.r`(Frequency)。辺あたりの山数。多いほど線が長い。
#[test]
fn zigzag_frequency_adds_ridges() {
    let few = painted(&stroked(
        horizontal_line(),
        vec![zigzag(20.0, 2.0, PointType::Corner)],
    ));
    let many = painted(&stroked(
        horizontal_line(),
        vec![zigzag(20.0, 6.0, PointType::Corner)],
    ));
    assert!(
        many > few,
        "山数を増やしても線が長くなっていない({few} → {many})"
    );
}

/// `zig-zag.pt`(Point Type)。corner は折れ、smooth は曲がる。
#[test]
fn zigzag_point_type_changes_the_corners() {
    let corner = stroked(
        horizontal_line(),
        vec![zigzag(20.0, 4.0, PointType::Corner)],
    );
    let smooth = stroked(
        horizontal_line(),
        vec![zigzag(20.0, 4.0, PointType::Smooth)],
    );
    assert!(painted(&corner) > 0 && painted(&smooth) > 0);
    assert!(
        corner.premultiplied_rgba8 != smooth.premultiplied_rgba8,
        "corner と smooth が同じ画を出している"
    );
}

// ---------------------------------------------------------------------------
// offset-path — `a`(量)/`lj`(角の結合)/`ml`(miter limit)。v1 は閉路限定
// ---------------------------------------------------------------------------

fn offset_path(amount: f64, join: LineJoin, miter_limit: f64) -> ShapeOp {
    ShapeOp::new(OpKind::OffsetPath {
        amount,
        join,
        miter_limit,
    })
}

/// `offset-path.a`(Amount)。正で太り、負で痩せる。
#[test]
fn offset_path_amount_grows_and_shrinks_the_outline() {
    let base = painted(&filled(square(), vec![]));
    let out = painted(&filled(
        square(),
        vec![offset_path(20.0, LineJoin::Miter, 4.0)],
    ));
    let inn = painted(&filled(
        square(),
        vec![offset_path(-20.0, LineJoin::Miter, 4.0)],
    ));
    assert!(
        out > base,
        "外側 offset で面積が増えていない({base} → {out})"
    );
    assert!(
        inn < base,
        "内側 offset で面積が減っていない({base} → {inn})"
    );
}

/// `offset-path.lj`(Line Join)。角の埋め方が変われば画が変わる。
#[test]
fn offset_path_line_join_changes_the_corner() {
    let of = |join| filled(square(), vec![offset_path(20.0, join, 4.0)]);
    let miter = of(LineJoin::Miter);
    let bevel = of(LineJoin::Bevel);
    let round = of(LineJoin::Round);
    assert!(
        painted(&miter) > painted(&bevel),
        "miter の角が bevel より張り出していない"
    );
    assert!(
        round.premultiplied_rgba8 != bevel.premultiplied_rgba8,
        "round と bevel が同じ画を出している"
    );
}

/// `offset-path.ml`(Miter Limit)。上限を割ると bevel へ縮退する。
#[test]
fn offset_path_miter_limit_falls_back_to_bevel() {
    // 鋭角の閉路 — miter の張り出しが大きくなる形。
    let spike = PathSource::Bezier(vec![Contour::closed([
        Point { x: -70.0, y: 40.0 },
        Point { x: 0.0, y: -60.0 },
        Point { x: 70.0, y: 40.0 },
        Point { x: 0.0, y: 20.0 },
    ])]);
    let generous = filled(
        spike.clone(),
        vec![offset_path(12.0, LineJoin::Miter, 20.0)],
    );
    let tight = filled(spike.clone(), vec![offset_path(12.0, LineJoin::Miter, 1.0)]);
    let bevel = filled(spike, vec![offset_path(12.0, LineJoin::Bevel, 20.0)]);
    assert!(
        painted(&generous) > painted(&tight),
        "miter limit を絞っても張り出しが減っていない"
    );
    assert_eq!(
        tight.premultiplied_rgba8, bevel.premultiplied_rgba8,
        "miter limit 超過が bevel へ縮退していない"
    );
}

/// **開路は拒否する**(v1 は閉路限定)。静かに恒等へ落とさない(裁定37)。
#[test]
fn offset_path_refuses_an_open_contour() {
    let shape = Shape {
        ops: vec![offset_path(10.0, LineJoin::Miter, 4.0)],
        stroke: Some(black_stroke(3.0)),
        ..Shape::new(horizontal_line())
    };
    assert_eq!(
        render(&shape, &canvas()),
        Err(VectorError::OpenPathOffset),
        "開路の offset が黙って通っている"
    );
}

// ---------------------------------------------------------------------------
// polystar — `pt`(頂点数)/`or`(外半径)/`ir`(内半径)/`sy`(星か多角形か)
// ---------------------------------------------------------------------------

fn polystar(points: f64, outer_radius: f64, inner_radius: f64, star_type: StarType) -> PathSource {
    PathSource::PolyStar {
        points,
        outer_radius,
        inner_radius,
        star_type,
    }
}

/// `polystar.sy`(Star Type)。星は多角形より痩せている(腕の間が抜ける)。
#[test]
fn polystar_star_is_thinner_than_the_polygon() {
    let star = painted(&filled(polystar(5.0, 80.0, 35.0, StarType::Star), vec![]));
    let polygon = painted(&filled(
        polystar(5.0, 80.0, 35.0, StarType::Polygon),
        vec![],
    ));
    assert!(star > 0 && polygon > 0);
    assert!(
        star < polygon,
        "星が多角形より広い({star} vs {polygon})— 腕の間が抜けていない"
    );
}

/// `polystar.pt`(Points)。頂点数が変われば輪郭が変わる。
#[test]
fn polystar_points_change_the_silhouette() {
    let three = filled(polystar(3.0, 80.0, 40.0, StarType::Polygon), vec![]);
    let eight = filled(polystar(8.0, 80.0, 40.0, StarType::Polygon), vec![]);
    assert!(
        painted(&eight) > painted(&three),
        "正多角形は頂点数が増えるほど外接円へ近づく(面積が増える)"
    );
}

/// `polystar.or`(Outer Radius)。外半径が大きいほど広い。
#[test]
fn polystar_outer_radius_scales_the_shape() {
    let small = painted(&filled(polystar(5.0, 40.0, 20.0, StarType::Star), vec![]));
    let large = painted(&filled(polystar(5.0, 80.0, 40.0, StarType::Star), vec![]));
    assert!(large > small, "外半径で大きさが変わっていない");
}

/// `polystar.ir`(Inner Radius)。**星のときだけ**意味を持つ。
#[test]
fn polystar_inner_radius_only_matters_for_the_star() {
    let thin = painted(&filled(polystar(5.0, 80.0, 20.0, StarType::Star), vec![]));
    let fat = painted(&filled(polystar(5.0, 80.0, 60.0, StarType::Star), vec![]));
    assert!(
        fat > thin,
        "内半径が星の太さに効いていない({thin} vs {fat})"
    );

    let a = filled(polystar(5.0, 80.0, 20.0, StarType::Polygon), vec![]);
    let b = filled(polystar(5.0, 80.0, 60.0, StarType::Polygon), vec![]);
    assert_eq!(
        a.premultiplied_rgba8, b.premultiplied_rgba8,
        "多角形が内半径に反応している"
    );
}

// ---------------------------------------------------------------------------
// gradient-fill / gradient-stroke / base-gradient
// ---------------------------------------------------------------------------

fn red() -> Rgb {
    Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    }
}

fn blue() -> Rgb {
    Rgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    }
}

fn two_stop(kind: GradientType, start: Point, end: Point) -> Gradient {
    Gradient {
        kind,
        start,
        end,
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: red(),
            },
            GradientStop {
                offset: 1.0,
                color: blue(),
            },
        ],
    }
}

/// `gradient-fill.ty` + `base-gradient.s`/`e`/`g`/`t`。
/// 左端が赤・右端が青になる = 色が図形の上で動いている。
#[test]
fn linear_gradient_fill_runs_the_colour_across_the_shape() {
    let g = two_stop(
        GradientType::Linear,
        Point { x: -50.0, y: 0.0 },
        Point { x: 50.0, y: 0.0 },
    );
    let out = render(
        &Shape {
            fill: Some(Fill {
                brush: Brush::Gradient(g),
                ..Fill::default()
            }),
            ..Shape::new(square())
        },
        &canvas(),
    )
    .unwrap();
    let left = rgb_at(&out, 55, 100);
    let right = rgb_at(&out, 145, 100);
    assert!(
        left.0 > left.2,
        "左端が赤寄りでない: {left:?}(start の色が効いていない)"
    );
    assert!(
        right.2 > right.0,
        "右端が青寄りでない: {right:?}(end の色が効いていない)"
    );
}

/// `base-gradient.t`(Gradient Type)。linear と radial は別の画。
#[test]
fn radial_gradient_runs_the_colour_outward_from_the_centre() {
    let of = |kind| {
        let g = two_stop(kind, Point { x: 0.0, y: 0.0 }, Point { x: 50.0, y: 0.0 });
        render(
            &Shape {
                fill: Some(Fill {
                    brush: Brush::Gradient(g),
                    ..Fill::default()
                }),
                ..Shape::new(square())
            },
            &canvas(),
        )
        .unwrap()
    };
    let radial = of(GradientType::Radial);
    let linear = of(GradientType::Linear);
    assert!(
        radial.premultiplied_rgba8 != linear.premultiplied_rgba8,
        "radial と linear が同じ画を出している"
    );
    // radial は中心が赤、左右どちらの端も青。linear なら左端は赤のまま。
    let centre = rgb_at(&radial, 100, 100);
    let left = rgb_at(&radial, 55, 100);
    let right = rgb_at(&radial, 145, 100);
    assert!(
        centre.0 > centre.2,
        "radial の中心が start 色でない: {centre:?}"
    );
    assert!(left.2 > left.0, "radial の左端が end 色でない: {left:?}");
    assert!(right.2 > right.0, "radial の右端が end 色でない: {right:?}");
}

/// `gradient-fill.r`(Fill Rule)。solid と同じく中マドが開く。
#[test]
fn gradient_fill_honours_the_fill_rule() {
    let donut = PathSource::Bezier(vec![
        Contour::closed([
            Point { x: -80.0, y: -80.0 },
            Point { x: 80.0, y: -80.0 },
            Point { x: 80.0, y: 80.0 },
            Point { x: -80.0, y: 80.0 },
        ]),
        Contour::closed([
            Point { x: -30.0, y: -30.0 },
            Point { x: 30.0, y: -30.0 },
            Point { x: 30.0, y: 30.0 },
            Point { x: -30.0, y: 30.0 },
        ]),
    ]);
    let of = |rule| {
        let g = two_stop(
            GradientType::Linear,
            Point { x: -80.0, y: 0.0 },
            Point { x: 80.0, y: 0.0 },
        );
        render(
            &Shape {
                fill: Some(Fill {
                    brush: Brush::Gradient(g),
                    rule,
                    ..Fill::default()
                }),
                ..Shape::new(donut.clone())
            },
            &canvas(),
        )
        .unwrap()
    };
    assert_ne!(
        alpha_at(&of(FillRule::NonZero), 100, 100),
        0,
        "nonzero で中央が抜けている"
    );
    assert_eq!(
        alpha_at(&of(FillRule::EvenOdd), 100, 100),
        0,
        "evenodd で中マドが開いていない"
    );
}

/// `gradient-stroke.ty`。Brush は fill/stroke に直交する — 線にも同じ塗りが乗る。
#[test]
fn gradient_stroke_paints_the_line_with_the_gradient() {
    let g = two_stop(
        GradientType::Linear,
        Point { x: -80.0, y: 0.0 },
        Point { x: 80.0, y: 0.0 },
    );
    let out = render(
        &Shape {
            stroke: Some(Stroke {
                brush: Brush::Gradient(g),
                width: 12.0,
                ..Stroke::default()
            }),
            ..Shape::new(horizontal_line())
        },
        &canvas(),
    )
    .unwrap();
    let left = rgb_at(&out, 30, 100);
    let right = rgb_at(&out, 170, 100);
    assert!(left.0 > left.2, "線の左端が赤寄りでない: {left:?}");
    assert!(right.2 > right.0, "線の右端が青寄りでない: {right:?}");
}

/// `shape-style.o` は gradient にも効く(不透明度の正本は1つ)。
#[test]
fn gradient_opacity_still_comes_from_shape_style() {
    let of = |opacity| {
        let g = two_stop(
            GradientType::Linear,
            Point { x: -50.0, y: 0.0 },
            Point { x: 50.0, y: 0.0 },
        );
        render(
            &Shape {
                fill: Some(Fill {
                    brush: Brush::Gradient(g),
                    opacity,
                    ..Fill::default()
                }),
                ..Shape::new(square())
            },
            &canvas(),
        )
        .unwrap()
    };
    let full = alpha_at(&of(1.0), 100, 100);
    let half = alpha_at(&of(0.5), 100, 100);
    assert!(
        full > half && half > 0,
        "gradient に opacity が効いていない"
    );
}

// ---------------------------------------------------------------------------
// stroke-dash.v — pattern の1要素の長さ
// ---------------------------------------------------------------------------

/// `stroke-dash.v`(Length)。長さを変えれば破線の刻みが変わる。
#[test]
fn dash_lengths_change_the_pieces() {
    let of = |pattern: Vec<f64>| {
        render(
            &Shape {
                stroke: Some(Stroke {
                    brush: Brush::Solid(Rgb::BLACK),
                    width: 4.0,
                    dash: Some(motolii_vector::Dash {
                        pattern,
                        offset: 0.0,
                    }),
                    ..Stroke::default()
                }),
                ..Shape::new(horizontal_line())
            },
            &canvas(),
        )
        .unwrap()
    };
    let coarse = of(vec![30.0, 10.0]);
    let fine = of(vec![6.0, 10.0]);
    assert!(
        painted(&coarse) > painted(&fine),
        "dash の長さが効いていない"
    );
}

// ---------------------------------------------------------------------------
// twist(shape-3)— `a`(角度)/`c`(中心)
// ---------------------------------------------------------------------------

/// 半径の違う頂点を持つ形。twist は**中心ほど大きく**回すので、
/// 全頂点が同じ半径にある正方形では観測できない。
fn star() -> PathSource {
    polystar(5.0, 80.0, 30.0, StarType::Star)
}

/// `twist.a`(Angle)。外縁は動かず、内側だけがねじれる。
#[test]
fn twist_angle_turns_the_inside_more_than_the_rim() {
    let straight = filled(star(), vec![]);
    let twisted = filled(
        star(),
        vec![ShapeOp::new(OpKind::Twist {
            angle: 60.0,
            center: Point::ZERO,
        })],
    );
    assert!(painted(&twisted) > 0, "twist で全部消えた");
    assert!(
        straight.premultiplied_rgba8 != twisted.premultiplied_rgba8,
        "angle が画に出ていない"
    );
    assert_eq!(
        filled(
            star(),
            vec![ShapeOp::new(OpKind::Twist {
                angle: 0.0,
                center: Point::ZERO,
            })]
        )
        .premultiplied_rgba8,
        straight.premultiplied_rgba8,
        "angle=0 が恒等になっていない"
    );
}

/// `twist.c`(Center)。ねじりの中心を動かせば別の画になる。
#[test]
fn twist_centre_moves_the_pivot() {
    let of = |center| {
        filled(
            star(),
            vec![ShapeOp::new(OpKind::Twist {
                angle: 60.0,
                center,
            })],
        )
    };
    let origin = of(Point::ZERO);
    let shifted = of(Point { x: 40.0, y: 0.0 });
    assert!(
        origin.premultiplied_rgba8 != shifted.premultiplied_rgba8,
        "center を動かしても画が変わらない"
    );
}
