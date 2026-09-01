//! 選んだ層は、絵の側でも選ばれて見えなければならない。
//!
//! 枠が出るかは寸法が出るかで決まる。寸法が出ないと Stage は枠も取っ手も
//! 描けず、触る側からは**選べていないように見える**。

mod testkit;

use motolii::doc::store::{
    Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming, RationalTime,
};
use motolii::render::engine::Engine;

fn doc_with(path: &str) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 480,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 30,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File { path: path.to_owned(), fingerprint: None },
            order: 0,
            timing: LayerTiming::place(0, None, 30),
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { name: Some("logo".into()), ..Default::default() },
    })
    .unwrap();
    doc
}

#[test]
fn a_still_image_layer_knows_its_own_size_so_the_frame_can_be_drawn() {
    let dir = testkit::tmp_dir("selected-still-size");
    let path = dir.join("logo.png");
    image::RgbaImage::from_pixel(64, 48, image::Rgba([255, 80, 80, 255]))
        .save(&path)
        .unwrap();
    let path = path.to_str().unwrap().to_owned();

    let doc = doc_with(&path);
    let layer = LayerId(1);
    let at = RationalTime::try_new(0, 30).unwrap();

    let mut engine = Engine::new().unwrap();
    engine.render_frame(&doc.view(), at).unwrap();

    let size = engine.selected_layer_size(&doc.view(), layer, at);
    assert_eq!(
        size,
        Some([64.0, 48.0]),
        "静止画の層に寸法が無い —— Stage は枠も取っ手も描けない: {:?}",
        engine.layer_failures()
    );
}
