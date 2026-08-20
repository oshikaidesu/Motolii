//! id 採番 — 墓標(tombstone)を id 再利用から守る(2026-08-20 の敵対的レビュー)。
//!
//! 敵対的レビューが実証した欠陥: shell の `next_layer_id()` は「現存する layer の
//! 最大 id + 1」で採番していた。**削除は墓標**(`present=false` の append であって
//! drop ではない、裁定2/56)なので、最大 id の layer を削除した直後にもう1枚置くと、
//! この素朴な採番は死んだ layer の id を再利用する。そこには2つの事故がある:
//!
//! - **根**: 死んだ layer には既に `meta`/property track/mask の component が
//!   entity path(`/layer/{id}`)に残っている。id を再利用すると、それらが新しい
//!   layer へ「復活」して付き直る
//! - **症状**: `Intent::SetMeta` は新規配置専用の柵(裁定108(c))を持つので、
//!   再利用した id には既に `meta` があり、正当な新規配置が `Err` になる
//!
//! 修正: 採番の正本を store 側 [`motolii_store::StoreView::next_layer_id`]
//! (**墓標込みの最大 id + 1**)へ移した。shell の `next_layer_id()` はこれを
//! 呼ぶだけになった(`next/shell/motolii-shell/src/lib.rs`)。

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn solid() -> LayerSource {
    LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    }
}

fn place(doc: &mut Document, layer: LayerId, start: i64, duration: i64) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming {
                    start,
                    duration,
                    source_in: 0,
                    ..Default::default()
                },
            },
        },
    ])
    .unwrap();
}

/// 旧 shell 実装のロジック(「現存層の最大 + 1」)をそのまま再現する。
/// この関数自体が欠陥の実体 — 墓標を見ないので死んだ id を再利用しうる。
fn naive_present_only_next_id(doc: &Document) -> u64 {
    doc.view()
        .layers()
        .last()
        .map(|last| last.0 + 1)
        .unwrap_or(1)
}

#[test]
fn removing_the_highest_layer_then_replacing_it_does_not_collide_with_the_tombstone() {
    let mut doc = doc_with_comp(300);
    let (l1, l2) = (LayerId(1), LayerId(2));
    place(&mut doc, l1, 0, 100);
    place(&mut doc, l2, 0, 100);

    // l2 にだけトラックを打っておく — id を再利用したら、このトラックが新しい layer に
    // 「復活」して見えるはず。
    let opacity = PropertyId::new(property::OPACITY).unwrap();
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.25),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer: l2,
        property: opacity.clone(),
        track,
    })
    .unwrap();

    doc.apply(Intent::RemoveLayer(l2)).unwrap(); // 墓標。meta / opacity track は残る。

    // 前提の確認: 素朴な採番はこの時点で死んだ id 2 を再利用しようとする。
    assert_eq!(
        naive_present_only_next_id(&doc),
        2,
        "この試験の前提(旧ロジックが衝突すること)が崩れている"
    );

    // 正本の採番は墓標を数えるので 3 を返す。
    let next_id = doc.view().next_layer_id();
    assert_eq!(next_id, 3, "next_layer_id が墓標の id を再利用してしまっている");

    // その id で新規配置すると成功する(SetMeta の新規配置専用の柵に引っかからない)。
    let fresh = LayerId(next_id);
    place(&mut doc, fresh, 50, 20);
    assert!(doc.view().has_layer(fresh));

    // 死んだ layer 2 の opacity track が新しい layer へ付き直っていない。
    assert_eq!(
        doc.view().value_at(fresh, &opacity, t(0)).unwrap(),
        None,
        "墓標の id を再利用していたら、死んだ layer のトラックが新しい layer に付いている"
    );
}

/// 素朴な採番(現存層のみ)で id を再利用すると、`Intent::SetMeta` の新規配置専用の柵
/// (裁定108(c))に引っかかって正当な新規配置が `Err` になる — 敵対的レビューが最初に
/// 見つけた症状そのもの。`next_layer_id` を使えばこの柵に触れないことの対比。
#[test]
fn reusing_a_tombstoned_id_trips_the_new_placement_guard() {
    let mut doc = doc_with_comp(300);
    let (l1, l2) = (LayerId(1), LayerId(2));
    place(&mut doc, l1, 0, 100);
    place(&mut doc, l2, 0, 100);
    doc.apply(Intent::RemoveLayer(l2)).unwrap();

    let reused_id = LayerId(naive_present_only_next_id(&doc));
    assert_eq!(reused_id, l2, "この試験の前提が崩れている");

    doc.apply(Intent::AddLayer(reused_id)).unwrap();
    let result = doc.apply(Intent::SetMeta {
        layer: reused_id,
        meta: LayerMeta {
            source: solid(),
            order: 0,
            timing: LayerTiming {
                start: 0,
                duration: 10,
                source_in: 0,
                ..Default::default()
            },
        },
    });
    assert!(
        result.is_err(),
        "墓標の id を再利用すると、正当な新規配置が SetMeta の柵に拒まれるはず \
         (next_layer_id を使えばそもそもこの id を選ばない)"
    );

    // 正本の採番ならこの衝突を避ける。
    let safe_id = LayerId(doc.view().next_layer_id());
    assert_ne!(safe_id, l2);
}
