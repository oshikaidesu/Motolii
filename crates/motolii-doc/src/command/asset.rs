use crate::asset::{Asset, AssetDraft, AssetError, AssetId};
use crate::Document;

use super::reservation::swap_if_valid;
use super::{Command, CommandError, PreparedAssetAdmission};

pub(crate) fn prepare_admit_asset(
    doc: &Document,
    draft: AssetDraft,
) -> Result<PreparedAssetAdmission, CommandError> {
    let expected_next = doc.assets.peek_next();
    let asset = draft.into_asset(AssetId::from_raw(expected_next));
    let command = Command::AdmitAsset {
        asset: asset.clone(),
    };
    let mut candidate = doc.clone();
    command.apply(&mut candidate)?;
    Ok(PreparedAssetAdmission {
        expected_next,
        asset,
    })
}

pub(crate) fn prepare_remove_asset(doc: &Document, id: AssetId) -> Result<Command, CommandError> {
    let asset = doc
        .assets
        .get(id)
        .cloned()
        .ok_or(AssetError::NotFound { id: id.get() })?;
    let command = Command::RemoveAsset { asset };
    let mut candidate = doc.clone();
    command.apply(&mut candidate)?;
    Ok(command)
}

pub(super) fn apply_admit_asset(doc: &mut Document, asset: &Asset) -> Result<(), CommandError> {
    let mut asset = asset.clone();
    asset.normalize_self();
    let id = asset.id;
    let next = doc.assets.peek_next();
    if doc.assets.get(id).is_some() {
        return Err(AssetError::Duplicate { id: id.get() }.into());
    }
    if id.get() > next {
        return Err(CommandError::AssetAdmissionCounterMismatch { id: id.get(), next });
    }

    let mut candidate = doc.clone();
    if id.get() == next {
        candidate.assets.insert(asset)?;
    } else {
        candidate.assets.restore(asset)?;
    }
    swap_if_valid(doc, candidate)
}

pub(super) fn apply_remove_asset(doc: &mut Document, asset: &Asset) -> Result<(), CommandError> {
    let mut expected = asset.clone();
    expected.normalize_self();
    let id = expected.id;
    let existing = doc
        .assets
        .get(id)
        .ok_or(AssetError::NotFound { id: id.get() })?;
    if existing != &expected {
        return Err(CommandError::RemoveAssetPayloadMismatch { id: id.get() });
    }
    let use_count = doc.asset_use_count(id);
    if use_count != 0 {
        return Err(CommandError::AssetInUse {
            id: id.get(),
            use_count,
        });
    }

    let mut candidate = doc.clone();
    candidate.assets.remove(id)?;
    swap_if_valid(doc, candidate)
}
