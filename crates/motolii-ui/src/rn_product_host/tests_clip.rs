//! trim / mark / split / select の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn set_position_key_time_and_clip_edits_update_timeline_projection_and_undo() {
    let _lock = test_lock();
    let host = create_host("timeline-edit-intents");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(target);
        let published = product
            .runtime
            .process_next(&mut queue, product.primary, product.projection_generation)
            .expect("process")
            .expect("published");
        product.primary = published.primary;
        product.projection_generation = published.projection_generation;
        Ok(())
    })
    .expect("seed primary");

    let before = read_snapshot(host);
    let before_layer = before
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    let before_start = before_layer.start;
    let before_duration = before_layer.duration;

    let add = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(add.accepted);
    let key_id = add
        .snapshot
        .expect("keyed")
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .position_keys[0]
        .key_id
        .clone();

    let moved_key = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                r#""host_handle":"{host}","target":"{layer}","key_id":"{key}","#,
                r#""time":{{"num":2,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
            key = key_id,
        ),
    );
    assert!(moved_key.accepted);
    let after_key_layer = moved_key
        .snapshot
        .expect("key moved")
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(
        after_key_layer.position_keys[0].time,
        RationalTime::from_seconds(2)
    );
    assert_eq!(after_key_layer.position_keys[0].key_id, key_id);

    // 先に右edgeを短くしてから start を動かす(compositionはみ出しを避ける)。
    let trimmed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":3,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(trimmed.accepted);
    let after_trim_layer = trimmed
        .snapshot
        .expect("trimmed")
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(after_trim_layer.start, before_start);
    assert_eq!(after_trim_layer.duration, RationalTime::from_seconds(3));

    let moved_clip = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_clip_start","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":2}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(moved_clip.accepted);
    let after_move_layer = moved_clip
        .snapshot
        .expect("clip moved")
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(after_move_layer.start, RationalTime::try_new(1, 2).unwrap());
    assert_eq!(after_move_layer.duration, RationalTime::from_seconds(3));

    let trimmed_in = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(trimmed_in.accepted);
    let after_in_layer = trimmed_in
        .snapshot
        .expect("in")
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(after_in_layer.start, RationalTime::from_seconds(1));
    // start 1..3.5 → duration 2.5 after left trim from 0.5
    assert_eq!(
        after_in_layer.duration,
        RationalTime::try_new(5, 2).unwrap()
    );

    for _ in 0..5 {
        assert!(dispatch_raw_json(
                host,
                &format!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
                ),
            )
            .accepted);
    }
    let restored_layer = read_snapshot(host)
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(restored_layer.start, before_start);
    assert_eq!(restored_layer.duration, before_duration);
    assert!(restored_layer.position_keys.is_empty());
    let _ = host_destroy_for_test(host);
}

#[test]
fn trim_then_set_time_drops_same_layer_from_stage_geometry() {
    let _lock = test_lock();
    let host = create_host("timeline-trim-stage-geometry");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    seed_primary(host, target);

    let seed = read_wire(host);
    assert_eq!(seed.stage_geometry.layers.len(), 1);
    assert_eq!(seed.stage_geometry.layers[0].layer_id, layer_id);
    assert_eq!(seed.current_time, RationalTime::ZERO);

    let trimmed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(trimmed.accepted);
    let at_zero = read_wire(host);
    assert_eq!(at_zero.stage_geometry.layers.len(), 1);
    assert_eq!(at_zero.stage_geometry.layers[0].layer_id, layer_id);

    // fps 30: frame 60 = 2s。clip は 0..1s なので playhead 外 → Stage から消える。
    let moved = dispatch_raw_json(host, &set_time_json(host, "60"));
    assert!(moved.accepted);
    let outside = read_wire(host);
    assert_eq!(outside.current_time, RationalTime::from_seconds(2));
    assert!(
        outside
            .stage_geometry
            .layers
            .iter()
            .all(|layer| layer.layer_id != layer_id),
        "trimmed clip must leave Stage at playhead 2s"
    );

    let restored = dispatch_raw_json(host, &set_time_json(host, "0"));
    assert!(restored.accepted);
    let inside = read_wire(host);
    assert_eq!(inside.current_time, RationalTime::ZERO);
    assert_eq!(inside.stage_geometry.layers.len(), 1);
    assert_eq!(inside.stage_geometry.layers[0].layer_id, layer_id);
    let _ = host_destroy_for_test(host);
}

#[test]
fn mark_in_at_playhead_trims_clip_and_drops_from_stage() {
    let _lock = test_lock();
    let host = create_host("mark-in-playhead");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    seed_primary(host, target);
    let fps = baseline.timeline.fps;
    assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
    let playhead = read_wire(host).current_time;
    assert_eq!(
        playhead,
        RationalTime::try_from_frame(30, fps).expect("frame 30")
    );

    let marked = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":{num},"den":{den}}}}}"#
            ),
            host = host,
            layer = layer_id,
            num = playhead.num(),
            den = playhead.den(),
        ),
    );
    assert!(marked.accepted);
    let after = marked.snapshot.expect("trimmed in");
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(layer.start, playhead);

    assert!(dispatch_raw_json(host, &set_time_json(host, "0")).accepted);
    let outside = read_wire(host);
    assert!(
        outside
            .stage_geometry
            .layers
            .iter()
            .all(|layer| layer.layer_id != layer_id),
        "clip in after playhead must leave Stage at t=0"
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn mark_out_at_playhead_trims_clip_and_drops_from_stage() {
    let _lock = test_lock();
    let host = create_host("mark-out-playhead");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    seed_primary(host, target);
    let fps = baseline.timeline.fps;
    assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
    let playhead = read_wire(host).current_time;

    let marked = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":{num},"den":{den}}}}}"#
            ),
            host = host,
            layer = layer_id,
            num = playhead.num(),
            den = playhead.den(),
        ),
    );
    assert!(marked.accepted);
    let after = marked.snapshot.expect("trimmed out");
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(layer.start, RationalTime::ZERO);
    assert_eq!(layer.duration, playhead);

    let outside_frame = RationalTime::try_from_frame(60, fps)
        .expect("frame 60")
        .try_to_frame_floor(fps)
        .expect("floor");
    assert!(dispatch_raw_json(host, &set_time_json(host, &outside_frame.to_string())).accepted);
    let outside = read_wire(host);
    assert!(
        outside
            .stage_geometry
            .layers
            .iter()
            .all(|layer| layer.layer_id != layer_id),
        "clip out before playhead must leave Stage"
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn split_then_set_time_drops_left_identity_and_shows_right() {
    let _lock = test_lock();
    let host = create_host("timeline-split-stage-geometry");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    seed_primary(host, target);

    let seed = read_wire(host);
    assert_eq!(seed.stage_geometry.layers.len(), 1);
    assert_eq!(seed.stage_geometry.layers[0].layer_id, layer_id);
    assert_eq!(seed.timeline.layers.len(), 1);

    let split = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"split","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(split.accepted);
    let after_split = read_wire(host);
    assert_eq!(after_split.timeline.layers.len(), 2);
    let right_id = after_split
        .timeline
        .layers
        .iter()
        .map(|layer| layer.layer_id.as_str())
        .find(|id| *id != layer_id)
        .expect("split allocates a new layer")
        .to_owned();
    assert_eq!(after_split.stage_geometry.layers.len(), 1);
    assert_eq!(after_split.stage_geometry.layers[0].layer_id, layer_id);

    // fps 30: frame 60 = 2s。左片は 0..1s なので playhead 外。右片の新 identity が見える。
    let moved = dispatch_raw_json(host, &set_time_json(host, "60"));
    assert!(moved.accepted);
    let outside = read_wire(host);
    assert_eq!(outside.current_time, RationalTime::from_seconds(2));
    assert!(
        outside
            .stage_geometry
            .layers
            .iter()
            .all(|layer| layer.layer_id != layer_id),
        "left split identity must leave Stage at playhead 2s"
    );
    assert!(
        outside
            .stage_geometry
            .layers
            .iter()
            .any(|layer| layer.layer_id == right_id),
        "right split identity must appear on Stage at playhead 2s"
    );

    let restored = dispatch_raw_json(host, &set_time_json(host, "0"));
    assert!(restored.accepted);
    let inside = read_wire(host);
    assert_eq!(inside.current_time, RationalTime::ZERO);
    assert_eq!(inside.stage_geometry.layers.len(), 1);
    assert_eq!(inside.stage_geometry.layers[0].layer_id, layer_id);
    let _ = host_destroy_for_test(host);
}

#[test]
fn select_layer_and_clear_selection_update_primary_layer_id() {
    let _lock = test_lock();
    let host = create_host("select-clear");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    assert!(baseline.primary_layer_id.is_none());

    let selected = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","#,
                r#""host_handle":"{host}","target":"{layer}"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(selected.accepted);
    assert_eq!(
        selected.snapshot.expect("selected").primary_layer_id,
        Some(layer_id.clone())
    );
    assert_eq!(read_snapshot(host).primary_layer_id, Some(layer_id));

    let cleared = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","host_handle":"{host}"}}"#
        ),
    );
    assert!(cleared.accepted);
    assert!(cleared
        .snapshot
        .expect("cleared")
        .primary_layer_id
        .is_none());
    assert!(read_snapshot(host).primary_layer_id.is_none());
    let _ = host_destroy_for_test(host);
}
