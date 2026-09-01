
use std::path::Path;

pub const MESH_EXTENSIONS: &[&str] = &["obj"];

#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("obj を読めない: {0}")]
    Read(String),
    #[error("面が1つも無い")]
    Empty,
}

/// 三角形の網。色は無ければ白。**焼かない** — 呼び手が空間へ積む。
#[derive(Clone, Debug)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 4]>,
}

pub fn is_mesh_extension(ext: &str) -> bool {
    MESH_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn is_mesh_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(is_mesh_extension)
}

/// 読むのは形だけ。材質は PBR へ黙って変換しない(裁定 M5-A2)。
pub fn load_mesh(path: &str) -> Result<MeshData, MeshError> {
    let options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let (models, _materials) =
        tobj::load_obj(path, &options).map_err(|e| MeshError::Read(e.to_string()))?;

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    for model in &models {
        let mesh = &model.mesh;
        let base = positions.len() as u32;
        for xyz in mesh.positions.chunks_exact(3) {
            positions.push([xyz[0], xyz[1], xyz[2]]);
        }
        for xyz in mesh.normals.chunks_exact(3) {
            normals.push([xyz[0], xyz[1], xyz[2]]);
        }
        for tri in mesh.indices.chunks_exact(3) {
            indices.push([base + tri[0], base + tri[1], base + tri[2]]);
        }
    }
    if indices.is_empty() {
        return Err(MeshError::Empty);
    }
    // 法線が無い obj は珍しくない。無い時は手前向きにしておく(真っ黒にしない)。
    if normals.len() != positions.len() {
        normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    }
    let colors = vec![[255u8, 255, 255, 255]; positions.len()];
    Ok(MeshData { positions, indices, normals, colors })
}
