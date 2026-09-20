use crate::edit::{Animate, Document, Intent};
use crate::doc::store::*;

/// Ease(Sequence)を掴んでいる間の下書き: previewSequence で status のゴーストが変わり、
/// cancel で戻り、sequence で本書きになる。
#[test]
fn preview_sequence_shows_in_status_and_cancels() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    for id in [11u64, 12] {
        rt.doc.apply_all([
            Intent::AddLayer(LayerId(id)),
            Intent::SetMeta { layer: LayerId(id), meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, 60) } },
        ]).unwrap();
    }
    let ghost_of = |rt: &crate::EditorRuntime, id: u64| -> serde_json::Value {
        let status = rt.status().unwrap();
        status["layers"].as_array().unwrap().iter().find(|l| l["id"] == id).map(|l| l["ghost"].clone()).unwrap()
    };
    rt.request(serde_json::json!({"op":"previewSequence","layers":[11,12],"ghosts":[0,7]})).unwrap();
    assert_eq!(rt.doc.view().attrs(LayerId(12)).unwrap().unwrap().ghost, Some(7), "view().attrs に下書きが乗る");
    assert_eq!(ghost_of(&rt, 12), serde_json::json!(7), "下書きが status に出る");
    assert!(ghost_of(&rt, 11).is_null());
    rt.request(serde_json::json!({"op":"cancelPreview"})).unwrap();
    assert!(ghost_of(&rt, 12).is_null(), "cancel で戻る");
    rt.request(serde_json::json!({"op":"sequence","layers":[11,12],"ghosts":[0,7]})).unwrap();
    assert_eq!(ghost_of(&rt, 12), serde_json::json!(7), "本書き");
    assert_eq!(rt.doc.view().attrs(LayerId(12)).unwrap().unwrap().ghost, Some(7));
}

/// 姿を持たない層(Null・HDR・音声・カメラ)はゴーストを持てない(裁定 2026-09-07 利用者「旨みがない」)。
/// status は `ghostable` で知らせ、sequence はその層を飛ばし、setAttrs は断る。
#[test]
fn layers_without_a_look_cannot_carry_a_ghost() {
    use crate::doc::store::{LayerId, LayerMeta, LayerSource, LayerTiming};
    let mut rt = crate::EditorRuntime::open("").unwrap();
    let sources = [
        (21u64, LayerSource::Shape),
        (22, LayerSource::Null),
        (23, LayerSource::File { path: "sky.hdr".into(), fingerprint: None }),
        (24, LayerSource::File { path: "voice.wav".into(), fingerprint: None }),
        (25, LayerSource::Camera),
    ];
    for (id, source) in sources {
        rt.doc.apply_all([
            Intent::AddLayer(LayerId(id)),
            Intent::SetMeta { layer: LayerId(id), meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 60) } },
        ]).unwrap();
    }
    let status = rt.status().unwrap();
    let ghostable = |id: u64| status["layers"].as_array().unwrap().iter().find(|l| l["id"] == id).unwrap()["ghostable"] == true;
    assert!(ghostable(21));
    for id in [22, 23, 24, 25] { assert!(!ghostable(id), "layer {id}"); }
    rt.request(serde_json::json!({"op":"sequence","layers":[21,22,23,24],"ghosts":[0,3,6,9]})).unwrap();
    for id in [22, 23, 24] {
        assert_eq!(rt.doc.view().attrs(LayerId(id)).unwrap().unwrap_or_default().ghost, None, "layer {id} は飛ばされる");
    }
    assert!(rt.request(serde_json::json!({"op":"setAttrs","layers":[24],"patch":{"ghost":5}})).is_err(), "音声に直接付けても断る");
    // 環境層は doc 自身が断り、環境にした瞬間に既存のゴーストも落ちる。
    rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"ghost":5}})).unwrap();
    rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"environment":true}})).unwrap();
    assert_eq!(rt.doc.view().attrs(LayerId(21)).unwrap().unwrap().ghost, None);
    assert!(rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"ghost":5}})).is_err());
}
