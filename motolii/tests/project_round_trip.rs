//! 作った物を仕舞って、開き直せる。
//!
//! 窓に New / Open / Save が無いと、**白紙から始められず、続きもできない**。
//! 機械は前から在ったので、ここで見ているのは道が通っているか。

mod testkit;

use motolii::doc::store::{
    Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming,
};

fn a_doc(name: &str) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1800,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Shape,
            order: 0,
            timing: LayerTiming::place(0, None, 1800),
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            name: Some(name.to_owned()),
            ..Default::default()
        },
    })
    .unwrap();
    doc
}

#[test]
fn a_saved_project_opens_again_with_its_layers() {
    let dir = testkit::tmp_dir("project-round-trip");
    let path = dir.join("song.rrd");

    a_doc("タイトルロゴ").save(&path).unwrap();
    let back = Document::load(&path).unwrap();

    let view = back.view();
    let comp = view.composition().unwrap().expect("comp が消えた");
    assert_eq!((comp.width, comp.height), (1920, 1080));
    let names: Vec<_> = view
        .layers()
        .iter()
        .filter_map(|l| view.attrs(*l).ok().flatten().map(|a| a.name))
        .collect();
    assert!(
        names.iter().any(|n| n == "タイトルロゴ"),
        "層の名前が消えた: {names:?}"
    );
}

#[test]
fn a_new_project_is_empty_but_has_a_frame() {
    let doc = motolii::ui::blank_project();
    let view = doc.view();
    assert!(view.layers().is_empty(), "白紙に層が居る");
    let comp = view.composition().unwrap().expect("白紙にも枠は要る");
    assert_eq!((comp.width, comp.height), (1920, 1080));
}

#[test]
fn a_failed_replacement_keeps_the_previous_project_readable() {
    let dir = testkit::tmp_dir("project-atomic-save");
    let path = dir.join("song.rrd");
    a_doc("old layer").save(&path).unwrap();

    std::fs::create_dir(dir.join(".song.rrd.tmp")).unwrap();
    let result = a_doc("new layer").save(&path);
    assert!(
        result.is_err(),
        "the blocked temporary path did not stop Save"
    );

    let back = Document::load(&path).expect("the previous project was damaged by failed Save");
    let view = back.view();
    let names = view
        .layers()
        .iter()
        .filter_map(|layer| view.attrs(*layer).ok().flatten().map(|attrs| attrs.name))
        .collect::<Vec<_>>();
    assert!(
        names.iter().any(|name| name == "old layer"),
        "old project was replaced: {names:?}"
    );
    assert!(
        !names.iter().any(|name| name == "new layer"),
        "failed Save became visible: {names:?}"
    );
}
