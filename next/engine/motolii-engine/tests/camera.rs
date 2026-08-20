//! `Composition.camera` の通し試験(裁定113/115、裁定116 実装)。
//!
//! `motolii-store`(数学は `motolii-core`)と `motolii-compositor` はそれぞれ単体で
//! 縛ってある。ここは `Engine::render_frame` を通しても同じことが言えるかだけを見る —
//! **camera も comp と同じく `view` から読む**(引数で渡さない、裁定40 と同じ規律)ので、
//! 「Document にカメラを書いたら、次の `render_frame` がそれを拾う」を確かめる。

use motolii_engine::Engine;
use motolii_store::{
    property, Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerAttrs,
    LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

const W: u32 = 64;
const H: u32 = 64;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    doc
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
    });
    track
}

fn camera_prop(name: &str) -> PropertyId {
    PropertyId::camera(name).unwrap()
}

/// camera を一切触っていない Document は、camera を書いたことのある Document の
/// 「既定カメラ」と同じ絵を出す — `resolve_camera` の既定値がそのまま
/// `Engine::render_frame` まで通ることの確認。
#[test]
fn a_document_without_camera_edits_renders_the_same_as_an_explicit_default_camera() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 0, 0, 255],
                width: W,
                height: H,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let untouched = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ZOOM),
        track: still(Value::F64(1.0)),
    })
    .unwrap();
    let explicit_default = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_eq!(untouched, explicit_default);
}

/// Document へ書いたカメラの変化が `render_frame` の出力を変える — camera が
/// `view` から読まれていて、engine に別の入れ口が無いことの通し確認。
#[test]
fn zooming_the_camera_in_the_document_changes_the_rendered_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 0, 0, 255],
                width: 24,
                height: 24,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let before = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ZOOM),
        track: still(Value::F64(3.0)),
    })
    .unwrap();
    let after = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_ne!(before, after, "zoom を書いたのに絵が変わらない");
}

/// pinned な layer は、Document のカメラをどれだけ動かしても
/// `Engine::render_frame` の出力上は不動(裁定113、通し確認)。
#[test]
fn a_pinned_layer_does_not_move_when_the_document_camera_moves() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 0, 0, 255],
                width: 24,
                height: 24,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer,
        attrs: LayerAttrs {
            pinned: true,
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let before = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_CENTER),
        track: still(Value::Vec2([15.0, -8.0])),
    })
    .unwrap();
    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ROLL),
        track: still(Value::F64(20.0)),
    })
    .unwrap();
    let after = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_eq!(before, after, "pinned な layer が Document のカメラで動いてしまった");
    // 何かは描けている(全部背景に消えていない)ことの保険。
    assert!(pixel(&before, 32, 32)[0] > 0 || pixel(&before, 10, 10)[0] > 0);
}
