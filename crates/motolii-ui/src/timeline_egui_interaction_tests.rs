//! Skia/Godot由来のTimeline操作を、現行Motolii契約へ照合するテスト。
//!
//! ここで確認するのは、現時点で実在する契約だけである。
//! `TimelineProjection::hit_test`、eguiのshortcut変換、既存の
//! `DomainIntent`/`InputRouter`、および trim/move の egui hit 分類
//! (`classify_bar_edge`) を検査する。
//! drag の Document commit は `app.rs` が所有し、ここではgreenにしない。
//!
//! 対応表:
//! - key/clip/空白 hit-test -> `TimelineHit`
//! - bar edge の Left/Right/Body 分類 -> `EguiTimelineHit` / `classify_bar_edge`
//! - Escape/Delete/Backspace/Cmd+Z/Cmd+Shift+Z -> `TimelineCommand`
//! - Cmd+C/X/V/D/A -> existing product host-kind keymap
//! - unknown intent/command -> typed rejection
//! - marquee、複数key編集、param-key copy/pasteは未接続として検査対象外
//!
//! no-move release の enqueue は `TimelineMoveGesture` /
//! `TimelineTrimGesture` の既存テスト (`same_pointer_release_is_a_noop` /
//! `same_value_release_is_a_noop`) が所有する。本ファイルから
//! `DocumentEditQueue` を組み立てて偽の queue 試験はしない。

use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, ItemEnvelope,
    KeyframeId, LayerId, Track, TrackItem, Transform2D,
};
use motolii_eval::Interp;

use crate::timeline_egui::{
    classify_bar_edge, timeline_command_for_key, EguiTimelineHit, TimelineCommand,
};
use crate::timeline_projection::TimelineHit;
use crate::{
    builtin_command_registry, product_action_host_kind, resolve_product_action, AsciiKey,
    CommandId, DomainIntent, EffectiveTrigger, InputPhase, InputRouter, InputRouterError, KeyToken,
    Modifiers, NormalizedInput, PlatformCommandModifier, ProductAction,
};

fn time(seconds: i64) -> RationalTime {
    RationalTime::try_new(seconds, 1).expect("fixture time")
}

fn fixture_with_key() -> (Document, motolii_doc::LayerId) {
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("V1").expect("track id");
    let asset = document
        .assets
        .allocate("timeline-test", "video/mp4", "timeline-test-hash")
        .expect("asset id");
    let layer = document
        .layers
        .allocate("Timeline layer")
        .expect("layer id");
    let key_id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id"));
    let mut keys = DocKeyframeTrack::new();
    keys.insert(DocKeyframe {
        id: key_id,
        t: time(1),
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Hold,
    });

    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform = Transform2D {
        position: DocParam::Keyframes(keys),
        ..Transform2D::identity()
    };
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: time(0),
            duration: time(3),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    document.validate().expect("valid timeline fixture");
    (document, layer)
}

fn trigger(key: KeyToken, modifiers: Modifiers) -> EffectiveTrigger {
    EffectiveTrigger::Keyboard {
        key,
        modifiers,
        phase: InputPhase::Press,
    }
}

fn meta() -> Modifiers {
    Modifiers::try_new([crate::Modifier::Meta]).expect("Meta modifier")
}

#[test]
fn hit_test_rejects_gap_and_outside_surface() {
    let (document, layer) = fixture_with_key();
    let projection = crate::project_timeline(
        &document,
        &crate::TimelineMetrics {
            band_height: 20.0,
            units_per_second: 100.0,
            key_half_extent: 8.0,
        },
        &crate::TimelineViewport {
            start: time(0),
            end: time(10),
        },
    )
    .expect("timeline projection");
    let bar = projection
        .bars()
        .iter()
        .find(|bar| bar.layer == layer)
        .expect("fixture bar");

    assert_eq!(
        projection.hit_test(bar.x_end, bar.y_top + 1.0),
        TimelineHit::None
    );
    assert_eq!(
        projection.hit_test(bar.x_start + 1.0, bar.y_bottom),
        TimelineHit::None
    );
    assert_eq!(projection.hit_test(f64::NAN, 0.5), TimelineHit::None);
}

#[test]
fn hit_test_prefers_key_and_does_not_expand_to_nearby_pixels() {
    let (document, layer) = fixture_with_key();
    let projection = crate::project_timeline(
        &document,
        &crate::TimelineMetrics {
            band_height: 20.0,
            units_per_second: 100.0,
            key_half_extent: 8.0,
        },
        &crate::TimelineViewport {
            start: time(0),
            end: time(10),
        },
    )
    .expect("timeline projection");
    let key = projection
        .keys()
        .iter()
        .find(|key| key.layer == layer)
        .expect("fixture key");

    assert_eq!(
        projection.hit_test(key.center_x, key.center_y),
        TimelineHit::Key {
            layer,
            key: key.key,
        }
    );
    assert_eq!(
        projection.hit_test(key.center_x + 9.0, key.center_y),
        TimelineHit::Bar { layer },
        "outside the key diamond but inside the clip remains a bar hit"
    );
}

#[test]
fn egui_shortcut_mapping_rejects_wrong_modifiers() {
    let none = egui::Modifiers::default();
    let command = egui::Modifiers {
        command: true,
        ..none
    };
    let shift = egui::Modifiers {
        shift: true,
        ..none
    };

    assert_eq!(
        timeline_command_for_key(egui::Key::Escape, none),
        Some(TimelineCommand::Escape)
    );
    assert_eq!(
        timeline_command_for_key(egui::Key::Delete, none),
        Some(TimelineCommand::Delete)
    );
    assert_eq!(
        timeline_command_for_key(egui::Key::Z, command),
        Some(TimelineCommand::Undo)
    );
    assert_eq!(
        timeline_command_for_key(
            egui::Key::Z,
            egui::Modifiers {
                command: true,
                shift: true,
                ..none
            },
        ),
        Some(TimelineCommand::Redo)
    );
    assert_eq!(timeline_command_for_key(egui::Key::Escape, shift), None);
    assert_eq!(timeline_command_for_key(egui::Key::Delete, command), None);
    assert_eq!(timeline_command_for_key(egui::Key::Z, none), None);
}

#[test]
fn product_shortcuts_reject_unbound_timeline_keys() {
    let registry = builtin_command_registry().expect("builtin registry");
    let delta = crate::KeymapDelta::default();
    let copy = resolve_product_action(
        &trigger(KeyToken::Ascii(AsciiKey::try_new('c').unwrap()), meta()),
        &registry,
        &delta,
        PlatformCommandModifier::Meta,
    )
    .expect("existing product copy mapping");
    assert_eq!(
        product_action_host_kind(&copy),
        Some(crate::PRODUCT_HOST_KIND_COPY)
    );
    assert!(matches!(copy, ProductAction::HostKind(_)));

    assert_eq!(
        resolve_product_action(
            &trigger(
                KeyToken::Ascii(AsciiKey::try_new('c').unwrap()),
                Modifiers::default(),
            ),
            &registry,
            &delta,
            PlatformCommandModifier::Meta,
        ),
        None,
        "copy without the platform command modifier is not a shortcut"
    );
    assert_eq!(
        resolve_product_action(
            &trigger(KeyToken::Ascii(AsciiKey::try_new('b').unwrap()), meta(),),
            &registry,
            &delta,
            PlatformCommandModifier::Meta,
        ),
        None,
        "an unregistered timeline shortcut must remain unconnected"
    );
}

#[test]
fn unknown_intent_command_is_rejected_instead_of_reaching_document() {
    assert_eq!(
        DomainIntent::try_from_adapter_kind(u16::MAX),
        Err(crate::DomainIntentError::UnknownAdapterKind { got: u16::MAX })
    );

    let mut router = InputRouter::new(builtin_command_registry().expect("builtin registry"));
    let unknown = CommandId::try_new("motolii.timeline.unwired_edit").expect("fixture command");
    assert_eq!(
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Press,
                id: unknown.clone(),
            })
            .expect_err("unknown command must be typed rejection"),
        InputRouterError::UnknownCommandId { id: unknown }
    );
}

fn fixture_layer() -> LayerId {
    LayerId::from_raw(1)
}

#[test]
fn classify_bar_edge_body_is_not_trim_for_wide_bar() {
    let layer = fixture_layer();
    // 100px bar: edge_width = 15.min(25) = 15。中央は Body であり trim ではない。
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 0.25, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
}

#[test]
fn classify_bar_edge_left_and_right_on_wide_bar() {
    let layer = fixture_layer();
    // 100px bar: Left if local_x <= 15, Right if local_x >= 85。
    // 境界ぴったりは f64/f32 変換で Body に落ちるので、端の内側を f32 で与える。
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 0.0, 200.0, layer),
        EguiTimelineHit::Left { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 10.0_f32 / 200.0, 200.0, layer),
        EguiTimelineHit::Left { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 0.5, 200.0, layer),
        EguiTimelineHit::Right { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 90.0_f32 / 200.0, 200.0, layer),
        EguiTimelineHit::Right { layer }
    );
}

#[test]
fn classify_bar_edge_narrow_bar_is_always_body() {
    let layer = fixture_layer();
    // 20px < 25: 端を含めて全幅が Body。trim にしない。
    assert_eq!(
        classify_bar_edge(0.0, 0.1, 0.0, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.1, 0.05, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.1, 0.1, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
}

#[test]
fn classify_bar_edge_rejects_non_finite_as_body() {
    let layer = fixture_layer();
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 0.0, f32::NAN, layer),
        EguiTimelineHit::Body { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.5, 0.0, f32::INFINITY, layer),
        EguiTimelineHit::Body { layer }
    );
    assert_eq!(
        classify_bar_edge(f32::NAN, 0.5, 0.25, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
    assert_eq!(
        classify_bar_edge(0.0, 0.5, f32::NAN, 200.0, layer),
        EguiTimelineHit::Body { layer }
    );
}
