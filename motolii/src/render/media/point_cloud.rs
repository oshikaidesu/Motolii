
use std::path::Path;

use re_types_core::Loggable as _;

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

pub fn asset_type_for_extension(extension: &str) -> Option<String> {
    let e = extension.to_ascii_lowercase();
    let e = e.as_str();
    if re_importer::SUPPORTED_VIDEO_EXTENSIONS.contains(&e) {
        Some(format!("video/{e}"))
    } else if re_importer::SUPPORTED_IMAGE_EXTENSIONS.contains(&e) {
        Some(format!("image/{e}"))
    } else if re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS.contains(&e) {
        Some(format!("pointcloud.{e}"))
    } else if re_importer::SUPPORTED_MESH_EXTENSIONS.contains(&e) {
        Some(format!("model/{e}"))
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
}

#[derive(Debug, Clone)]
pub struct PointCloudData {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 4]>,
}

impl PointCloudData {
    pub fn point_count(&self) -> usize {
        self.positions.len()
    }

    /// 点群を囲む球(中心と半径)。層を作る時の初期倍率に使う。
    pub fn bounding_sphere(&self) -> ([f32; 3], f32) {
        if self.positions.is_empty() {
            return ([0.0; 3], 0.0);
        }
        let mut lo = self.positions[0];
        let mut hi = self.positions[0];
        for p in &self.positions {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let center = [
            (lo[0] + hi[0]) * 0.5,
            (lo[1] + hi[1]) * 0.5,
            (lo[2] + hi[2]) * 0.5,
        ];
        let radius = self
            .positions
            .iter()
            .map(|p| {
                let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        (center, radius)
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

    Ok(PointCloudData { positions, colors })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    const ASCII_PLY: &str = "ply\n\
format ascii 1.0\n\
element vertex 3\n\
property float x\n\
property float y\n\
property float z\n\
property uchar red\n\
property uchar green\n\
property uchar blue\n\
end_header\n\
0 0 0 255 0 0\n\
1 0 0 0 255 0\n\
0 1 0 0 0 255\n";

    #[test]
    fn extension_recognition_delegates_to_re_importer() {
        assert!(is_point_cloud_extension("ply"));
        assert!(is_point_cloud_extension("PLY"));
        assert!(!is_point_cloud_extension("mp4"));
        assert!(!is_point_cloud_extension("glb"));
        assert_eq!(POINT_CLOUD_EXTENSIONS, re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS);
    }

    #[test]
    fn rerun_importable_extension_covers_all_registered_formats_not_just_point_clouds() {
        assert!(is_rerun_importable_extension("ply")); // point cloud
        assert!(is_rerun_importable_extension("PNG")); // image
        assert!(is_rerun_importable_extension("glb")); // mesh
        assert!(is_rerun_importable_extension("rrd")); // rerun native
        assert!(!is_rerun_importable_extension("exe"));
    }

    #[test]
    fn load_point_cloud_reads_positions_and_colors_from_a_real_ply_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("three_points.ply");
        std::fs::File::create(&path)
            .and_then(|mut f| f.write_all(ASCII_PLY.as_bytes()))
            .expect("write fixture ply");

        let data = load_point_cloud(&path).expect("load_point_cloud");
        assert_eq!(data.point_count(), 3);
        assert_eq!(data.positions, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(
            data.colors,
            vec![[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]]
        );
    }
}

#[cfg(test)]
mod real_files {
    use super::{asset_type_for_extension, is_rerun_importable_extension, load_point_cloud};

    fn testdata() -> Option<std::path::PathBuf> {
        std::env::var_os("MOTOLII_TESTDATA").map(std::path::PathBuf::from)
    }

    #[test]
    fn every_fetched_sample_is_importable_and_gets_an_asset_type() {
        let Some(dir) = testdata() else { return };
        let expected = [
            ("Box.glb", "model/glb"),
            ("Duck.glb", "model/glb"),
            ("dolphins.ply", "pointcloud.ply"),
            ("test.png", "image/png"),
            ("sample.mp4", "video/mp4"),
        ];
        for (file, asset_type) in expected {
            let path = dir.join(file);
            assert!(path.exists(), "{file} が無い");
            let ext = path.extension().unwrap().to_str().unwrap();
            assert!(is_rerun_importable_extension(ext), "{file} を読めない");
            assert_eq!(asset_type_for_extension(ext).as_deref(), Some(asset_type));
        }
    }

    #[test]
    fn a_real_ply_loads_its_points() {
        let Some(dir) = testdata() else { return };
        let data = load_point_cloud(&dir.join("dolphins.ply")).expect("ply");
        assert_eq!(data.positions.len(), 855);
    }
}
