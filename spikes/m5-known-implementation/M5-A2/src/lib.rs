use std::io::Cursor;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct FaithfulObjAsset {
    pub meshes: Vec<ObjMesh>,
    pub materials: Vec<ObjMaterial>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub texcoords: Option<Vec<[f32; 2]>>,
    pub indices: Vec<u32>,
    pub material: MaterialBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterialBinding {
    Named(usize),
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjMaterial {
    pub name: String,
    pub diffuse_texture: Option<String>,
    pub normal_texture: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ObjLoweringError {
    #[error("OBJ parse failed: {message}")]
    Parse { message: String },
    #[error("OBJ mesh {mesh} has a non-triangular index count {count}")]
    NonTriangular { mesh: usize, count: usize },
    #[error("OBJ mesh {mesh} contains non-finite vertex data")]
    NonFiniteVertex { mesh: usize },
    #[error("OBJ mesh {mesh} refers to missing material index {material}")]
    MissingMaterial { mesh: usize, material: usize },
}

/// OBJを製品型へ変換せず、faithful assetのprivate probeへlowerする。
pub fn lower_obj(bytes: &[u8]) -> Result<FaithfulObjAsset, ObjLoweringError> {
    let load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ignore_points: true,
        ignore_lines: true,
    };
    let (models, materials) = tobj::load_obj_buf(&mut Cursor::new(bytes), &load_options, |_path| {
        Ok((Vec::new(), Default::default()))
    })
    .map_err(|error| ObjLoweringError::Parse {
        message: error.to_string(),
    })?;

    let materials = materials
        .map_err(|error| ObjLoweringError::Parse {
            message: error.to_string(),
        })?
        .into_iter()
        .map(|material| ObjMaterial {
            name: material.name,
            diffuse_texture: material.diffuse_texture,
            normal_texture: material.normal_texture,
        })
        .collect::<Vec<_>>();

    let mut meshes = Vec::with_capacity(models.len());
    for (mesh_index, model) in models.into_iter().enumerate() {
        let mesh = model.mesh;
        if mesh.indices.len() % 3 != 0 {
            return Err(ObjLoweringError::NonTriangular {
                mesh: mesh_index,
                count: mesh.indices.len(),
            });
        }
        if mesh.positions.iter().any(|value| !value.is_finite())
            || mesh.normals.iter().any(|value| !value.is_finite())
            || mesh.texcoords.iter().any(|value| !value.is_finite())
        {
            return Err(ObjLoweringError::NonFiniteVertex { mesh: mesh_index });
        }

        let material = match mesh.material_id {
            None => MaterialBinding::Missing,
            Some(material) if material < materials.len() => MaterialBinding::Named(material),
            Some(material) => {
                return Err(ObjLoweringError::MissingMaterial {
                    mesh: mesh_index,
                    material,
                })
            }
        };
        let positions = mesh
            .positions
            .chunks_exact(3)
            .map(|value| [value[0], value[1], value[2]])
            .collect();
        let normals = (!mesh.normals.is_empty()).then(|| {
            mesh.normals
                .chunks_exact(3)
                .map(|value| [value[0], value[1], value[2]])
                .collect()
        });
        let texcoords = (!mesh.texcoords.is_empty()).then(|| {
            mesh.texcoords
                .chunks_exact(2)
                .map(|value| [value[0], value[1]])
                .collect()
        });
        meshes.push(ObjMesh {
            name: model.name,
            positions,
            normals,
            texcoords,
            indices: mesh.indices,
            material,
        });
    }

    Ok(FaithfulObjAsset { meshes, materials })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRIANGLE: &str = "mtllib material.mtl\no triangle\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nusemtl painted\nf 1/1/1 2/2/1 3/3/1\n";

    #[test]
    fn lower_preserves_triangle_and_optional_attribute_presence() {
        let asset = lower_obj(TRIANGLE.as_bytes()).expect("triangle should parse");
        assert_eq!(asset.meshes.len(), 1);
        assert_eq!(asset.meshes[0].positions.len(), 3);
        assert_eq!(asset.meshes[0].indices, vec![0, 1, 2]);
        assert!(asset.meshes[0].normals.is_some());
        assert!(asset.meshes[0].texcoords.is_some());
        assert_eq!(asset.meshes[0].material, MaterialBinding::Missing);
    }

    #[test]
    fn missing_normals_uv_and_mtl_are_explicit_not_synthesized() {
        let obj = b"o bare\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let asset = lower_obj(obj).expect("bare triangle should parse");
        let mesh = &asset.meshes[0];
        assert!(mesh.normals.is_none());
        assert!(mesh.texcoords.is_none());
        assert_eq!(mesh.material, MaterialBinding::Missing);
        assert!(asset.materials.is_empty());
    }

    #[test]
    fn missing_mtl_does_not_become_pbr_material() {
        let obj =
            b"mtllib missing.mtl\no bare\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl missing\nf 1 2 3\n";
        let asset = lower_obj(obj).expect("missing MTL is an explicit absence");
        assert_eq!(asset.meshes[0].material, MaterialBinding::Missing);
        assert!(asset.materials.is_empty());
    }

    #[test]
    fn malformed_obj_is_typed_failure() {
        let error = lower_obj(b"f nope").expect_err("malformed input must fail");
        assert!(matches!(error, ObjLoweringError::Parse { .. }));
    }
}
