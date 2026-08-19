//! owns: R0 の合否判定そのもの。上流に「編集ソフトとして成立するか」の審判は無い。
//!
//! 落ちたら 2026-08-20 リセット裁定の軸が立たない。移植より先にこれを通す。
//! 数値は全部出力するので、合否だけでなく実測値を読むこと。

use std::time::Instant;

use r0_store_edit::{
    comp_timeline, dense_track, eval_linear, new_store, read_scalar_on, read_track, store_bytes,
    store_chunks, timeline_names, write_scalar_at_comp, write_track,
};

const PATH: &str = "/comp/layer0/position/x";

/// R0-A: **Document を `comp` 軸に置くと undo が query では成立しない**。
///
/// これは失敗を期待する試験ではなく、素朴な形の限界を機械で固定する試験である。
/// `LatestAtQuery` は単一 timeline しか取らない(`re_chunk/src/latest_at.rs`)ので、
/// 「comp=0 の値を edit=0 時点で」という2次元の問い合わせが原理的に書けない。
#[test]
fn r0_a_comp_axis_cannot_express_undo_by_query() {
    let mut db = new_store();

    // 2打点の property。edit=0 で作る。
    write_scalar_at_comp(&mut db, PATH, 0, 0, 10.0);
    write_scalar_at_comp(&mut db, PATH, 10, 0, 20.0);

    assert_eq!(read_scalar_on(&db, PATH, comp_timeline(), 0), Some(10.0));
    assert_eq!(read_scalar_on(&db, PATH, comp_timeline(), 10), Some(20.0));

    // comp=0 の打点を編集した(edit=1)。
    write_scalar_at_comp(&mut db, PATH, 0, 1, 99.0);
    assert_eq!(read_scalar_on(&db, PATH, comp_timeline(), 0), Some(99.0));

    // ここが要点: comp 軸で読む限り、edit=1 の編集を「無かったこと」にする問い合わせが無い。
    // edit 軸で読めば編集前へ戻れるが、それは comp 位置の情報を失う(下の assert)。
    let on_edit_zero = read_scalar_on(&db, PATH, r0_store_edit::edit_timeline(), 0);
    assert_eq!(
        on_edit_zero,
        Some(20.0),
        "edit 軸の latest-at は『最後に書かれた行』を返すだけで、どの comp 位置の値かを選べない"
    );

    println!(
        "R0-A: timelines={:?} — comp 軸 latest-at は edit を無視する(2次元 query は無い)",
        timeline_names(&db)
    );
}

/// R0-2: track を `edit` 軸へ丸ごと置くと、undo も redo も **query の移動だけ**で成立する。
/// drop も replay も要らない(= rerun blueprint の undo と同じ機構)。
#[test]
fn r0_2_undo_and_redo_are_time_travel() {
    let mut db = new_store();

    write_track(&mut db, PATH, 0, &[(0, 10.0), (10, 20.0)]);
    write_track(&mut db, PATH, 1, &[(0, 99.0), (10, 20.0)]);
    write_track(&mut db, PATH, 2, &[(0, 99.0), (10, 55.0)]);

    assert_eq!(read_track(&db, PATH, 2), Some(vec![(0, 99.0), (10, 55.0)]));
    // undo 1回
    assert_eq!(read_track(&db, PATH, 1), Some(vec![(0, 99.0), (10, 20.0)]));
    // undo 2回
    assert_eq!(read_track(&db, PATH, 0), Some(vec![(0, 10.0), (10, 20.0)]));
    // redo(戻すだけ。store は何も失っていない)
    assert_eq!(read_track(&db, PATH, 2), Some(vec![(0, 99.0), (10, 55.0)]));

    // 1000編集を1回のクエリで飛び越せる(undo 単位は Motolii が選べる)。
    for seq in 3..1003 {
        write_track(&mut db, PATH, seq, &[(0, seq as f32), (10, 20.0)]);
    }
    assert_eq!(read_track(&db, PATH, 0), Some(vec![(0, 10.0), (10, 20.0)]));

    println!("R0-2: undo/redo = latest-at の移動のみ。drop も replay も不要");
}

/// R0-1: 300打点の property を 1000回書き換える(= scrub 中のドラッグ相当)。
#[test]
fn r0_1_edit_storm_does_not_degrade() {
    const KEYS: i64 = 300;
    const EDITS: i64 = 1000;
    const QUERY_BUDGET_US: u128 = 1000; // 1 query < 1ms(60fps の1フレーム 16.6ms に対する余裕)
    const BYTES_BUDGET: u64 = 64 * 1024 * 1024;

    let mut db = new_store();

    write_track(&mut db, PATH, 0, &dense_track(KEYS, 0.0));
    let baseline = query_micros(&db, 0);
    let bytes_after_1 = store_bytes(&db);

    let write_start = Instant::now();
    for seq in 1..EDITS {
        write_track(&mut db, PATH, seq, &dense_track(KEYS, seq as f32));
    }
    let write_elapsed = write_start.elapsed();

    let after = query_micros(&db, EDITS - 1);
    let after_oldest = query_micros(&db, 0);

    let track = read_track(&db, PATH, EDITS - 1).expect("track after storm");
    assert_eq!(track.len(), KEYS as usize);
    assert_eq!(track[0].1, (EDITS - 1) as f32);

    println!(
        "R0-1: {EDITS}編集 × {KEYS}打点 — chunks={} bytes={:.1}MB(1編集時 {:.1}KB)\n      \
         書き込み合計={:?}({:.1}µs/編集)\n      \
         query: 1編集時={baseline}µs 最新={after}µs 最古={after_oldest}µs",
        store_chunks(&db),
        store_bytes(&db) as f64 / 1024.0 / 1024.0,
        bytes_after_1 as f64 / 1024.0,
        write_elapsed,
        write_elapsed.as_micros() as f64 / EDITS as f64,
    );

    assert!(
        after < QUERY_BUDGET_US,
        "最新 query が {after}µs で予算 {QUERY_BUDGET_US}µs を超えた"
    );
    assert!(
        after_oldest < QUERY_BUDGET_US,
        "最古 query(= undo 直後)が {after_oldest}µs で予算 {QUERY_BUDGET_US}µs を超えた"
    );
    assert!(
        store_bytes(&db) < BYTES_BUDGET,
        "store が {}MB まで膨らんだ(予算 {}MB)",
        store_bytes(&db) / 1024 / 1024,
        BYTES_BUDGET / 1024 / 1024
    );
}

/// R0-3: 300打点 × 10 property を、300フレーム分ぜんぶ評価して 60fps に収まるか。
/// キャッシュを置かない素朴な経路で測る — ここが通るなら front に cache 層が要らない。
#[test]
fn r0_3_keyframe_density_fits_60fps() {
    const KEYS: i64 = 300;
    const PROPS: usize = 10;
    const FRAMES: i64 = 300;
    const FRAME_BUDGET_US: u128 = 16_600;

    let mut db = new_store();
    let paths: Vec<String> = (0..PROPS)
        .map(|i| format!("/comp/layer{i}/position/x"))
        .collect();
    for (i, path) in paths.iter().enumerate() {
        write_track(&mut db, path, 0, &dense_track(KEYS, i as f32));
    }

    let start = Instant::now();
    let mut sink = 0.0f32;
    for frame in 0..FRAMES {
        for path in &paths {
            let track = read_track(&db, path, 0).expect("track");
            sink += eval_linear(&track, frame);
        }
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed.as_micros() / FRAMES as u128;

    println!(
        "R0-3: {PROPS}property × {KEYS}打点 × {FRAMES}フレーム — 合計={:?} 1フレーム={per_frame}µs \
         (予算 {FRAME_BUDGET_US}µs) sink={sink}",
        elapsed
    );

    assert!(
        per_frame < FRAME_BUDGET_US,
        "1フレーム {per_frame}µs で 60fps 予算 {FRAME_BUDGET_US}µs を超えた"
    );
}

/// R0-4: 保存 → 読込 で全 query の結果が一致すること。
#[test]
fn r0_4_save_load_roundtrip() {
    let mut db = new_store();
    for seq in 0..50 {
        write_track(&mut db, PATH, seq, &dense_track(64, seq as f32));
    }

    let messages: Vec<_> = db
        .to_messages(None)
        .collect::<Result<Vec<_>, _>>()
        .expect("to_messages");

    let mut restored = new_store();
    for msg in &messages {
        restored.add_log_msg(msg).expect("add_log_msg");
    }

    for seq in 0..50 {
        assert_eq!(
            read_track(&db, PATH, seq),
            read_track(&restored, PATH, seq),
            "edit={seq} で保存前後が食い違った"
        );
    }

    println!(
        "R0-4: {}メッセージで往復一致。timelines={:?}",
        messages.len(),
        timeline_names(&restored)
    );
}

/// R0-5: Motolii 側で建てた custom component が rerun の木の外で往復すること(裁定4の前提)。
#[test]
fn r0_5_custom_components_live_outside_the_rerun_tree() {
    let mut db = new_store();
    let keys = vec![(0, 1.5), (7, -2.25), (100, 0.0)];
    write_track(&mut db, PATH, 0, &keys);

    assert_eq!(read_track(&db, PATH, 0), Some(keys));
    println!("R0-5: motolii.KeyFrame / motolii.KeyValue は re_types を fork せずに成立");
}

fn query_micros(db: &re_entity_db::EntityDb, at_edit: i64) -> u128 {
    let start = Instant::now();
    let track = read_track(db, PATH, at_edit);
    let elapsed = start.elapsed().as_micros();
    assert!(track.is_some());
    elapsed
}
