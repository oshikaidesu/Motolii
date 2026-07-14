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

/// 元Clipのaudioを別laneへ分離するcommand列を組み立てる。
///
/// A4 / 設計§4.4: 同一Track(または同一Group)内への時間重なり配置は禁止。
/// 呼び出し側は必ず `destination_parent != source_parent` の別laneを渡す。
pub fn plan_detach_audio(
    doc: &Document,
    source_parent: ParentLocator,
    clip_index: usize,
    destination_parent: ParentLocator,
    destination_index: usize,
    new_layer: LayerId,
    new_layer_name: &str,
) -> Result<Vec<Command>, CommandError> {
    if source_parent == destination_parent {
        return Err(CommandError::DetachSameLane);
    }

    let items = items_at(doc, source_parent)?;
    let item = items.get(clip_index).ok_or(CommandError::IndexOutOfRange {
        index: clip_index,
        len: items.len(),
    })?;
    let TrackItem::Clip(original) = item else {
        return Err(CommandError::AudioComponentNotFound {
            layer: envelope_layer(item),
            index: 0,
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
            index: 0,
        });
    };

    let enabled_indices: Vec<usize> = original_audio
        .iter()
        .enumerate()
        .filter(|(_, component)| component.enabled)
        .map(|(index, _)| index)
        .collect();
    if enabled_indices.is_empty() {
        return Err(CommandError::AudioComponentNotFound {
            layer: original.envelope.layer_id.get(),
            index: 0,
        });
    }

    // 挿入先の境界を先に検査(失敗時に部分commandを返さない)。
    let dest_len = items_at(doc, destination_parent)?.len();
    if destination_index > dest_len {
        return Err(CommandError::IndexOutOfRange {
            index: destination_index,
            len: dest_len,
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
        parent: destination_parent,
        index: destination_index,
        item: detached,
        layer_names: BTreeMap::from([(new_layer, new_layer_name.to_string())]),
    }];
    commands.extend(enabled_indices.into_iter().map(|index| {
        Command::SetAudioComponentEnabled {
            target: original.envelope.layer_id,
            index,
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
