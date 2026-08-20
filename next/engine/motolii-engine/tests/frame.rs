//! Document から絵が出るまでの通し試験。
//!
//! store(意味)→ eval(補間)→ compositor(合成)が1本で繋がっていることを見る。
//! ここが通ると「Document のある時刻の姿」が初めて画になる。

use motolii_compositor::CompSpec;
use motolii_engine::Engine;
use motolii_store::{Composition, 
    property, Document, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta, LayerSource,
    PropertyId, RationalTime, Value,
};
use motolii_store::LayerTiming;

const W: u32 = 64;
const H: u32 = 64;

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn solid(rgba: [u8; 4], w: u32, h: u32) -> LayerSource {
    LayerSource::Solid {
        rgba,
        width: w,
        height: h,
    }
}

#[test]
fn document_becomes_a_frame() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    let back = LayerId(1);
    let front = LayerId(2);

    doc.apply(Intent::AddLayer(back)).unwrap();
    doc.apply(Intent::SetMeta {
        layer: back,
        meta: LayerMeta {
            source: solid([255, 0, 0, 255], W, H),
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    doc.apply(Intent::AddLayer(front)).unwrap();
    doc.apply(Intent::SetMeta {
        layer: front,
        meta: LayerMeta {
            source: solid([0, 255, 0, 255], 32, 32),
            order: 1,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert!(
        pixel(&frame, 10, 10)[1] > pixel(&frame, 10, 10)[0],
        "手前の緑 layer が左上を覆う: {:?}",
        pixel(&frame, 10, 10)
    );
    assert!(
        pixel(&frame, 50, 50)[0] > pixel(&frame, 50, 50)[1],
        "覆われていない所は奥の赤: {:?}",
        pixel(&frame, 50, 50)
    );
}

/// keyframe が実際に画を動かすこと。**これが通ると「動画ソフト」の芯が繋がる**。
#[test]
fn keyframes_move_the_picture_over_time() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    let layer = LayerId(1);

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: solid([255, 255, 255, 255], 16, 16),
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    // 0フレームで左上、30フレームで右下へ動く。
    let mut x = KeyframeTrack::new();
    x.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
    });
    x.insert(Keyframe {
        t: t(30),
        value: Value::F64(48.0),
        interp: Interp::Linear,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION_X).unwrap(),
        track: x,
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let at_start = engine.render_frame(&doc.view(), t(0)).unwrap();
    let at_mid = engine.render_frame(&doc.view(), t(15)).unwrap();
    let at_end = engine.render_frame(&doc.view(), t(30)).unwrap();

    assert!(pixel(&at_start, 4, 4)[0] > 128, "0フレームでは左上に居る");
    assert!(pixel(&at_start, 56, 4)[0] < 128, "0フレームでは右上に居ない");

    assert!(pixel(&at_mid, 28, 4)[0] > 128, "15フレームでは中央付近");

    assert!(pixel(&at_end, 56, 4)[0] > 128, "30フレームでは右上に居る");
    assert!(pixel(&at_end, 4, 4)[0] < 128, "30フレームでは左上に居ない");

    assert_ne!(at_start, at_end, "時間で絵が変わらないなら動画ではない");
}

/// undo が**絵まで**戻ること。store の時間旅行が画に効いているかを見る。
#[test]
fn undo_changes_the_rendered_frame() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    let layer = LayerId(1);

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: solid([255, 0, 0, 255], W, H),
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let red = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: solid([0, 0, 255, 255], W, H),
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
    let blue = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_ne!(red, blue);

    assert!(doc.undo());
    let back_to_red = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_eq!(red, back_to_red, "undo は絵まで byte 一致で戻るべき");
}
