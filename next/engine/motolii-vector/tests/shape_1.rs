//! `shape-1` の受入条件。**先に落ちる試験として書いた**(実装より前)。
//!
//! 見るのは全部**出た画素**であって内部状態ではない。演算子の実装を差し替えても
//! 「線が伸びる」「コピーが放射状に並ぶ」「角が丸まる」「中マドが開く」が
//! 観測できる限り緑であってほしいため。

use motolii_vector::{
    render, Brush, Canvas, Composite, Contour, Fill, FillRule, LineCap, LineJoin, OpKind,
    PathSource, Point, Raster, RepeaterTransform, Rgb, Shape, ShapeOp, Stroke, TrimMultiple,
    Vertex,
};

// ---------------------------------------------------------------------------
// 器具 — 画素を数える/拾うだけ。ここに幾何を持ち込むと試験が実装の写しになる。
// ---------------------------------------------------------------------------

/// alpha が 0 でない画素の数。
fn painted(r: &Raster) -> usize {
    r.premultiplied_rgba8
        .chunks_exact(4)
        .filter(|p| p[3] != 0)
        .count()
}

fn alpha_at(r: &Raster, x: u32, y: u32) -> u8 {
    let i = ((y * r.width + x) * 4 + 3) as usize;
    r.premultiplied_rgba8[i]
}

fn canvas() -> Canvas {
    Canvas::centered(200, 200)
}

/// 原点中央の 100x100 正方形(角丸・trim の土台)。
fn square() -> PathSource {
    PathSource::Rectangle {
        size: Point { x: 100.0, y: 100.0 },
    }
}

fn black_stroke(width: f64) -> Stroke {
    Stroke {
        brush: Brush::Solid(Rgb::BLACK),
        width,
        ..Stroke::default()
    }
}

fn black_fill() -> Fill {
    Fill {
        brush: Brush::Solid(Rgb::BLACK),
        ..Fill::default()
    }
}

fn trim(start: f64, end: f64) -> ShapeOp {
    ShapeOp::new(OpKind::TrimPath {
        start,
        end,
        offset: 0.0,
        multiple: TrimMultiple::Simultaneously,
    })
}

// ---------------------------------------------------------------------------
// trim-path — 0→1 で線の長さが単調に増える
// ---------------------------------------------------------------------------

#[test]
fn trim_path_draws_the_line_progressively() {
    let mut previous = 0usize;
    let mut counts = Vec::new();
    for step in 0..=10 {
        let end = step as f64 / 10.0;
        let shape = Shape {
            ops: vec![trim(0.0, end)],
            stroke: Some(black_stroke(4.0)),
            ..Shape::new(square())
        };
        let n = painted(&render(&shape, &canvas()).unwrap());
        assert!(
            n >= previous,
            "trim end={end} で描かれた画素が減った({previous} → {n})。\
             0→1 は単調に増えなければならない"
        );
        previous = n;
        counts.push(n);
    }
    assert_eq!(counts[0], 0, "end=0 は何も描かれない");
    assert!(
        counts[10] > counts[5] && counts[5] > counts[1],
        "端だけでなく途中も伸びていること: {counts:?}"
    );
}

/// `trim-path.o`(Offset)= 切り出し窓の回転。同じ長さのまま位置が動く。
#[test]
fn trim_path_offset_rotates_the_window_without_changing_length() {
    let make = |offset: f64| {
        let shape = Shape {
            ops: vec![ShapeOp::new(OpKind::TrimPath {
                start: 0.0,
                end: 0.25,
                offset,
                multiple: TrimMultiple::Simultaneously,
            })],
            stroke: Some(black_stroke(4.0)),
            ..Shape::new(square())
        };
        render(&shape, &canvas()).unwrap()
    };
    let a = make(0.0);
    let b = make(0.25);
    let (na, nb) = (painted(&a), painted(&b));
    assert!(na > 0 && nb > 0);
    // 正方形を 1/4 ずつ切るので長さは同じ。画素数は丸めぶんだけ揺れる。
    assert!(
        (na as i64 - nb as i64).abs() < (na as i64) / 5,
        "offset で長さが変わってはいけない: {na} vs {nb}"
    );
    assert!(
        a.premultiplied_rgba8 != b.premultiplied_rgba8,
        "offset で窓の位置が動いていない"
    );
}

/// `trim-path.m`(Multiple)。輪郭2本を「同時に」切るか「順番に」切るかで画が違う。
#[test]
fn trim_multiple_shapes_distinguishes_simultaneous_from_individual() {
    let two_lines = PathSource::Bezier(vec![
        Contour::open([Point { x: -80.0, y: -40.0 }, Point { x: -20.0, y: -40.0 }]),
        Contour::open([Point { x: 20.0, y: 40.0 }, Point { x: 80.0, y: 40.0 }]),
    ]);
    let make = |multiple| {
        let shape = Shape {
            ops: vec![ShapeOp::new(OpKind::TrimPath {
                start: 0.0,
                end: 0.5,
                offset: 0.0,
                multiple,
            })],
            stroke: Some(black_stroke(4.0)),
            ..Shape::new(two_lines.clone())
        };
        render(&shape, &canvas()).unwrap()
    };
    let simultaneous = make(TrimMultiple::Simultaneously);
    let individually = make(TrimMultiple::Individually);
    assert!(
        simultaneous.premultiplied_rgba8 != individually.premultiplied_rgba8,
        "Simultaneously と Individually が同じ画を出している"
    );
    // Individually = 連結した全長の前半 → 2本目には何も乗らない。
    let second_line_row = 140; // y=+40 が中心 100 から +40
    let painted_on_second: usize = (0..individually.width)
        .filter(|x| alpha_at(&individually, *x, second_line_row) != 0)
        .count();
    assert_eq!(
        painted_on_second, 0,
        "Individually は連結長の前半だけなので 2本目の輪郭は空のはず"
    );
    let simultaneous_second: usize = (0..simultaneous.width)
        .filter(|x| alpha_at(&simultaneous, *x, second_line_row) != 0)
        .count();
    assert!(
        simultaneous_second > 0,
        "Simultaneously は輪郭ごとに同じ窓で切るので 2本目にも半分乗る"
    );
}

// ---------------------------------------------------------------------------
// repeater — コピーが増え、tr の anchor で放射状に並ぶ
// ---------------------------------------------------------------------------

fn repeater(copies: f64, transform: RepeaterTransform) -> ShapeOp {
    ShapeOp::new(OpKind::Repeater {
        copies,
        offset: 0.0,
        transform,
        composite: Composite::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    })
}

#[test]
fn repeater_adds_copies() {
    let dot = PathSource::Ellipse {
        size: Point { x: 10.0, y: 10.0 },
    };
    let step = RepeaterTransform {
        position: Point { x: 20.0, y: 0.0 },
        ..RepeaterTransform::IDENTITY
    };
    let one = painted(
        &render(
            &Shape {
                ops: vec![repeater(1.0, step)],
                fill: Some(black_fill()),
                ..Shape::new(dot.clone())
            },
            &canvas(),
        )
        .unwrap(),
    );
    let four = painted(
        &render(
            &Shape {
                ops: vec![repeater(4.0, step)],
                fill: Some(black_fill()),
                ..Shape::new(dot)
            },
            &canvas(),
        )
        .unwrap(),
    );
    assert!(one > 0, "コピー1個で何も描かれていない");
    assert!(
        four > one * 3,
        "copies=4 で 4個ぶんに増えていない({one} → {four})"
    );
}

/// `repeater.tr` の **anchor** が放射状配置を成立させる。
///
/// `anchor == position` にすると、行列は「anchor まわりの回転」になる。
/// よってコピーは **anchor を中心とする環**に並ぶ — 図形源が原点中央にしか
/// 作れない(裁定74)以上、環の中心を決めているのは anchor 1つである。
#[test]
fn repeater_anchor_lays_copies_out_radially() {
    let dot = PathSource::Ellipse {
        size: Point { x: 12.0, y: 12.0 },
    };
    let pivot = Point { x: 0.0, y: -40.0 };
    let radial = RepeaterTransform {
        anchor: pivot,
        position: pivot,
        rotation: 90.0,
        ..RepeaterTransform::IDENTITY
    };
    let shape = Shape {
        ops: vec![repeater(4.0, radial)],
        fill: Some(black_fill()),
        ..Shape::new(dot)
    };
    let out = render(&shape, &canvas()).unwrap();

    // canvas 中央 = 局所原点 (100,100)。環の中心は anchor = 局所 (0,-40) = 画素 (100,60)、
    // 半径は原点との距離 40。90度ずつなので4点が東西南北に載る。
    for (x, y) in [(100u32, 100u32), (60, 60), (100, 20), (140, 60)] {
        assert_ne!(
            alpha_at(&out, x, y),
            0,
            "放射状の位置 ({x},{y}) にコピーが載っていない"
        );
    }
    // 環の中心は空 = 一直線に並んだのでも、1箇所に重なったのでもない。
    assert_eq!(
        alpha_at(&out, 100, 60),
        0,
        "環の中心が埋まっている = 放射状になっていない"
    );
}

/// `repeater-transform.so`/`eo` は幾何ではなく重み。最後のコピーが薄くなる。
#[test]
fn repeater_start_and_end_opacity_fade_the_copies() {
    let dot = PathSource::Ellipse {
        size: Point { x: 20.0, y: 20.0 },
    };
    let shape = Shape {
        ops: vec![ShapeOp::new(OpKind::Repeater {
            copies: 3.0,
            offset: 0.0,
            transform: RepeaterTransform {
                position: Point { x: 60.0, y: 0.0 },
                ..RepeaterTransform::IDENTITY
            },
            composite: Composite::Above,
            start_opacity: 1.0,
            end_opacity: 0.2,
        })],
        fill: Some(black_fill()),
        ..Shape::new(dot)
    };
    let out = render(&shape, &canvas()).unwrap();
    let first = alpha_at(&out, 100, 100);
    let last = alpha_at(&out, 220.min(out.width - 1), 100);
    assert_ne!(first, 0, "1個目が描かれていない");
    let last = if last == 0 {
        alpha_at(&out, 199, 100)
    } else {
        last
    };
    assert!(
        last < first,
        "end_opacity=0.2 なのに最後のコピーが薄くなっていない({first} → {last})"
    );
}

/// `repeater.o`(Offset)= 開始インデックスのずらし。0 と 1 で位置が動く。
#[test]
fn repeater_offset_shifts_the_starting_index() {
    let dot = PathSource::Ellipse {
        size: Point { x: 12.0, y: 12.0 },
    };
    let make = |offset: f64| {
        let shape = Shape {
            ops: vec![ShapeOp::new(OpKind::Repeater {
                copies: 2.0,
                offset,
                transform: RepeaterTransform {
                    position: Point { x: 40.0, y: 0.0 },
                    ..RepeaterTransform::IDENTITY
                },
                composite: Composite::Above,
                start_opacity: 1.0,
                end_opacity: 1.0,
            })],
            fill: Some(black_fill()),
            ..Shape::new(dot.clone())
        };
        render(&shape, &canvas()).unwrap()
    };
    assert_ne!(
        alpha_at(&make(0.0), 100, 100),
        0,
        "offset=0 は原点から始まる"
    );
    assert_eq!(
        alpha_at(&make(1.0), 100, 100),
        0,
        "offset=1 なら 1個目が 1段ぶんずれて原点は空く"
    );
}

/// `repeater.m`(Composite)= 重ね順。
///
/// **観測には fill と stroke の色差が要る**。層が持つ fill/stroke は各1つなので
/// (裁定73)、コピー同士は同じ色になり、source-over は同色の重ねに対して
/// 順序不変(`a₁+a₂-a₁a₂` は対称)。つまり *不透明度差だけでは順序が画に出ない*。
/// 出るのは「手前のコピーの塗りが、奥のコピーの線を隠す」経路である。
#[test]
fn repeater_composite_changes_the_stacking_order() {
    let big = PathSource::Rectangle {
        size: Point { x: 80.0, y: 80.0 },
    };
    let make = |composite| {
        let shape = Shape {
            ops: vec![ShapeOp::new(OpKind::Repeater {
                copies: 2.0,
                offset: 0.0,
                transform: RepeaterTransform {
                    position: Point { x: 30.0, y: 0.0 },
                    ..RepeaterTransform::IDENTITY
                },
                composite,
                start_opacity: 1.0,
                end_opacity: 1.0,
            })],
            fill: Some(Fill {
                brush: Brush::Solid(Rgb {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                }),
                ..Fill::default()
            }),
            stroke: Some(Stroke {
                brush: Brush::Solid(Rgb {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                }),
                width: 10.0,
                ..Stroke::default()
            }),
            ..Shape::new(big.clone())
        };
        render(&shape, &canvas()).unwrap()
    };
    let above = make(Composite::Above);
    let below = make(Composite::Below);
    assert!(
        above.premultiplied_rgba8 != below.premultiplied_rgba8,
        "Above と Below が同じ画を出している = 重ね順が効いていない"
    );
    // 重なりの中(1つ目の右辺 = 局所 x=+40 → 画素 140)で、隠す側の色が入れ替わる。
    let pick = |r: &Raster, x: u32| {
        let i = ((100 * r.width + x) * 4) as usize;
        r.premultiplied_rgba8[i..i + 4].to_vec()
    };
    assert_ne!(pick(&above, 140), pick(&below, 140), "重なりの色が同じ");
}

// ---------------------------------------------------------------------------
// rounded-corners — 角が丸まる(頂点が増え、角の画素が背景になる)
// ---------------------------------------------------------------------------

#[test]
fn rounded_corners_round_the_corner_pixels() {
    let sharp = Shape {
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let round = Shape {
        ops: vec![ShapeOp::new(OpKind::RoundedCorners { radius: 30.0 })],
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let sharp = render(&sharp, &canvas()).unwrap();
    let round = render(&round, &canvas()).unwrap();

    // 100x100 の角は canvas 中央 (100,100) から ±50 → (51,51) は角のすぐ内側。
    assert_ne!(alpha_at(&sharp, 52, 52), 0, "角丸なしで角が塗られていない");
    assert_eq!(
        alpha_at(&round, 52, 52),
        0,
        "radius=30 なのに角の画素が残っている"
    );
    // 辺の中央は両方とも塗られたまま = 図形ごと消えたのではない。
    assert_ne!(alpha_at(&round, 100, 52), 0, "辺の中央まで削れている");
    assert!(
        painted(&round) < painted(&sharp),
        "角丸で面積が減っていない"
    );
}

// 「頂点数が増える」は画素から観測できないので、crate 内の単体試験
// (`src/lib.rs` の `geometry_tests`)に置いてある。公開口を1本に保つため。

// ---------------------------------------------------------------------------
// fill-rule — 中マドが開く(裁定74: パスブーリアンの代わり)
// ---------------------------------------------------------------------------

/// 同じ巻き方向の入れ子2輪郭。nonzero は塗り潰し、evenodd は内側が抜ける。
fn donut() -> PathSource {
    let ring = |h: f64| {
        Contour::closed([
            Point { x: -h, y: -h },
            Point { x: h, y: -h },
            Point { x: h, y: h },
            Point { x: -h, y: h },
        ])
    };
    PathSource::Bezier(vec![ring(60.0), ring(25.0)])
}

#[test]
fn fill_rule_opens_the_hole() {
    let make = |rule| {
        let shape = Shape {
            fill: Some(Fill {
                brush: Brush::Solid(Rgb::BLACK),
                rule,
                ..Fill::default()
            }),
            ..Shape::new(donut())
        };
        render(&shape, &canvas()).unwrap()
    };
    let nonzero = make(FillRule::NonZero);
    let evenodd = make(FillRule::EvenOdd);

    assert_ne!(
        alpha_at(&nonzero, 100, 100),
        0,
        "nonzero で中心が抜けている"
    );
    assert_eq!(
        alpha_at(&evenodd, 100, 100),
        0,
        "evenodd なのに中マドが開いていない"
    );
    // 外側のリング部分はどちらも塗られている。
    assert_ne!(alpha_at(&nonzero, 100, 60), 0);
    assert_ne!(alpha_at(&evenodd, 100, 60), 0);
    assert!(
        painted(&evenodd) < painted(&nonzero),
        "evenodd の方が面積が小さいはず"
    );
}

// ---------------------------------------------------------------------------
// 決定性 — 同じ記述から2回描いて byte 一致(裁定15 の隣)
// ---------------------------------------------------------------------------

#[test]
fn the_same_description_renders_byte_identical_twice() {
    let shape = Shape {
        ops: vec![
            ShapeOp::new(OpKind::RoundedCorners { radius: 12.0 }),
            trim(0.1, 0.8),
            ShapeOp::new(OpKind::Repeater {
                copies: 5.0,
                offset: 0.0,
                transform: RepeaterTransform {
                    anchor: Point { x: 0.0, y: -40.0 },
                    position: Point { x: 0.0, y: -40.0 },
                    rotation: 37.0,
                    scale: Point { x: 0.9, y: 0.9 },
                },
                composite: Composite::Below,
                start_opacity: 1.0,
                end_opacity: 0.25,
            }),
        ],
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 0.2,
                g: 0.6,
                b: 0.9,
            }),
            rule: FillRule::EvenOdd,
            opacity: 0.75,
            hidden: false,
        }),
        stroke: Some(Stroke {
            brush: Brush::Solid(Rgb::BLACK),
            width: 3.5,
            cap: LineCap::Round,
            join: LineJoin::Bevel,
            miter_limit: 2.0,
            dash: None,
            opacity: 0.9,
            hidden: false,
        }),
        ..Shape::new(square())
    };
    let a = render(&shape, &canvas()).unwrap();
    let b = render(&shape, &canvas()).unwrap();
    assert_eq!(
        a.premultiplied_rgba8, b.premultiplied_rgba8,
        "同じ記述から2回描いて byte 一致しない = 決定的でない"
    );
    assert!(painted(&a) > 0, "そもそも何も描かれていない");
}

// ---------------------------------------------------------------------------
// パス源・スタイル語彙
// ---------------------------------------------------------------------------

#[test]
fn each_path_source_paints_something_different() {
    let size = Point { x: 120.0, y: 60.0 };
    let of = |source| {
        render(
            &Shape {
                fill: Some(black_fill()),
                ..Shape::new(source)
            },
            &canvas(),
        )
        .unwrap()
    };
    let rect = of(PathSource::Rectangle { size });
    let ellipse = of(PathSource::Ellipse { size });
    let bezier = of(PathSource::Bezier(vec![Contour::closed([
        Point { x: -60.0, y: -30.0 },
        Point { x: 60.0, y: -30.0 },
        Point { x: 0.0, y: 30.0 },
    ])]));
    assert!(painted(&rect) > painted(&ellipse), "楕円は矩形より狭いはず");
    assert!(painted(&ellipse) > 0 && painted(&bezier) > 0);
    // 楕円の角は空き、矩形の角は埋まる = 同じ `s` から違うパスが出ている。
    assert_ne!(alpha_at(&rect, 42, 72), 0);
    assert_eq!(alpha_at(&ellipse, 42, 72), 0);
}

/// `path.ks` は in/out tangent を持つ(Lottie `v`/`i`/`o` と同型)。
#[test]
fn bezier_source_honours_tangents() {
    let straight = PathSource::Bezier(vec![Contour::open([
        Point { x: -70.0, y: 0.0 },
        Point { x: 70.0, y: 0.0 },
    ])]);
    let curved = PathSource::Bezier(vec![Contour {
        closed: false,
        vertices: vec![
            Vertex {
                point: Point { x: -70.0, y: 0.0 },
                in_tangent: Point::ZERO,
                out_tangent: Point { x: 0.0, y: -80.0 },
            },
            Vertex {
                point: Point { x: 70.0, y: 0.0 },
                in_tangent: Point { x: 0.0, y: -80.0 },
                out_tangent: Point::ZERO,
            },
        ],
    }]);
    let of = |source| {
        render(
            &Shape {
                stroke: Some(black_stroke(4.0)),
                ..Shape::new(source)
            },
            &canvas(),
        )
        .unwrap()
    };
    let straight = of(straight);
    let curved = of(curved);
    // 中央の列で、上から見て最初に塗られた行。直線は中央のまま、曲線は上へ膨らむ。
    let topmost = |r: &Raster| (0..r.height).find(|y| alpha_at(r, 100, *y) != 0);
    let s = topmost(&straight).expect("直線が描かれていない");
    let c = topmost(&curved).expect("曲線が描かれていない");
    assert!((96..=104).contains(&s), "直線が中央から外れている: {s}");
    assert!(
        c < s - 40,
        "out_tangent が効いていない(直線 {s} 行に対し曲線 {c} 行)"
    );
}

#[test]
fn stroke_width_and_color_reach_the_pixels() {
    let thin = render(
        &Shape {
            stroke: Some(black_stroke(2.0)),
            ..Shape::new(square())
        },
        &canvas(),
    )
    .unwrap();
    let thick = render(
        &Shape {
            stroke: Some(black_stroke(12.0)),
            ..Shape::new(square())
        },
        &canvas(),
    )
    .unwrap();
    assert!(painted(&thick) > painted(&thin) * 3, "線幅が効いていない");

    let red = render(
        &Shape {
            stroke: Some(Stroke {
                brush: Brush::Solid(Rgb {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                }),
                width: 12.0,
                ..Stroke::default()
            }),
            ..Shape::new(square())
        },
        &canvas(),
    )
    .unwrap();
    // 辺の真ん中の画素は不透明な赤(premultiplied でも a=255 なら値はそのまま)。
    let i = ((100 * red.width + 50) * 4) as usize;
    assert_eq!(&red.premultiplied_rgba8[i..i + 4], &[255, 0, 0, 255]);
}

/// `base-stroke.lc`(Line Cap)。開いた線の端で butt / square の長さが変わる。
#[test]
fn line_cap_extends_the_open_ends() {
    let line = PathSource::Bezier(vec![Contour::open([
        Point { x: -40.0, y: 0.0 },
        Point { x: 40.0, y: 0.0 },
    ])]);
    let of = |cap| {
        render(
            &Shape {
                stroke: Some(Stroke {
                    brush: Brush::Solid(Rgb::BLACK),
                    width: 16.0,
                    cap,
                    ..Stroke::default()
                }),
                ..Shape::new(line.clone())
            },
            &canvas(),
        )
        .unwrap()
    };
    let butt = of(LineCap::Butt);
    let square_cap = of(LineCap::Square);
    let round_cap = of(LineCap::Round);
    assert!(
        painted(&square_cap) > painted(&butt),
        "square cap が伸びていない"
    );
    assert!(
        painted(&round_cap) > painted(&butt),
        "round cap が伸びていない"
    );
    assert!(
        painted(&square_cap) > painted(&round_cap),
        "square は round より広い"
    );
}

/// `base-stroke.lj` と `ml2`(Line Join / Miter Limit)。
#[test]
fn line_join_and_miter_limit_change_the_corner() {
    let spike = PathSource::Bezier(vec![Contour::open([
        Point { x: -70.0, y: 40.0 },
        Point { x: 0.0, y: -40.0 },
        Point { x: 70.0, y: 40.0 },
    ])]);
    let of = |join, miter_limit| {
        render(
            &Shape {
                stroke: Some(Stroke {
                    brush: Brush::Solid(Rgb::BLACK),
                    width: 14.0,
                    join,
                    miter_limit,
                    ..Stroke::default()
                }),
                ..Shape::new(spike.clone())
            },
            &canvas(),
        )
        .unwrap()
    };
    let miter = of(LineJoin::Miter, 10.0);
    let bevel = of(LineJoin::Bevel, 10.0);
    let round = of(LineJoin::Round, 10.0);
    assert!(painted(&miter) > painted(&bevel), "miter が尖っていない");
    assert!(
        round.premultiplied_rgba8 != bevel.premultiplied_rgba8,
        "round join が bevel と同じ画"
    );
    // miter_limit を下げると bevel へ縮退する。
    let clipped = of(LineJoin::Miter, 1.0);
    assert_eq!(
        clipped.premultiplied_rgba8, bevel.premultiplied_rgba8,
        "miter_limit を割ったのに bevel へ落ちていない"
    );
}

/// `base-stroke.d`(Dashes)。上流 `StrokeDash` をそのまま通す。
#[test]
fn dashes_break_the_line_into_pieces() {
    let line = PathSource::Bezier(vec![Contour::open([
        Point { x: -80.0, y: 0.0 },
        Point { x: 80.0, y: 0.0 },
    ])]);
    let of = |dash| {
        render(
            &Shape {
                stroke: Some(Stroke {
                    brush: Brush::Solid(Rgb::BLACK),
                    width: 6.0,
                    dash,
                    ..Stroke::default()
                }),
                ..Shape::new(line.clone())
            },
            &canvas(),
        )
        .unwrap()
    };
    let solid = of(None);
    let dashed = of(Some(motolii_vector::Dash {
        pattern: vec![10.0, 10.0],
        offset: 0.0,
    }));
    assert!(painted(&dashed) < painted(&solid), "破線になっていない");
    assert!(painted(&dashed) > 0, "破線で全部消えた");
}

/// `shape-style.o`(Opacity)は fill / stroke それぞれが持つ。
#[test]
fn shape_style_opacity_scales_the_alpha() {
    let of = |opacity| {
        render(
            &Shape {
                fill: Some(Fill {
                    brush: Brush::Solid(Rgb::BLACK),
                    opacity,
                    ..Fill::default()
                }),
                ..Shape::new(square())
            },
            &canvas(),
        )
        .unwrap()
    };
    assert_eq!(alpha_at(&of(1.0), 100, 100), 255);
    let half = alpha_at(&of(0.5), 100, 100);
    assert!(
        (120..=135).contains(&half),
        "opacity=0.5 の alpha が {half}"
    );
    assert_eq!(alpha_at(&of(0.0), 100, 100), 0);
}

/// `graphic-element.hd`(Hidden)は演算子1段と fill/stroke の有効/無効。
#[test]
fn hidden_disables_one_stack_entry_without_removing_it() {
    let with_round = Shape {
        ops: vec![ShapeOp::new(OpKind::RoundedCorners { radius: 30.0 })],
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let mut disabled = with_round.clone();
    disabled.ops[0].hidden = true;
    let plain = Shape {
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let c = canvas();
    assert_eq!(
        render(&disabled, &c).unwrap().premultiplied_rgba8,
        render(&plain, &c).unwrap().premultiplied_rgba8,
        "hidden な演算子が効いてしまっている"
    );
    assert!(
        render(&with_round, &c).unwrap().premultiplied_rgba8
            != render(&plain, &c).unwrap().premultiplied_rgba8
    );

    let hidden_fill = Shape {
        fill: Some(Fill {
            hidden: true,
            ..black_fill()
        }),
        ..Shape::new(square())
    };
    assert_eq!(
        painted(&render(&hidden_fill, &c).unwrap()),
        0,
        "hidden な fill が描かれた"
    );
}

/// 演算子スタックは**順序付き**(裁定73)。並べ替えると結果が変わる。
#[test]
fn the_operator_stack_is_ordered() {
    let round = ShapeOp::new(OpKind::RoundedCorners { radius: 25.0 });
    let cut = trim(0.0, 0.5);
    let c = canvas();
    let a = render(
        &Shape {
            ops: vec![round.clone(), cut.clone()],
            stroke: Some(black_stroke(4.0)),
            ..Shape::new(square())
        },
        &c,
    )
    .unwrap();
    let b = render(
        &Shape {
            ops: vec![cut, round],
            stroke: Some(black_stroke(4.0)),
            ..Shape::new(square())
        },
        &c,
    )
    .unwrap();
    assert!(
        a.premultiplied_rgba8 != b.premultiplied_rgba8,
        "順序を入れ替えても同じ画 = スタックが順序を持っていない"
    );
}

#[test]
fn a_zero_sized_canvas_is_an_error_not_an_empty_picture() {
    let err = render(
        &Shape {
            fill: Some(black_fill()),
            ..Shape::new(square())
        },
        &Canvas::centered(0, 0),
    );
    assert!(err.is_err(), "0x0 の canvas が静かに空の画を返した");
}

#[test]
fn a_shape_without_style_paints_nothing() {
    let out = render(&Shape::new(square()), &canvas()).unwrap();
    assert_eq!(painted(&out), 0);
    assert_eq!(out.premultiplied_rgba8.len(), 200 * 200 * 4);
}

/// 出力は **premultiplied** RGBA8。half-alpha の白は各色成分も半分になる。
#[test]
fn the_output_is_premultiplied() {
    let out = render(
        &Shape {
            fill: Some(Fill {
                brush: Brush::Solid(Rgb {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                }),
                opacity: 0.5,
                ..Fill::default()
            }),
            ..Shape::new(square())
        },
        &canvas(),
    )
    .unwrap();
    let i = ((100 * out.width + 100) * 4) as usize;
    let px = &out.premultiplied_rgba8[i..i + 4];
    assert_eq!(
        px[3], px[0],
        "premultiplied なら白の R は alpha と同じ値: {px:?}"
    );
    assert!(px[0] < 255 && px[0] > 0, "{px:?}");
}
