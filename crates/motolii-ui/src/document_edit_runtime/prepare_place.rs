//! 配置requestを既存Add/Admit commandへ落とす。queueは増やさない。

use std::collections::BTreeMap;
use std::fs;

use motolii_core::RationalTime;
use motolii_doc::{
    build_import_clip_source, Asset, AssetId, AudioComponent, Clip, ClipSource, Command, DocParam,
    Document, DocumentPluginError, ImportAvMode, ItemEnvelope, LayerId, ParentLocator,
    SourceFingerprintV1, StandardShape, TrackItem, VectorContent, VectorRecipe,
};
use motolii_plugin::{PluginCatalog, PluginKind};

use super::error::DocumentEditRuntimeError;
use super::requests::{PlaceMediaRequest, PlaceRectangleRequest, PlaceVismRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VectorShapeKind {
    Rectangle,
    Ellipse,
}

impl VectorShapeKind {
    /// 既定サイズは正準座標(高さ1.0)基準の0.2角。Rectangleと同値にして初期見えを揃える。
    fn standard_shape(self) -> StandardShape {
        let width = DocParam::const_f64(0.2);
        let height = DocParam::const_f64(0.2);
        match self {
            Self::Rectangle => StandardShape::Rect { width, height },
            Self::Ellipse => StandardShape::Ellipse { width, height },
        }
    }

    const fn layer_name(self) -> &'static str {
        match self {
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
        }
    }
}

pub(super) fn prepare_vector_shape_command(
    snapshot: &Document,
    current_primary: Option<LayerId>,
    request: PlaceRectangleRequest,
    shape: VectorShapeKind,
) -> Result<(Command, LayerId, u64), DocumentEditRuntimeError> {
    if !request.position[0].is_finite() || !request.position[1].is_finite() {
        return Err(DocumentEditRuntimeError::NonFiniteDropPosition);
    }
    if request.playhead < RationalTime::ZERO {
        return Err(DocumentEditRuntimeError::PlayheadOutsideComposition);
    }
    let duration = snapshot.composition.duration.try_sub(request.playhead)?;
    if duration < snapshot.composition.fps.frame_duration() {
        return Err(DocumentEditRuntimeError::RemainingDurationBelowOneFrame);
    }

    let mut layers = snapshot.layers.clone();
    let expected_live_next = layers.peek_next();
    let layer_id = layers.reserve()?;
    let (track_id, index) = rectangle_insertion(snapshot, current_primary)
        .ok_or(DocumentEditRuntimeError::NoTrackForRectangle)?;

    let mut envelope = ItemEnvelope::new(layer_id);
    envelope.transform.position = motolii_doc::DocParam::const_vec2(request.position);
    let item = TrackItem::Clip(Clip {
        envelope,
        start: request.playhead,
        duration,
        time_map: Default::default(),
        source: ClipSource::Vector {
            recipe: VectorRecipe {
                content: VectorContent::StandardShape {
                    shape: shape.standard_shape(),
                },
                modifiers: Vec::new(),
            },
        },
    });
    Ok((
        Command::AddTrackItem {
            parent: ParentLocator::Track(track_id),
            index,
            item,
            layer_names: BTreeMap::from([(layer_id, shape.layer_name().to_owned())]),
        },
        layer_id,
        expected_live_next,
    ))
}

pub(super) fn prepare_vism_command(
    snapshot: &Document,
    catalog: &PluginCatalog,
    current_primary: Option<LayerId>,
    request: PlaceVismRequest,
) -> Result<(Command, LayerId, u64), DocumentEditRuntimeError> {
    if !request.position[0].is_finite() || !request.position[1].is_finite() {
        return Err(DocumentEditRuntimeError::NonFiniteDropPosition);
    }
    if request.playhead < RationalTime::ZERO {
        return Err(DocumentEditRuntimeError::PlayheadOutsideComposition);
    }
    let duration = snapshot.composition.duration.try_sub(request.playhead)?;
    if duration < snapshot.composition.fps.frame_duration() {
        return Err(DocumentEditRuntimeError::RemainingDurationBelowOneFrame);
    }

    let current_version = catalog
        .get(&request.plugin_id)
        .ok_or_else(|| DocumentPluginError::ContractMissing {
            plugin_id: request.plugin_id.clone(),
        })?
        .node
        .version;
    let recipe = motolii_doc::prepare_plugin_recipe(
        &request.plugin_id,
        PluginKind::LayerSource,
        current_version,
        &BTreeMap::new(),
        catalog,
    )?;
    let layer_name = catalog
        .get(&request.plugin_id)
        .map(|contract| {
            if contract.node.display_name.trim().is_empty() {
                request.plugin_id.clone()
            } else {
                contract.node.display_name.to_owned()
            }
        })
        .unwrap_or_else(|| request.plugin_id.clone());

    let mut layers = snapshot.layers.clone();
    let expected_live_next = layers.peek_next();
    let layer_id = layers.reserve()?;
    let (track_id, index) = rectangle_insertion(snapshot, current_primary)
        .ok_or(DocumentEditRuntimeError::NoTrackForRectangle)?;

    let mut envelope = ItemEnvelope::new(layer_id);
    envelope.transform.position = motolii_doc::DocParam::const_vec2(request.position);
    let item = TrackItem::Clip(Clip {
        envelope,
        start: request.playhead,
        duration,
        time_map: Default::default(),
        source: ClipSource::Plugin {
            plugin_id: recipe.plugin_id,
            effect_version: recipe.current_version,
            params: recipe.params,
            extra: Default::default(),
        },
    });
    Ok((
        Command::AddTrackItem {
            parent: ParentLocator::Track(track_id),
            index,
            item,
            layer_names: BTreeMap::from([(layer_id, layer_name)]),
        },
        layer_id,
        expected_live_next,
    ))
}

pub(super) fn prepare_media_commands(
    snapshot: &Document,
    current_primary: Option<LayerId>,
    request: PlaceMediaRequest,
) -> Result<(Vec<Command>, LayerId, u64), DocumentEditRuntimeError> {
    if !request.position[0].is_finite() || !request.position[1].is_finite() {
        return Err(DocumentEditRuntimeError::NonFiniteDropPosition);
    }
    if request.playhead < RationalTime::ZERO {
        return Err(DocumentEditRuntimeError::PlayheadOutsideComposition);
    }
    let duration = snapshot.composition.duration.try_sub(request.playhead)?;
    if duration < snapshot.composition.fps.frame_duration() {
        return Err(DocumentEditRuntimeError::RemainingDurationBelowOneFrame);
    }
    let canonical = request
        .path
        .canonicalize()
        .map_err(|_| DocumentEditRuntimeError::LibraryFileUnreadable)?;
    if !canonical.is_file() {
        return Err(DocumentEditRuntimeError::LibraryFileUnreadable);
    }
    let abs = Asset::normalize_path(&canonical.to_string_lossy());
    let file =
        fs::File::open(&canonical).map_err(|_| DocumentEditRuntimeError::LibraryFileUnreadable)?;
    let fingerprint = SourceFingerprintV1::from_reader(file)
        .map_err(|_| DocumentEditRuntimeError::LibraryFileUnreadable)?;

    let mut commands = Vec::new();
    let asset_id = if let Some(existing) = existing_asset_for_path(snapshot, &abs) {
        existing
    } else {
        let asset = Asset {
            id: AssetId::from_raw(snapshot.assets.peek_next()),
            name: request.name.clone(),
            asset_type: request.asset_type.clone(),
            content_hash: fingerprint.content_hash(),
            path_absolute: Some(abs),
            path_project_relative: None,
            file_name: Some(request.name.clone()),
            size_bytes: Some(fingerprint.size_bytes()),
            head_hash: None,
            tail_hash: None,
        };
        let id = asset.id;
        commands.push(Command::AdmitAsset { asset });
        id
    };

    let source = match request.kind.as_str() {
        "audio" => ClipSource::Asset {
            asset: asset_id,
            video: None,
            audio: vec![AudioComponent::ordinal(0)],
        },
        "video" | "image" => build_import_clip_source(asset_id, ImportAvMode::VideoOnly),
        _ => return Err(DocumentEditRuntimeError::LibraryFileUnreadable),
    };

    let mut layers = snapshot.layers.clone();
    let expected_live_next = layers.peek_next();
    let layer_id = layers.reserve()?;
    let (track_id, index) = rectangle_insertion(snapshot, current_primary)
        .ok_or(DocumentEditRuntimeError::NoTrackForRectangle)?;
    let mut envelope = ItemEnvelope::new(layer_id);
    envelope.transform.position = motolii_doc::DocParam::const_vec2(request.position);
    commands.push(Command::AddTrackItem {
        parent: ParentLocator::Track(track_id),
        index,
        item: TrackItem::Clip(Clip {
            envelope,
            start: request.playhead,
            duration,
            time_map: Default::default(),
            source,
        }),
        layer_names: BTreeMap::from([(layer_id, request.name)]),
    });
    Ok((commands, layer_id, expected_live_next))
}

pub(super) fn existing_asset_for_path(snapshot: &Document, abs: &str) -> Option<AssetId> {
    let normalized = Asset::normalize_path(abs);
    snapshot.assets.iter().find_map(|asset| {
        let path = asset.path_absolute.as_deref()?;
        (Asset::normalize_path(path) == normalized).then_some(asset.id)
    })
}

pub(super) fn rectangle_insertion(
    snapshot: &Document,
    current_primary: Option<LayerId>,
) -> Option<(motolii_doc::TrackId, usize)> {
    if let Some(primary) = current_primary {
        for track in &snapshot.tracks {
            if let Some(index) = track
                .items
                .iter()
                .position(|item| item_layer_id(item) == primary)
            {
                if rectangle_selection_is_compatible(&track.items[index]) {
                    return Some((track.id, index + 1));
                }
                break;
            }
        }
    }
    snapshot
        .tracks
        .first()
        .map(|track| (track.id, track.items.len()))
}

pub(super) fn item_layer_id(item: &TrackItem) -> LayerId {
    match item {
        TrackItem::Clip(clip) => clip.envelope.layer_id,
        TrackItem::Group(group) => group.envelope.layer_id,
    }
}

pub(super) fn rectangle_selection_is_compatible(item: &TrackItem) -> bool {
    match item {
        TrackItem::Group(_) => true,
        TrackItem::Clip(clip) => match &clip.source {
            ClipSource::Asset { video, .. } => video.is_some(),
            ClipSource::Plugin { .. } | ClipSource::Vector { .. } => true,
        },
    }
}
