//! 意味等価の柵(裁定140): `StoreView::track()` の**解析済みキャッシュ**は
//! 常に「その revision で素解析したのと同じ値」を返さねばならない。
//!
//! `view.rs::source_at_path` は `Document::track_cache`(revision 鍵)を経由するが、
//! 呼び手(このテストを含め crate 外)から見た挙動は「毎回素で `serde_json` を
//! 解いた場合」と**区別できてはいけない** — 区別できるなら、それは front に
//! 「前回の値」が見える二重帳簿の入口になる(タスク#19 の裁定140 が防ごうとした穴)。
//!
//! 検査の型: 同じ edit 列を2本の `Document` へ適用する。片方(`warm`)は**edit ごとに
//! 読んでからキャッシュを温める**(そのエントリは次の edit で古くなる)。もう片方
//! (`cold`)は最後まで一度も読まない(= 毎回が実質的な素解析)。両方の最終状態は
//! 同じ edit 列を経ているので、最後の読みは**warm=キャッシュ経由 / cold=素解析**の
//! 対比になる — 一致しなければキャッシュが古い値を漏らしている。

use motolii_store::{
    property, Document, Interp, Intent, Keyframe, KeyframeTrack, LayerId, PropertyId,
    RationalTime, Value,
};
use proptest::prelude::*;

const LAYER: LayerId = LayerId(1);

fn track_prop() -> PropertyId {
    PropertyId::new(property::POSITION_X).expect("property name")
}

/// キャッシュを素通りさせる「無関係な」property。`SetTrack`(対象 track を触らない)
/// でも revision は動くので、cache 無効化が「対象 property のキーだけ」でなく
/// **revision 丸ごと**であることも同時に検査する。
fn other_prop() -> PropertyId {
    PropertyId::new(property::OPACITY).expect("property name")
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).expect("rational time")
}

/// `values` の各要素を `k*10` フレーム目のキーにした track(裁定92 の平坦 track と
/// 同じ形、`Interp::Linear`)。
fn track_from(values: &[f64]) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for (k, value) in values.iter().enumerate() {
        track.insert(Keyframe {
            t: t(k as i64 * 10),
            value: Value::F64(*value),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    track
}

#[derive(Clone, Debug)]
enum Edit {
    /// 対象 track そのものを書き換える。
    SetTarget(Vec<f64>),
    /// 対象とは別の property を書き換える — revision は動くが対象 track の
    /// JSON 自体は変わらない。
    SetOther(f64),
}

fn edit_strategy() -> impl Strategy<Value = Edit> {
    prop_oneof![
        prop::collection::vec(-1000.0f64..1000.0, 1..=5).prop_map(Edit::SetTarget),
        (-1000.0f64..1000.0).prop_map(Edit::SetOther),
    ]
}

fn apply_edit(doc: &mut Document, edit: &Edit) {
    let intent = match edit {
        Edit::SetTarget(values) => Intent::SetTrack {
            layer: LAYER,
            property: track_prop(),
            track: track_from(values),
        },
        Edit::SetOther(value) => Intent::SetTrack {
            layer: LAYER,
            property: other_prop(),
            track: track_from(&[*value]),
        },
    };
    doc.apply(intent).expect("apply must succeed for a well-formed SetTrack");
}

proptest! {
    /// warm(古くなったキャッシュを毎回作ってから edit する)と cold(最後まで
    /// 一度も読まない = 実質的に毎回素解析)が、同じ edit 列の後で**同じ値**を返す。
    #[test]
    fn cached_track_equals_a_cold_parse_after_any_edit_sequence(edits in prop::collection::vec(edit_strategy(), 0..=8)) {
        let mut warm = Document::new();
        warm.apply(Intent::AddLayer(LAYER)).unwrap();
        let mut cold = Document::new();
        cold.apply(Intent::AddLayer(LAYER)).unwrap();

        for edit in &edits {
            // **意図的にキャッシュへ古い値を仕込む** — この読みの結果は次の edit で
            // revision が動くので古くなる。ここを読まない cold 側との対比が
            // 「キャッシュ経由 vs 素解析」の対比そのもの。
            let _ = warm.view().track(LAYER, &track_prop());
            let _ = warm.view().track(LAYER, &other_prop());

            apply_edit(&mut warm, edit);
            apply_edit(&mut cold, edit);
        }

        let warm_target = warm.view().track(LAYER, &track_prop()).unwrap();
        let cold_target = cold.view().track(LAYER, &track_prop()).unwrap();
        prop_assert_eq!(warm_target, cold_target);

        let warm_other = warm.view().track(LAYER, &other_prop()).unwrap();
        let cold_other = cold.view().track(LAYER, &other_prop()).unwrap();
        prop_assert_eq!(warm_other, cold_other);
    }

    /// 同一 revision 内で `track()` を連続で呼んでも(2回目はキャッシュ hit)、
    /// 1回目(必然的に miss = 素解析)と同じ値を返す。
    #[test]
    fn repeated_reads_within_the_same_revision_agree(values in prop::collection::vec(-1000.0f64..1000.0, 1..=5)) {
        let mut doc = Document::new();
        doc.apply(Intent::AddLayer(LAYER)).unwrap();
        doc.apply(Intent::SetTrack {
            layer: LAYER,
            property: track_prop(),
            track: track_from(&values),
        })
        .unwrap();

        let view = doc.view();
        let first = view.track(LAYER, &track_prop()).unwrap();
        let second = view.track(LAYER, &track_prop()).unwrap();
        let third = view.track(LAYER, &track_prop()).unwrap();
        prop_assert_eq!(&first, &second);
        prop_assert_eq!(&second, &third);
    }
}
