
use std::path::Path;
use std::sync::Arc;

use re_types_core::Loggable as _;

use crate::render::media::{SpatialBounds, SpatialBoundsError};

pub use re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS as POINT_CLOUD_EXTENSIONS;

pub fn is_point_cloud_extension(extension: &str) -> bool {
    let extension = extension.to_ascii_lowercase();
    re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS.contains(&extension.as_str())
}

pub fn is_point_cloud_path(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(is_point_cloud_extension)
}

pub fn is_rerun_importable_extension(extension: &str) -> bool {
    re_importer::is_supported_file_extension(&extension.to_ascii_lowercase())
}

/// 静止画か。動画と違って**1枚を焼いて置くだけ**なので、道が別れる。
pub fn is_still_image_path(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|e| re_importer::SUPPORTED_IMAGE_EXTENSIONS.contains(&e.as_str()))
}

pub(super) fn non_audio_asset_type_for_extension(extension: &str) -> Option<String> {
    let e = extension.to_ascii_lowercase();
    let e = e.as_str();
    if re_importer::SUPPORTED_VIDEO_EXTENSIONS.contains(&e) {
        Some(format!("video/{e}"))
    } else if re_importer::SUPPORTED_IMAGE_EXTENSIONS.contains(&e) {
        Some(format!("image/{e}"))
    } else if re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS.contains(&e) {
        Some(format!("pointcloud.{e}"))
    } else if crate::render::media::MESH_EXTENSIONS.contains(&e) {
        Some(format!("model/{e}"))
    } else if re_importer::SUPPORTED_MESH_EXTENSIONS.contains(&e) {
        None
    } else if re_importer::is_supported_file_extension(e) {
        Some(format!("application/{e}"))
    } else {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PointCloudError {
    #[error("読めない: {0}")]
    Io(#[from] std::io::Error),
    #[error("component を取り出せない: {0}")]
    Component(String),
    #[error(transparent)]
    Bounds(#[from] SpatialBoundsError),
}

#[derive(Debug, Clone)]
pub struct PointCloudData {
    pub positions: Arc<Vec<[f32; 3]>>,
    pub colors: Arc<Vec<[u8; 4]>>,
    bounds: SpatialBounds,
}

impl PointCloudData {
    pub fn point_count(&self) -> usize {
        self.positions.len()
    }

    pub fn bounds(&self) -> SpatialBounds {
        self.bounds
    }

    /// 点群を囲む球(中心と半径)。層を作る時の初期倍率に使う。
    pub fn bounding_sphere(&self) -> ([f32; 3], f32) {
        (self.bounds.center(), self.bounds.radius())
    }
}

pub fn load_point_cloud(path: &Path) -> Result<PointCloudData, PointCloudError> {
    let contents = std::fs::read(path)?;
    let archetype = re_sdk_types::archetypes::Points3D::from_file_contents(&contents)?;

    let positions: Vec<[f32; 3]> = archetype
        .positions
        .as_ref()
        .map(|batch| {
            re_sdk_types::components::Position3D::from_arrow(batch.array.as_ref())
                .map(|values| values.into_iter().map(|p| p.0.into()).collect())
        })
        .transpose()
        .map_err(|e| PointCloudError::Component(e.to_string()))?
        .unwrap_or_default();

    let colors: Vec<[u8; 4]> = archetype
        .colors
        .as_ref()
        .map(|batch| {
            re_sdk_types::components::Color::from_arrow(batch.array.as_ref())
                .map(|values| values.into_iter().map(|c| c.0.to_array()).collect())
        })
        .transpose()
        .map_err(|e| PointCloudError::Component(e.to_string()))?
        .unwrap_or_default();

    let bounds = SpatialBounds::from_points(positions.iter().copied())?;
    Ok(PointCloudData {
        positions: Arc::new(positions),
        colors: Arc::new(colors),
        bounds,
    })
}
