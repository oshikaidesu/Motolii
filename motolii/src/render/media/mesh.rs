
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_cube() -> std::path::PathBuf {
        let obj = "\
v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\n\
f 1 2 3\nf 1 3 4\n";
        let path = std::env::temp_dir().join(format!("motolii-mesh-{}.obj", std::process::id()));
        std::fs::write(&path, obj).unwrap();
        path
    }

    #[test]
    fn a_quad_loads_as_two_triangles() {
        let path = write_cube();
        let mesh = load_mesh(path.to_str().unwrap()).expect("読めるはず");
        assert_eq!(mesh.indices.len(), 2, "三角形が2枚にならない");
        assert_eq!(mesh.positions.len(), mesh.colors.len());
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_missing_file_is_an_error_not_a_panic() {
        assert!(load_mesh("/nonexistent/none.obj").is_err());
    }

    #[test]
    fn only_obj_is_claimed_as_a_mesh() {
        assert!(is_mesh_path("a/b/c.obj"));
        assert!(is_mesh_path("A.OBJ"));
        assert!(!is_mesh_path("a.ply"));
        assert!(!is_mesh_path("a.mp4"));
    }
}
