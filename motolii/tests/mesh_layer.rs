
use motolii::render::engine::Engine;
use motolii::doc::store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime,
};

const W: u32 = 256;
const H: u32 = 256;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

/// 画面いっぱいの四角を2枚の三角形で。
fn write_fixture_obj() -> std::path::PathBuf {
    let obj = "\
v 40 40 0\nv 200 40 0\nv 200 200 0\nv 40 200 0\n\
f 1 2 3\nf 1 3 4\n";
    let path = std::env::temp_dir().join(format!("motolii-mesh-layer-{}.obj", std::process::id()));
    std::fs::write(&path, obj).expect("write fixture obj");
    path
}

fn place(doc: &mut Document, layer: LayerId, path: &std::path::Path) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File {
                path: path.to_string_lossy().into_owned(),
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

#[test]
fn an_obj_layer_puts_pixels_on_the_frame() {
    let path = write_fixture_obj();
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, &path);

    let mut engine = Engine::new().expect("headless engine");
    let frame = engine
        .render_frame(&doc.view(), t(0))
        .expect("render_frame が網の layer を描けるはず");

    assert_eq!(frame.len(), (W * H * 4) as usize);
    let lit = frame
        .chunks_exact(4)
        .filter(|p| !(p[0] == 0 && p[1] == 0 && p[2] == 0))
        .count();
    assert!(
        lit > 0,
        "網の画素が1つも出ていない(obj → MeshData → run の view へ積む、のどこかで途切れている): {:?}",
        engine.layer_failures()
    );
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_missing_obj_isolates_to_this_layer() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, std::path::Path::new("/nonexistent/none.obj"));

    let mut engine = Engine::new().expect("headless engine");
    let frame = engine
        .render_frame(&doc.view(), t(0))
        .expect("読めない網でも comp 全体は Err にならない");
    assert_eq!(frame.len(), (W * H * 4) as usize);
    assert!(
        !engine.layer_failures().is_empty(),
        "読めない網は理由つきで積まれるはず"
    );
}

/// 平面へ収めると、絵は消えず板になる(裁定 2026-08-30「選択肢として残す」)。
#[test]
fn flattening_keeps_the_picture() {
    use motolii::doc::store::LayerAttrsPatch;

    let path = write_fixture_obj();
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, &path);
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { flatten: Some(true), ..Default::default() },
    })
    .unwrap();

    let mut engine = Engine::new().expect("headless engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("焼いても描けるはず");
    let lit = frame
        .chunks_exact(4)
        .filter(|p| !(p[0] == 0 && p[1] == 0 && p[2] == 0))
        .count();
    assert!(lit > 0, "平面へ収めたら絵が消えた: {:?}", engine.layer_failures());
    std::fs::remove_file(&path).ok();
}
