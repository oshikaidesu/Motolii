use std::path::Path;

use crate::render::media::{SpatialBounds, SpatialBoundsError};

/// Rerun Asset3Dのstable形式。DAEはupstream importerのsubsetなので、黙って受理しない。
pub const MESH_EXTENSIONS: &[&str] = &["glb", "obj", "stl"];

#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("対応していない3D形式: {0}")]
    Unsupported(String),
    #[error("3D素材を読めない: {0}")]
    Read(String),
    #[error(transparent)]
    Bounds(#[from] SpatialBoundsError),
}

pub fn is_mesh_extension(ext: &str) -> bool {
    MESH_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn is_mesh_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_mesh_extension)
}

pub fn load_mesh_bounds(path: &str) -> Result<SpatialBounds, MeshError> {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| MeshError::Unsupported(path.to_owned()))?;
    match extension.as_str() {
        "obj" => obj_bounds(path),
        "glb" => gltf_bounds(path),
        "stl" => stl_bounds(path),
        _ => Err(MeshError::Unsupported(extension)),
    }
}

fn obj_bounds(path: &str) -> Result<SpatialBounds, MeshError> {
    let options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let (models, _materials) =
        tobj::load_obj(path, &options).map_err(|error| MeshError::Read(error.to_string()))?;
    SpatialBounds::from_points(models.iter().flat_map(|model| {
        model
            .mesh
            .positions
            .chunks_exact(3)
            .map(|point| [point[0], point[1], point[2]])
    }))
    .map_err(Into::into)
}

fn gltf_bounds(path: &str) -> Result<SpatialBounds, MeshError> {
    let (document, buffers, _images) =
        gltf::import(path).map_err(|error| MeshError::Read(error.to_string()))?;
    let mut points = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            collect_gltf_node_points(&node, glam::Affine3A::IDENTITY, &buffers, &mut points);
        }
    }
    SpatialBounds::from_points(points).map_err(Into::into)
}

fn collect_gltf_node_points(
    node: &gltf::Node<'_>,
    parent_from_node: glam::Affine3A,
    buffers: &[gltf::buffer::Data],
    points: &mut Vec<[f32; 3]>,
) {
    let local = match node.transform() {
        gltf::scene::Transform::Matrix { matrix } => {
            glam::Affine3A::from_mat4(glam::Mat4::from_cols_array_2d(&matrix))
        }
        gltf::scene::Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => glam::Affine3A::from_scale_rotation_translation(
            glam::Vec3::from(scale),
            glam::Quat::from_array(rotation),
            glam::Vec3::from(translation),
        ),
    };
    let world_from_node = parent_from_node * local;
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            if let Some(positions) = reader.read_positions() {
                points.extend(positions.map(|point| {
                    world_from_node
                        .transform_point3(glam::Vec3::from(point))
                        .to_array()
                }));
            }
        }
    }
    for child in node.children() {
        collect_gltf_node_points(&child, world_from_node, buffers, points);
    }
}

fn stl_bounds(path: &str) -> Result<SpatialBounds, MeshError> {
    let mut file = std::fs::File::open(path).map_err(|error| MeshError::Read(error.to_string()))?;
    let mesh = stl_io::read_stl(&mut file).map_err(|error| MeshError::Read(error.to_string()))?;
    SpatialBounds::from_points(mesh.vertices.into_iter().map(|vertex| vertex.0)).map_err(Into::into)
}
