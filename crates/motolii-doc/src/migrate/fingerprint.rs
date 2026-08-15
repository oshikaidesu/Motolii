use std::collections::BTreeSet;

use motolii_core::{RationalTime, TimeMap};
use motolii_eval::DataTracks;

use crate::param::DocParam;
use crate::param_eval::{eval_doc_param, ResolvedLayerParams};
use crate::schema::{Clip, ClipSource, PathOp, TrackItem};
use crate::Document;

use super::SemanticFingerprint;

/// 移行後Documentから意味指紋を取る(S12自動比較用)。
pub fn semantic_fingerprint(doc: &Document, sample_times: &[RationalTime]) -> SemanticFingerprint {
    let tracks = DataTracks::new();
    let resolved = ResolvedLayerParams::default();
    let mut param_evals = Vec::new();
    let mut dependency_edges = BTreeSet::new();
    let mut timemap_samples = Vec::new();

    for track in &doc.tracks {
        for item in &track.items {
            collect_semantics(
                item,
                sample_times,
                &tracks,
                &resolved,
                &mut param_evals,
                &mut dependency_edges,
                &mut timemap_samples,
            );
        }
    }

    SemanticFingerprint {
        param_evals,
        dependency_edges,
        timemap_samples,
    }
}

fn collect_semantics(
    item: &TrackItem,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
    timemap_samples: &mut Vec<(u64, String, String)>,
) {
    match item {
        TrackItem::Clip(clip) => {
            collect_clip_semantics(
                clip,
                sample_times,
                tracks,
                resolved,
                param_evals,
                deps,
                timemap_samples,
            );
        }
        TrackItem::Group(group) => {
            collect_envelope_semantics(
                &group.envelope,
                sample_times,
                tracks,
                resolved,
                param_evals,
                deps,
            );
            for child in &group.children {
                collect_semantics(
                    child,
                    sample_times,
                    tracks,
                    resolved,
                    param_evals,
                    deps,
                    timemap_samples,
                );
            }
        }
    }
}

fn collect_clip_semantics(
    clip: &Clip,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
    timemap_samples: &mut Vec<(u64, String, String)>,
) {
    let layer = clip.envelope.layer_id.get();
    collect_envelope_semantics(
        &clip.envelope,
        sample_times,
        tracks,
        resolved,
        param_evals,
        deps,
    );
    for t in sample_times {
        if let Ok(src) = clip.time_map.try_map(*t) {
            timemap_samples.push((layer, format!("{t:?}"), format!("{src:?}")));
        }
    }
    if let ClipSource::Vector { recipe } = &clip.source {
        for (i, op) in recipe.modifiers.iter().enumerate() {
            let _ = i;
            collect_path_op_params(layer, op, sample_times, tracks, resolved, param_evals);
        }
    }
    if let ClipSource::Asset { audio, .. } = &clip.source {
        for (i, comp) in audio.iter().enumerate() {
            if let Some(name) = audio_gain_fingerprint_name(i) {
                for t in sample_times {
                    push_eval(param_evals, layer, name, &comp.gain, *t, tracks, resolved);
                }
            }
        }
    }
}

fn audio_gain_fingerprint_name(index: usize) -> Option<&'static str> {
    match index {
        0 => Some("audio[0].gain"),
        1 => Some("audio[1].gain"),
        2 => Some("audio[2].gain"),
        3 => Some("audio[3].gain"),
        _ => Some("audio[n].gain"),
    }
}

fn collect_envelope_semantics(
    env: &crate::schema::ItemEnvelope,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
) {
    let layer = env.layer_id.get();
    if let Some(parent) = env.transform.parent {
        deps.insert((layer, "parent", parent.get()));
    }
    collect_param_deps(layer, &env.transform.position, deps);
    collect_param_deps(layer, &env.transform.anchor, deps);
    collect_param_deps(layer, &env.transform.scale, deps);
    collect_param_deps(layer, &env.transform.rotation, deps);
    collect_param_deps(layer, &env.opacity, deps);

    for t in sample_times {
        push_eval(
            param_evals,
            layer,
            "opacity",
            &env.opacity,
            *t,
            tracks,
            resolved,
        );
        push_eval(
            param_evals,
            layer,
            "position",
            &env.transform.position,
            *t,
            tracks,
            resolved,
        );
        push_eval(
            param_evals,
            layer,
            "rotation",
            &env.transform.rotation,
            *t,
            tracks,
            resolved,
        );
    }
}

fn collect_param_deps(from: u64, param: &DocParam, deps: &mut BTreeSet<(u64, &'static str, u64)>) {
    match param {
        DocParam::LookAt { target, .. } => {
            deps.insert((from, "look_at", target.get()));
        }
        DocParam::Follow { target, .. } => {
            deps.insert((from, "follow", target.get()));
        }
        DocParam::Vec2Axes { x, y } => {
            collect_param_deps(from, x, deps);
            collect_param_deps(from, y, deps);
        }
        _ => {}
    }
}

fn collect_path_op_params(
    layer: u64,
    op: &PathOp,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
) {
    let params: &[(&str, &DocParam)] = match op {
        PathOp::PuckerBloat { amount } => &[("pucker_bloat.amount", amount)],
        PathOp::Offset { distance, .. } => &[("offset.distance", distance)],
        PathOp::Trim {
            start, end, offset, ..
        } => &[
            ("trim.start", start),
            ("trim.end", end),
            ("trim.offset", offset),
        ],
        PathOp::Twist { angle, center } => &[("twist.angle", angle), ("twist.center", center)],
        PathOp::RoundCorners { radius } => &[("round.radius", radius)],
        PathOp::ZigZag { amount, ridges, .. } => {
            &[("zigzag.amount", amount), ("zigzag.ridges", ridges)]
        }
        PathOp::Wiggle { amp, freq, .. } => &[("wiggle.amp", amp), ("wiggle.freq", freq)],
        PathOp::Repeater { copies, offset, .. } => {
            &[("repeater.copies", copies), ("repeater.offset", offset)]
        }
    };
    for (name, param) in params {
        for t in sample_times {
            push_eval(param_evals, layer, name, param, *t, tracks, resolved);
        }
    }
}

fn push_eval(
    out: &mut Vec<(u64, &'static str, String)>,
    layer: u64,
    name: &'static str,
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) {
    if let Ok(v) = eval_doc_param(param, t, tracks, resolved) {
        out.push((layer, name, format!("{v:?}")));
    }
}

/// 旧`timeline_start`付きTimeMapの写像を、現行契約(clip_local)で再現して比較する。
pub fn legacy_timemap_source(
    source_start: RationalTime,
    timeline_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
    timeline_time: RationalTime,
) -> Result<RationalTime, motolii_core::TimeMapError> {
    let delta = timeline_time.try_sub(timeline_start)?;
    let scaled = delta.try_mul_i64(speed_num)?;
    let unit = RationalTime::try_new(1, speed_den)?;
    let mapped = scaled.try_mul(unit)?;
    Ok(source_start.try_add(mapped)?)
}

/// `clip.start == timeline_start`のとき、現行TimeMapのclip_local写像と一致することの検査口。
pub fn modern_timemap_source(
    time_map: &TimeMap,
    clip_start: RationalTime,
    timeline_time: RationalTime,
) -> Result<RationalTime, motolii_core::TimeMapError> {
    let local = timeline_time.try_sub(clip_start)?;
    time_map.try_map(local)
}
