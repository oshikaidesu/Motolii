//! product_runtime分割後も、元fileを読むoracleは連結sourceを見る。

use super::*;
use crate::browser_host::BrowserPlaceIntent;
use crate::host_pointer_capture::HostPointerCancel;
use crate::inspector_host_runtime::InspectorPositionAxis;
use crate::native_host_layout::NativeHostLayout;
use crate::render_worker::RenderGeneration;
use crate::stage_chrome_host_runtime::StagePlaybackState;
use crate::timeline_trim_gesture::{TimelineTrimEdge, TimelineTrimGesture};
use crate::{
    builtin_command_registry, product_builtin_keymap, resolve_keymap, AsciiKey, CommandId,
    EffectiveTrigger, InputPhase, KeyToken, KeymapDelta, Modifier, Modifiers,
    PlatformBindingConstraints, PlatformCommandModifier,
};
use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use motolii_doc::{
    Command, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, EffectId, KeyframeId, LayerId,
    TrackItem,
};
use motolii_transport::Transport;

use super::app::*;
use super::browser::*;
use super::easing::*;
use super::error::*;
use super::inspector::*;
use super::place::*;
use super::place_overlay::*;
use super::playback::*;
use super::playhead::*;
use super::position::*;
use super::projection::*;
use super::publish::*;
use super::stage_transport::*;
use super::surface::*;
use super::timeline::*;

pub(super) const PRODUCTION_SOURCE: &str = concat!(
    include_str!("../app.rs"),
    include_str!("../browser.rs"),
    include_str!("../easing.rs"),
    include_str!("../error.rs"),
    include_str!("../inspector.rs"),
    include_str!("../place.rs"),
    include_str!("../place_overlay.rs"),
    include_str!("../playback.rs"),
    include_str!("../playhead.rs"),
    include_str!("../position.rs"),
    include_str!("../projection.rs"),
    include_str!("../publish.rs"),
    include_str!("../stage_transport.rs"),
    include_str!("../surface.rs"),
    include_str!("../timeline.rs"),
    include_str!("../window_input.rs"),
);

pub(super) fn position_keyframe_document() -> (motolii_doc::Document, LayerId, [KeyframeId; 2]) {
    let mut document = crate::static_preview::bootstrap_document().unwrap();
    let layer = match &document.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip.envelope.layer_id,
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    };
    let ids = [KeyframeId::from_raw(101), KeyframeId::from_raw(102)];
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: ids[0],
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: motolii_eval::Interp::Linear,
    });
    track.insert(DocKeyframe {
        id: ids[1],
        t: RationalTime::try_new(2, 1).unwrap(),
        value: DocValue::Vec2([1.0, 1.0]),
        interp: motolii_eval::Interp::Hold,
    });
    match &mut document.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            clip.envelope.transform.position = DocParam::Keyframes(track);
        }
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    (document, layer, ids)
}
pub(super) fn test_layout(epoch: u64) -> NativeHostLayout {
    test_layout_with(epoch, crate::layout::PanelLayout::built_in())
}

pub(super) fn test_layout_with(
    epoch: u64,
    authority: crate::layout::PanelLayout,
) -> NativeHostLayout {
    let frame =
        FrameDesc::try_packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true).unwrap();
    NativeHostLayout::try_new(epoch, 1000, 800, 1.0, frame, &authority)
        .unwrap()
        .unwrap()
}

pub(super) fn test_source() -> BrowserPlaceIntent {
    BrowserPlaceIntent {
        scope_ref: "builtin-stable".to_owned(),
        item_id: "rectangle".to_owned(),
    }
}

#[test]
fn stage_transport_production_lifecycle_has_only_the_admitted_publish_paths() {
    let source = PRODUCTION_SOURCE;
    let production = source.split("#[cfg(test)]").next().unwrap();
    let method = |name: &str| {
        let start = production.find(name).unwrap();
        let tail = &production[start..];
        let end = ["\n    fn ", "\n    pub(crate) fn "]
            .into_iter()
            .filter_map(|marker| tail[1..].find(marker).map(|end| end + 1))
            .min()
            .unwrap_or(tail.len());
        &tail[..end]
    };
    let free_fn = |name: &str| {
        let start = production.find(name).unwrap();
        let tail = &production[start..];
        &tail[..tail[1..]
            .find("\nfn ")
            .map(|end| end + 1)
            .unwrap_or(tail.len())]
    };

    assert!(production.contains("StageChromeHostRuntime::new(\n            &window,\n            &stage_transport_snapshot("));
    assert_eq!(
        production.matches("self.publish_stage_transport()").count(),
        9
    );
    assert!(method("fn refresh_editor_playhead").contains("self.publish_stage_transport()?"));
    assert!(method("fn publish_stage_transport")
        .contains("publish_stage_transport_snapshot_with_state("));
    assert!(!method("fn update_layout").contains("publish_stage_transport"));
    assert!(!method("fn update_layout").contains("refresh_editor_playhead"));
    let cancel = method("fn cancel_editor_playhead");
    assert!(cancel.contains(
        "if self.editor_playhead.scrub.is_none() {\n            return Ok(());\n        }"
    ));
    assert!(cancel.contains("if changed {\n            self.refresh_editor_playhead()?;"));
    let snapshot = free_fn("fn stage_transport_snapshot(");
    let publish = free_fn("fn publish_stage_transport_snapshot<");
    for body in [snapshot, publish] {
        for forbidden in [
            "journal",
            "history",
            "undo",
            "document_queue",
            "projection_generation",
        ] {
            assert!(!body.contains(forbidden));
        }
    }
    assert!(snapshot.contains("document: &motolii_doc::Document"));
    assert!(publish.contains("document: &motolii_doc::Document"));
}

#[test]
fn rn_product_snapshot_mouths_are_not_reimplemented_in_winit_runtime() {
    let production = PRODUCTION_SOURCE;
    assert!(!production.contains("WireProductSnapshot"));
    assert!(!production.contains("motolii_rn_host_read_snapshot_json"));
    assert!(!production.contains("motolii_rn_host_create"));
    assert!(production.contains("self.current_document = published.snapshot;"));
    let rn = include_str!("../../rn_product_host.rs");
    assert!(rn.contains("fn motolii_rn_host_create"));
    assert!(rn.contains("fn motolii_rn_host_read_snapshot_json"));
    assert!(rn.contains("fn motolii_rn_host_dispatch_intent_json"));
    assert!(rn.contains("fn accept_live_snapshot"));
}

#[test]
fn inspector_position_key_wake_route_resolves_current_primary_and_playhead_before_browser_poll() {
    let source = PRODUCTION_SOURCE;
    let production = source.split("#[cfg(test)]").next().unwrap();
    let method = |name: &str| {
        let start = production.find(name).unwrap();
        let tail = &production[start..];
        let end = ["\n    fn ", "\n    pub(crate) fn "]
            .into_iter()
            .filter_map(|marker| tail[1..].find(marker).map(|end| end + 1))
            .min()
            .unwrap_or(tail.len());
        &tail[..end]
    };

    let wake = method("pub(crate) fn handle_product_event");
    let position_route = method("fn process_inspector_position_key_intents");
    assert!(
        wake.find("self.process_inspector_position_key_intents(event_loop)")
            < wake.find("self.poll_browser(event_loop)")
    );
    for required in [
        "let Some(target) = self.primary else",
        "time: self.editor_playhead.current",
        "self.document_queue.push_add_position_key(request)",
        "self.document_runtime.process_next(",
        "self.adopt_full_publish(event_loop, published, \"inspector-add-position-key\")",
    ] {
        assert!(position_route.contains(required), "{required}");
    }
    for forbidden in ["RationalTime::ZERO", "mock", "postMessage", "Interp"] {
        assert!(!position_route.contains(forbidden), "{forbidden}");
    }
}

mod easing;
mod place;
mod playback;
mod timeline;
