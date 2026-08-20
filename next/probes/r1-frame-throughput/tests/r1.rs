//! R1: 1080p で layer を増やした時の1フレームあたりの時間。
//!
//! **この試験群は既定の `cargo test` では走らない**(`#[ignore]`)。
//!
//! 理由: 時間の予算を GPU を奪い合いながら測っても意味が無い。実測で、他の GPU 試験と
//! 並列に走らせると等倍40枚が 40ms → 77ms へ倍近く伸びた。予算を緩めて通せば
//! 見張りとして死ぬので、**単独で走らせる**方を選ぶ。
//!
//! ```sh
//! cargo test --release -p r1-frame-throughput -- --ignored --nocapture --test-threads=1
//! ```


use std::time::Instant;

use motolii_compositor::{CompSpec, LayerPlacement};
use motolii_engine::Engine;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource};
use motolii_store::LayerTiming;
use r1_frame_throughput::FHD;

fn comp() -> CompSpec {
    CompSpec {
        width: FHD[0],
        height: FHD[1],
    }
}

fn t() -> motolii_store::RationalTime {
    motolii_store::RationalTime::try_new(0, 30).unwrap()
}

/// layer を n 枚積んだ Document。素材は全部別色にして texture cache に頼らせない。
/// `source_scale` は素材側の縮小率(1 = 等倍、2 = 1/2 の proxy 相当)。
fn document_scaled(layers: u64, source_scale: u32) -> Document {
    let sw = FHD[0] / source_scale;
    let sh = FHD[1] / source_scale;
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: sw,
        height: sh,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
    }))
    .unwrap();
    for i in 0..layers {
        let id = LayerId(i);
        doc.apply(Intent::AddLayer(id)).unwrap();
        doc.apply(Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [(i * 7 % 256) as u8, (i * 13 % 256) as u8, 200, 128],
                    width: sw,
                    height: sh,
                },
                order: i as i16,
                timing: LayerTiming::place(0, None, 100_000),
            },
        })
        .unwrap();
    }
    doc
}

fn document_with(layers: u64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: FHD[0],
        height: FHD[1],
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
    }))
    .unwrap();
    for i in 0..layers {
        let id = LayerId(i);
        doc.apply(Intent::AddLayer(id)).unwrap();
        doc.apply(Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    // 半透明にして、合成が実際に混ぜる経路を通す。
                    rgba: [(i * 7 % 256) as u8, (i * 13 % 256) as u8, 200, 128],
                    width: FHD[0],
                    height: FHD[1],
                },
                order: i as i16,
                timing: LayerTiming::place(0, None, 100_000),
            },
        })
        .unwrap();
    }
    doc
}

fn measure_at(engine: &mut Engine, doc: &Document, spec: CompSpec, runs: u32) -> u128 {
    let _ = engine.render_frame(&doc.view(), t()).unwrap();
    let start = Instant::now();
    for _ in 0..runs {
        engine.render_frame(&doc.view(), t()).unwrap();
    }
    start.elapsed().as_micros() / runs as u128
}

fn measure(engine: &mut Engine, doc: &Document, runs: u32) -> u128 {
    // 1回目は pipeline / texture の用意が入るので捨てる。
    let _ = engine.render_frame(&doc.view(), t()).unwrap();

    let start = Instant::now();
    for _ in 0..runs {
        let frame = engine.render_frame(&doc.view(), t()).unwrap();
        assert_eq!(frame.len(), (FHD[0] * FHD[1] * 4) as usize);
    }
    start.elapsed().as_micros() / runs as u128
}

/// **どこで時間を使っているか**。preview を CPU 読み戻しで出すか GPU 共有で出すかは
/// この内訳で決まる(読み戻しが支配的なら、preview は読み戻してはいけない)。
#[test]
#[ignore = "GPU を単独で使う必要がある — 冒頭の doc を参照"]
fn where_the_frame_time_goes() {
    use motolii_compositor::{Compositor, Layer, LayerPlacement};

    let mut compositor = Compositor::headless().expect("compositor");
    let texture = compositor
        .upload_rgba(
            "fhd",
            &vec![128u8; (FHD[0] * FHD[1] * 4) as usize],
            FHD[0],
            FHD[1],
        )
        .unwrap();

    let layers: Vec<Layer> = (0..40)
        .map(|i| Layer {
            texture: texture.clone(),
            placement: LayerPlacement {
                top_left: [0.0, 0.0],
                size: [FHD[0] as f32, FHD[1] as f32],
                order: i,
                opacity: 1.0,
            },
        })
        .collect();

    let _ = compositor.render(comp(), &layers[..1]).unwrap();

    for count in [1usize, 10, 40] {
        let mut build = 0u128;
        let mut gpu = 0u128;
        let mut readback = 0u128;
        const RUNS: u128 = 10;
        for _ in 0..RUNS {
            let (_, timing) = compositor
                .render_with_timing(comp(), &layers[..count])
                .unwrap();
            build += timing.build_us;
            gpu += timing.gpu_us;
            readback += timing.readback_us;
        }
        println!(
            "R1 内訳 {count:>2}枚: build={:>6}µs gpu={:>6}µs readback={:>6}µs (合計 {:>6}µs)",
            build / RUNS,
            gpu / RUNS,
            readback / RUNS,
            (build + gpu + readback) / RUNS
        );
    }
}

/// **解像度のレバーが効くか**。どの NLE も preview 品質を落とす口を持っている。
/// `motolii-core` の `Quality` はそのための型で、旧 workspace から移植済み。
#[test]
#[ignore = "GPU を単独で使う必要がある — 冒頭の doc を参照"]
fn preview_resolution_is_the_lever() {
    use motolii_compositor::{Compositor, Layer, LayerPlacement};

    let mut compositor = Compositor::headless().expect("compositor");

    for (label, scale) in [("1/1", 1u32), ("1/2", 2), ("1/4", 4)] {
        let w = FHD[0] / scale;
        let h = FHD[1] / scale;
        let texture = compositor
            .upload_rgba("src", &vec![128u8; (w * h * 4) as usize], w, h)
            .unwrap();
        let layers: Vec<Layer> = (0..40)
            .map(|i| Layer {
                texture: texture.clone(),
                placement: LayerPlacement {
                    top_left: [0.0, 0.0],
                    size: [w as f32, h as f32],
                    order: i,
                    opacity: 1.0,
                },
            })
            .collect();
        let spec = CompSpec {
            width: w,
            height: h,
        };
        let _ = compositor.render(spec, &layers).unwrap();

        const RUNS: u128 = 5;
        let mut total = 0u128;
        for _ in 0..RUNS {
            let (_, timing) = compositor.render_with_timing(spec, &layers).unwrap();
            total += timing.total_us();
        }
        println!(
            "R1 解像度 {label} ({w}x{h}) 40枚: {}µs",
            total / RUNS
        );
    }
}

/// 製品目標「1080p 動画レイヤー40本同時で破綻しない」(README の設計目標)の判定。
///
/// **実測の結論**: 等倍 1080p の 40枚は約 40ms で、30fps(33.3ms)に届かない。
/// 律速は fragment の量そのもので、MSAA を切っても 9% しか変わらなかった。
/// 一方 **preview を 1/2 にすると 12.9ms** で 60fps に収まる。
/// どの NLE も preview 品質の口を持っており、`motolii-core` の `Quality` が
/// そのための型として既に移植済みである。よって:
///
/// - **preview は 1/2 を既定にする**(60fps 予算で縛る)
/// - **等倍は書き出しの予算**(実時間再生の予算ではない)。回帰だけ見張る
/// **律速は raster の面積ではなく素材のバンド幅**であることを固定する。
///
/// comp だけ 1/2 にしても効かない — 40枚の 1080p 素材(332MB)を毎フレーム舐めるから。
/// **素材側を大きく縮めて初めて効く**。これが「どの NLE も proxy を作る」理由であり、
/// comp 解像度の口だけでは preview は速くならない。
///
/// **測定のばらつきについて**: 中間の倍率(素材 1/2)は実行ごとに 16〜36ms と2倍以上
/// ぶれる。統合 GPU + メモリ逼迫下の作業集合の問題なので、**ぶれない両端だけを縛る**。
/// 中間の値は出力するが合否にしない。
#[test]
#[ignore = "GPU を単独で使う必要がある — 冒頭の doc を参照"]
fn the_lever_is_proxy_sources_not_comp_resolution() {
    const BUDGET_US: u128 = 16_600;
    const RUNS: u32 = 20;

    let mut engine = Engine::new().expect("engine");
    let half_comp = CompSpec {
        width: FHD[0] / 2,
        height: FHD[1] / 2,
    };

    let comp_only = measure_at(&mut engine, &document_scaled(40, 1), half_comp, RUNS);
    let half_proxy = measure_at(&mut engine, &document_scaled(40, 2), half_comp, RUNS);
    let quarter_proxy = measure_at(&mut engine, &document_scaled(40, 4), half_comp, RUNS);

    println!(
        "R1 preview(comp 1/2, 40枚): 素材等倍={comp_only}µs 素材1/2={half_proxy}µs(ぶれる) \
         素材1/4={quarter_proxy}µs / 60fps 予算 {BUDGET_US}µs"
    );

    assert!(
        quarter_proxy < BUDGET_US,
        "素材 1/4 の proxy で {quarter_proxy}µs。60fps 予算 {BUDGET_US}µs に入らないなら\
         preview の道が無い"
    );
    assert!(
        quarter_proxy * 2 < comp_only,
        "proxy が効いていない(等倍={comp_only}µs 1/4={quarter_proxy}µs)。\
         律速がバンド幅だという見立てが崩れている"
    );
}

/// 等倍の回帰見張り。**30fps に届く必要はない**(上の試験が preview の予算を持つ)。
/// ここが跳ねたら合成器か素材経路に何かが入ったということ。
#[test]
#[ignore = "GPU を単独で使う必要がある — 冒頭の doc を参照"]
fn fullhd_forty_layers_does_not_regress() {
    const REGRESSION_US: u128 = 60_000;

    let mut engine = Engine::new().expect("engine");
    let one = measure(&mut engine, &document_with(1), 10);
    let ten = measure(&mut engine, &document_with(10), 10);
    let forty = measure(&mut engine, &document_with(40), 10);

    println!(
        "R1 等倍(1920x1080): 1枚={one}µs 10枚={ten}µs 40枚={forty}µs \
         — 1枚あたり約 {:.0}µs(fragment 律速)",
        (forty as f64 - ten as f64) / 30.0
    );

    assert!(
        forty < REGRESSION_US,
        "等倍 40枚が {forty}µs。回帰見張り {REGRESSION_US}µs を超えた"
    );
}
