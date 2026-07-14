//! AG-3: import時のcomponent選択と音声分離コマンドの構築。

use std::collections::BTreeMap;

use crate::{
    AudioComponent, Clip, ClipSource, Command, CommandError, Document, ItemEnvelope, LayerId,
    ParentLocator, TrackItem, VideoComponent,
};

/// AV import時に作るAsset Clip componentの組合せ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportAvMode {
    VideoOnly,
    VideoAndAudio {
        video_ordinal: u32,
        audio_ordinal: u32,
    },
}

/// import選択からAsset Clip sourceを構築する。
pub fn build_import_clip_source(asset: crate::AssetId, mode: ImportAvMode) -> ClipSource {
    match mode {
        ImportAvMode::VideoOnly => ClipSource::asset_video_only(asset),
        ImportAvMode::VideoAndAudio {
            video_ordinal,
            audio_ordinal,
        } => ClipSource::Asset {
            asset,
            video: Some(VideoComponent::ordinal(video_ordinal)),
            audio: vec![AudioComponent::ordinal(audio_ordinal)],
        },
    }
}

/// 元クリップの直後へaudio-only Clipを追加し、元の有効なaudio componentを無効化する。
pub fn plan_detach_audio(
    doc: &Document,
    parent: ParentLocator,
    clip_index: usize,
    new_layer: LayerId,
    new_layer_name: &str,
) -> Result<Vec<Command>, CommandError> {
    let items = items_at(doc, parent)?;
    let item = items.get(clip_index).ok_or(CommandError::IndexOutOfRange {
        index: clip_index,
        len: items.len(),
    })?;
    let TrackItem::Clip(original) = item else {
        return Err(CommandError::AudioComponentNotFound {
            layer: envelope_layer(item),
            ordinal: 0,
        });
    };
    let ClipSource::Asset {
        asset,
        audio: original_audio,
        ..
    } = &original.source
    else {
        return Err(CommandError::AudioComponentNotFound {
            layer: original.envelope.layer_id.get(),
            ordinal: 0,
        });
    };

    let enabled_ordinals: Vec<u32> = original_audio
        .iter()
        .filter(|component| component.enabled)
        .map(|component| component.stream.ordinal)
        .collect();
    let first_ordinal = enabled_ordinals.first().copied().unwrap_or(0);
    if enabled_ordinals.is_empty() {
        return Err(CommandError::AudioComponentNotFound {
            layer: original.envelope.layer_id.get(),
            ordinal: first_ordinal,
        });
    }

    let detached = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(new_layer),
        start: original.start,
        duration: original.duration,
        time_map: original.time_map,
        source: ClipSource::Asset {
            asset: *asset,
            video: None,
            audio: original_audio.clone(),
        },
    });
    let mut commands = vec![Command::AddTrackItem {
        parent,
        index: clip_index + 1,
        item: detached,
        layer_names: BTreeMap::from([(new_layer, new_layer_name.to_string())]),
    }];
    commands.extend(enabled_ordinals.into_iter().map(|ordinal| {
        Command::SetAudioComponentEnabled {
            target: original.envelope.layer_id,
            ordinal,
            old: true,
            new: false,
        }
    }));
    Ok(commands)
}

fn items_at(doc: &Document, parent: ParentLocator) -> Result<&[TrackItem], CommandError> {
    match parent {
        ParentLocator::Track(track_id) => doc
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .map(|track| track.items.as_slice())
            .ok_or(CommandError::TrackNotFound(track_id.get())),
        ParentLocator::Group(layer) => doc
            .tracks
            .iter()
            .find_map(|track| group_children(&track.items, layer))
            .ok_or(CommandError::GroupNotFound(layer.get())),
    }
}

fn group_children(items: &[TrackItem], target: LayerId) -> Option<&[TrackItem]> {
    for item in items {
        if let TrackItem::Group(group) = item {
            if group.envelope.layer_id == target {
                return Some(&group.children);
            }
            if let Some(children) = group_children(&group.children, target) {
                return Some(children);
            }
        }
    }
    None
}

fn envelope_layer(item: &TrackItem) -> u64 {
    match item {
        TrackItem::Clip(clip) => clip.envelope.layer_id.get(),
        TrackItem::Group(group) => group.envelope.layer_id.get(),
    }
}
