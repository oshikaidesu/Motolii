//! attach_effect / set_effect_param / opacity の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn attach_effect_rejects_when_target_is_not_primary() {
    let _lock = test_lock();
    let host = create_host("catalog-target-mismatch");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    let before = read_snapshot(host);
    let before_wire = read_wire(host);
    let before_effect_count = layer_effects(&before_wire, &layer_id).len();

    let mismatch_target = if layer_id == "1" { "2" } else { "1" };
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{target}","plugin_id":"core.filter.opacity"}}"#,
            ),
            host = host,
            target = mismatch_target,
        ),
    );
    assert!(!response.accepted);
    assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));

    let after = read_snapshot(host);
    assert_eq!(after.primary_layer_id, before.primary_layer_id);
    assert_eq!(after.projection_generation, before.projection_generation);
    assert_eq!(after.revision, before.revision);
    assert_eq!(
        layer_effects(&read_wire(host), &layer_id).len(),
        before_effect_count
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn attach_effect_grows_layer_effects_and_undo_clears() {
    let _lock = test_lock();
    let host = create_host("attach-effect");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(layer_effects(&read_wire(host), &layer_id).is_empty());

    let attached = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(attached.accepted);
    let after = read_wire(host);
    let effects = layer_effects(&after, &layer_id);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].plugin_id, "core.filter.opacity");
    assert_eq!(effects[0].params.len(), 1);
    assert_eq!(effects[0].params[0].param_id, "amount");
    assert_eq!(effects[0].params[0].value, 1.0);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    assert!(undone.accepted);
    assert!(layer_effects(&read_wire(host), &layer_id).is_empty());
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_effect_param_updates_value_preserves_others_and_undo_restores() {
    let _lock = test_lock();
    let host = create_host("set-effect-param");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    // 第二 effect を足して他 effect 不変を検証する。
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let before = read_wire(host);
    let effects_before = layer_effects(&before, &layer_id);
    assert_eq!(effects_before.len(), 2);
    let first_id = effects_before[0].effect_use_id.clone();
    let second_id = effects_before[1].effect_use_id.clone();
    let second_amount = effects_before[1].params[0].value;

    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                r#""host_handle":"{host}","target":"{layer}","effect_use_id":"{use_id}","#,
                r#""param_id":"amount","value":0.4}}"#
            ),
            host = host,
            layer = layer_id,
            use_id = first_id,
        ),
    );
    assert!(changed.accepted);
    let after = read_wire(host);
    let effects = layer_effects(&after, &layer_id);
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0].effect_use_id, first_id);
    assert_eq!(effects[0].params[0].value, 0.4);
    assert_eq!(effects[1].effect_use_id, second_id);
    assert_eq!(effects[1].params[0].value, second_amount);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    assert!(undone.accepted);
    let restored_wire = read_wire(host);
    let restored = layer_effects(&restored_wire, &layer_id);
    assert_eq!(restored[0].params[0].value, 1.0);
    assert_eq!(restored[1].params[0].value, second_amount);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_effect_param_updates_value_in_read_snapshot_json_selected_not_unselected_bloom() {
    let _lock = test_lock();
    let host = create_host("inspector-selected-params-json");
    let baseline = read_wire(host);
    assert!(baseline.primary_layer_id.is_none());
    let unselected = read_snapshot_json(host);
    let unselected_text = unselected.to_string();
    assert!(
        !unselected_text.contains("Echo Bloom"),
        "unselected snapshot must not invent Echo Bloom"
    );
    assert!(unselected.get("primary_layer_id").is_none());
    assert!(unselected.get("selected_doc_params").is_none());
    assert!(unselected.get("nodes").is_none());
    assert!(unselected.get("active_effect_use_id").is_none());
    assert!(inspector_selected_effects(&unselected).is_empty());
    for layer in unselected["timeline"]["layers"]
        .as_array()
        .expect("timeline layers")
    {
        assert!(layer["effects"].as_array().expect("effects").is_empty());
    }
    assert!(
        unselected["catalog"]["effects"]
            .as_array()
            .expect("catalog")
            .iter()
            .any(|effect| effect["plugin_id"] == "core.filter.opacity"),
        "catalog may list opacity; Inspector must not treat it as a selected effect"
    );

    let layer_id = baseline.timeline.layers[0].layer_id.clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let effect_use_id = layer_effects(&read_wire(host), &layer_id)[0]
        .effect_use_id
        .clone();
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                    r#""host_handle":"{host}","target":"{layer}","effect_use_id":"{use_id}","#,
                    r#""param_id":"amount","value":0.4}}"#
                ),
                host = host,
                layer = layer_id,
                use_id = effect_use_id,
            ),
        )
        .accepted
    );

    let selected = read_snapshot_json(host);
    assert_eq!(selected["primary_layer_id"], layer_id);
    let selected_effects = inspector_selected_effects(&selected);
    assert_eq!(selected_effects.len(), 1);
    assert_eq!(selected_effects[0]["plugin_id"], "core.filter.opacity");
    assert_eq!(selected_effects[0]["effect_use_id"], effect_use_id);
    let amount = selected_effects[0]["params"]
        .as_array()
        .expect("params")
        .iter()
        .find(|param| param["param_id"] == "amount")
        .expect("amount");
    assert_eq!(amount["value"], 0.4);
    let selected_params = selected
        .get("selected_doc_params")
        .expect("selected_doc_params");
    assert_eq!(selected_params["layer_id"], layer_id);
    assert_eq!(selected_params["opacity"], 1.0);
    assert!(selected_params.get("position").is_none());
    assert!(selected_params.get("rotation").is_none());
    assert!(selected_params.get("scale").is_none());
    assert_eq!(
        selected_params["effects"],
        serde_json::Value::Array(selected_effects.clone())
    );
    assert!(!selected.to_string().contains("Echo Bloom"));

    assert!(dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","host_handle":"{host}"}}"#
            ),
        )
        .accepted);
    let cleared = read_snapshot_json(host);
    assert!(cleared.get("primary_layer_id").is_none());
    assert!(cleared.get("selected_doc_params").is_none());
    assert!(inspector_selected_effects(&cleared).is_empty());
    assert!(!cleared.to_string().contains("Echo Bloom"));
    let cleared_wire = read_wire(host);
    let live = layer_effects(&cleared_wire, &layer_id);
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].plugin_id, "core.filter.opacity");
    assert_eq!(live[0].params[0].param_id, "amount");
    assert_eq!(live[0].params[0].value, 0.4);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_opacity_updates_document_opacity_in_snapshot() {
    let _lock = test_lock();
    let host = create_host("set-opacity");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);

    let before = read_wire(host);
    assert_eq!(
        before
            .selected_doc_params
            .as_ref()
            .expect("selected_doc_params")
            .opacity,
        Some(1.0)
    );

    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_opacity","#,
                r#""host_handle":"{host}","target":"{layer}","value":0.4}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(changed.accepted);

    let after = read_wire(host);
    assert_eq!(
        after
            .selected_doc_params
            .as_ref()
            .expect("selected_doc_params")
            .opacity,
        Some(0.4)
    );
    let live = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(const_f64_param(
            &find_envelope_in_document(product.runtime.snapshot().as_ref(), target)
                .expect("envelope")
                .opacity,
        ))
    })
    .expect("lookup");
    assert_eq!(live, Some(0.4));
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_opacity_on_keyframes_at_playhead_writes_without_collapsing() {
    let _lock = test_lock();
    let host = create_host("set-opacity-keyframes");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_param_key","#,
                    r#""host_handle":"{host}","target":"{layer}","property":"opacity","#,
                    r#""time":{{"num":0,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );
    let keyed = read_wire(host);
    let layer = keyed
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(layer.param_keys.len(), 1);
    assert_eq!(layer.param_keys[0].property, "opacity");
    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_opacity","#,
                r#""host_handle":"{host}","target":"{layer}","value":0.25}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(changed.accepted, "reason={:?}", changed.reason);
    let after = read_wire(host);
    assert_eq!(
        after
            .selected_doc_params
            .as_ref()
            .expect("selected_doc_params")
            .opacity,
        Some(0.25)
    );
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(layer.param_keys.len(), 1);
    assert_eq!(layer.param_keys[0].property, "opacity");
    assert_eq!(layer.param_keys[0].value, Some(0.25));
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_effect_param_color_without_value_writes() {
    let _lock = test_lock();
    let host = create_host("set-effect-color");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.tint"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );
    let attached = read_wire(host);
    let effect_use_id = layer_effects(&attached, &layer_id)[0].effect_use_id.clone();
    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                r#""host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","#,
                r#""param_id":"color","color":[0.2,1.0,1.0,1.0]}}"#
            ),
            host = host,
            layer = layer_id,
            effect = effect_use_id,
        ),
    );
    assert!(changed.accepted, "reason={:?}", changed.reason);
    let after = read_wire(host);
    let color = layer_effects(&after, &layer_id)[0]
        .params
        .iter()
        .find(|param| param.param_id == "color")
        .expect("color");
    assert_eq!(color.color, Some([0.2, 1.0, 1.0, 1.0]));
    let _ = host_destroy_for_test(host);
}

#[test]
fn effect_intents_reject_absent_target_plugin_param_and_non_finite_without_mutation() {
    let _lock = test_lock();
    let host = create_host("effect-rejects");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let before = serde_json::to_vec(&read_wire(host)).expect("before json");
    let before_wire = read_wire(host);
    let use_id = layer_effects(&before_wire, &layer_id)[0]
        .effect_use_id
        .clone();

    let missing_target = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","plugin_id":"core.filter.opacity"}}"#
            ),
            host = host,
        ),
    );
    assert!(!missing_target.accepted);

    let missing_plugin = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.missing"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!missing_plugin.accepted);

    let missing_param = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                r#""host_handle":"{host}","target":"{layer}","effect_use_id":"{use_id}","#,
                r#""param_id":"nope","value":0.2}}"#
            ),
            host = host,
            layer = layer_id,
            use_id = use_id,
        ),
    );
    assert!(!missing_param.accepted);

    let non_finite = dispatch_wire(
        host,
        WireIntentEnvelope {
            version: 1,
            direction: RN_TO_HOST.to_owned(),
            kind: "set_effect_param".into(),
            host_handle: host.to_string(),
            stage_handle: None,
            projection_generation: None,
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
            phase: None,
            view_local_x: None,
            view_local_y: None,
            sequence: None,
            frame: None,
            position: None,
            playhead: None,
            target: Some(layer_id.clone()),
            dest: None,
            key_id: None,
            property: None,
            time: None,
            new: None,
            interp: None,
            delta: None,
            plugin_id: None,
            item_id: None,
            effect_use_id: Some(use_id),
            param_id: Some("amount".into()),
            value: Some(f64::NAN),
            output_path: None,
            color: None,
        },
    );
    assert!(!non_finite.accepted);

    let after = serde_json::to_vec(&read_wire(host)).expect("after json");
    assert_eq!(after, before);
    let _ = host_destroy_for_test(host);
}
