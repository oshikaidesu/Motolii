//! marker — コンポジションのロケータ。Lottie の `helpers/marker`(cm/tm/dr)から取る。
//!
//! **layer のマスクとは違い、マーカーには他の property から参照される id が無い**
//! (`mask.{id}.shape` のような名前付けの相手が居ない)ので `MaskId` に相当する
//! 安定識別子は持たない。並びは `Vec<Marker>` の順そのもの。
//!
//! ここで固定するのは3つ:
//! - マーカーは comp 全体の持ち物(layer には付かない)
//! - 編集は `edit` timeline 上の普通の刻み(undo が効く)
//! - 保存は履歴を畳んでも残る(裁定56 の畳みが知らない component にしない)

use motolii_store::{Composition, Document, Fps, Intent, Marker, RationalTime};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
    }))
    .unwrap();
    doc
}

/// マーカーが1枚も無ければ空。**「無い」と「読めない」の区別**(裁定37)は
/// マスクと同じく「無い = 空」でよい — 壊れているのは JSON が読めない時だけ。
#[test]
fn no_markers_is_an_empty_list_not_an_error() {
    let doc = doc_with_comp();
    assert_eq!(doc.view().markers().unwrap(), Vec::new());
}

/// 名前・時刻・尺を持ち、順序は明示された列の順(裁定66 と同じ考え方 — 添字も
/// 暗黙の隣接参照も作らない)。
#[test]
fn markers_keep_their_declared_order() {
    let mut doc = doc_with_comp();
    let markers = vec![
        Marker {
            name: "Verse".to_owned(),
            time: t(0),
            duration: t(0),
        },
        Marker {
            name: "Chorus".to_owned(),
            time: t(30),
            duration: t(15),
        },
    ];
    doc.apply(Intent::SetMarkers {
        markers: markers.clone(),
    })
    .unwrap();

    assert_eq!(doc.view().markers().unwrap(), markers);
}

/// **範囲ロケータの尺は duration**(out point ではない、裁定64 と同じ理由)。
/// 尺 0 は「範囲ではなく単発のロケータ」を表せる。
#[test]
fn duration_is_a_length_not_an_out_point() {
    let mut doc = doc_with_comp();
    doc.apply(Intent::SetMarkers {
        markers: vec![Marker {
            name: "hit".to_owned(),
            time: t(10),
            duration: t(0),
        }],
    })
    .unwrap();

    let markers = doc.view().markers().unwrap();
    assert_eq!(markers[0].duration, t(0));
}

/// マーカーの編集も `edit` timeline 上の普通の刻み。自前の履歴機構を持たない。
#[test]
fn editing_markers_is_undoable_like_any_other_edit() {
    let mut doc = doc_with_comp();
    doc.apply(Intent::SetMarkers {
        markers: vec![Marker {
            name: "one".to_owned(),
            time: t(0),
            duration: t(0),
        }],
    })
    .unwrap();
    assert_eq!(doc.view().markers().unwrap().len(), 1);

    assert!(doc.undo());
    assert!(
        doc.view().markers().unwrap().is_empty(),
        "マーカー追加が undo で戻らない"
    );
    assert!(doc.redo());
    assert_eq!(doc.view().markers().unwrap()[0].name, "one");
}

/// 保存は履歴を畳む(裁定56)。マーカーは `meta`/`composition` の中に無いぶん、
/// ここで固定しておかないと `flattened()` の手列挙から黙って落ちる(裁定108(b) と同型)。
#[test]
fn markers_survive_a_save_and_load_round_trip() {
    let mut doc = doc_with_comp();
    doc.apply(Intent::SetMarkers {
        markers: vec![Marker {
            name: "drop".to_owned(),
            time: t(120),
            duration: t(8),
        }],
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-marker-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("markers.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    assert_eq!(
        loaded.view().markers().unwrap(),
        vec![Marker {
            name: "drop".to_owned(),
            time: t(120),
            duration: t(8),
        }],
        "保存で畳む時にマーカーが落ちている"
    );
}
