//! wraps: `re_importer`(拡張子 registry) + `re_sdk_types`(`.ply` パーサ)。
//!
//! **パーサはここに無い**。`re_sdk_types::archetypes::Points3D::from_file_contents` が
//! 実体(`ply-rs-bw` を内部で使う、`archetypes::points3d_ext` 参照) —
//! `re_importer::importer_archetype::load_point_cloud` が呼ぶのと**同じ関数**を
//! ここでも呼ぶ。Rerun 側がこの関数を更新すれば、ここは無改造でその恩恵を受ける。
//!
//! 拡張子の正本は `re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS`。motolii は
//! 独自の一覧を持たない — 将来 Rerun がこの定数へ拡張子を足せば
//! [`is_point_cloud_extension`] は無改造でそれを拾う。
//!
//! **mesh/rrd など他の Rerun 対応型を足す時のひな形**: この module と同じ形
//! (`re_importer::SUPPORTED_*_EXTENSIONS` を引く判定関数 + `re_sdk_types` の
//! 対応する `from_file_contents` を呼ぶ読み口)を並べて増やすだけでよい —
//! `re_importer` 依存は既にこの crate に居るので、新しい外部依存は増えない。

use std::path::Path;

use re_types_core::Loggable as _;

/// `.ply` 等、点群として開ける拡張子(先頭 `.` 無し)。**正本は `re_importer`**——
/// 呼び手(`browser_surface::pick_media_path`/`asset_type_for` 等)はこれを import
/// filter・種別判定にそのまま使い、独自の一覧を持たない。
pub use re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS as POINT_CLOUD_EXTENSIONS;

/// この拡張子(先頭 `.` 無し、大小文字を問わない)が点群として開けるか。
/// **正本は `re_importer`**——独自の拡張子表を持たない。
pub fn is_point_cloud_extension(extension: &str) -> bool {
    let extension = extension.to_ascii_lowercase();
    re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS.contains(&extension.as_str())
}

/// この拡張子(先頭 `.` 無し、大小文字を問わない)を Rerun の組み込み importer の
/// どれかが読めるか。形式ごとの一覧はここに持たず `re_importer::is_supported_file_extension`
/// (`SUPPORTED_*_EXTENSIONS` 群を横断する)へそのまま委ねる。
pub fn is_rerun_importable_extension(extension: &str) -> bool {
    re_importer::is_supported_file_extension(&extension.to_ascii_lowercase())
}

/// 拡張子から素材台帳の `asset_type` を決める。区分の正本は
/// `re_importer::SUPPORTED_*_EXTENSIONS`。読めない拡張子は `None`。
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

/// 点群1本ぶんの幾何。world 単位はファイルそのままの座標系
/// (スケール/単位の正規化はこの切片の非目標)。
#[derive(Debug, Clone)]
pub struct PointCloudData {
    pub positions: Vec<[f32; 3]>,
    /// `positions` と同じ長さとは限らない——ply に色列が無ければ空
    /// (呼び手は「足りない分は白」という re_renderer 既定へ委ねる)。
    pub colors: Vec<[u8; 4]>,
}

impl PointCloudData {
    pub fn point_count(&self) -> usize {
        self.positions.len()
    }
}

/// ファイルを読み、`Points3D::from_file_contents`(Rerun 本体の `.ply` importer が
/// 使うのと同じ関数、`re_importer::importer_archetype::load_point_cloud` 参照)で
/// 点群へ写す。
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
        // Motolii が独自に持っていない列挙 — re_importer の定数そのものを検査する。
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

/// 実ファイルを通す検分。`MOTOLII_TESTDATA` にディレクトリを渡した時だけ走る
/// (リポにバイナリを置かないため)。
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
