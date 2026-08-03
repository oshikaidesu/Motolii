use std::collections::BTreeSet;

use gltf::mesh::Mode;
use mikktspace::Geometry;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct PreflightPolicy<'a> {
    pub max_bytes: usize,
    pub supported_required_extensions: &'a [&'a str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreflightReport {
    pub meshes: usize,
    pub primitives: usize,
    pub generated_tangent_corners: usize,
    pub source_cameras: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PreflightError {
    #[error("asset is {actual} bytes; limit is {max} bytes")]
    Oversize { actual: usize, max: usize },
    #[error("input is not a binary glTF (GLB)")]
    NotBinaryGlb,
    #[error("glTF parse or schema validation failed: {message}")]
    InvalidGltf { message: String },
    #[error("required glTF extensions are unsupported: {names:?}")]
    UnsupportedRequiredExtensions { names: Vec<String> },
    #[error("GLB contains an external {kind} URI at index {index}: {uri}")]
    ExternalResource {
        kind: &'static str,
        index: usize,
        uri: String,
    },
    #[error("GLB buffer {buffer} requires a missing BIN chunk")]
    MissingBlob { buffer: usize },
    #[error("mesh {mesh} primitive {primitive} uses unsupported mode {mode:?}")]
    UnsupportedPrimitiveMode {
        mesh: usize,
        primitive: usize,
        mode: Mode,
    },
    #[error("mesh {mesh} primitive {primitive} is missing {attribute}")]
    MissingAttribute {
        mesh: usize,
        primitive: usize,
        attribute: String,
    },
    #[error("mesh {mesh} primitive {primitive} has invalid triangle indices")]
    InvalidTriangleIndices { mesh: usize, primitive: usize },
    #[error("mesh {mesh} primitive {primitive} failed MikkTSpace tangent generation")]
    TangentGenerationFailed { mesh: usize, primitive: usize },
}

pub fn preflight_glb(
    bytes: &[u8],
    policy: PreflightPolicy<'_>,
) -> Result<PreflightReport, PreflightError> {
    if bytes.len() > policy.max_bytes {
        return Err(PreflightError::Oversize {
            actual: bytes.len(),
            max: policy.max_bytes,
        });
    }
    if !bytes.starts_with(b"glTF") {
        return Err(PreflightError::NotBinaryGlb);
    }

    // required extension名をvalidation errorへ潰さず、拒否理由として保持する。
    let unvalidated = gltf::Gltf::from_slice_without_validation(bytes).map_err(|error| {
        PreflightError::InvalidGltf {
            message: error.to_string(),
        }
    })?;

    let supported: BTreeSet<&str> = policy
        .supported_required_extensions
        .iter()
        .copied()
        .collect();
    let mut unsupported: Vec<String> = unvalidated
        .extensions_required()
        .filter(|name| !supported.contains(*name))
        .map(str::to_owned)
        .collect();
    unsupported.sort();
    unsupported.dedup();
    if !unsupported.is_empty() {
        return Err(PreflightError::UnsupportedRequiredExtensions { names: unsupported });
    }

    // 名前回収後もschema validationを省略しない。
    let asset = gltf::Gltf::from_slice(bytes).map_err(|error| PreflightError::InvalidGltf {
        message: error.to_string(),
    })?;

    for buffer in asset.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin if asset.blob.is_none() => {
                return Err(PreflightError::MissingBlob {
                    buffer: buffer.index(),
                });
            }
            gltf::buffer::Source::Uri(uri) => {
                return Err(PreflightError::ExternalResource {
                    kind: "buffer",
                    index: buffer.index(),
                    uri: uri.to_owned(),
                });
            }
            gltf::buffer::Source::Bin => {}
        }
    }
    for image in asset.images() {
        if let gltf::image::Source::Uri { uri, .. } = image.source() {
            return Err(PreflightError::ExternalResource {
                kind: "image",
                index: image.index(),
                uri: uri.to_owned(),
            });
        }
    }

    let mut primitives = 0;
    let mut generated_tangent_corners = 0;
    for mesh in asset.meshes() {
        for primitive in mesh.primitives() {
            let primitive_index = primitive.index();
            primitives += 1;
            if primitive.mode() != Mode::Triangles {
                return Err(PreflightError::UnsupportedPrimitiveMode {
                    mesh: mesh.index(),
                    primitive: primitive_index,
                    mode: primitive.mode(),
                });
            }
            if primitive.get(&gltf::Semantic::Positions).is_none() {
                return Err(PreflightError::MissingAttribute {
                    mesh: mesh.index(),
                    primitive: primitive_index,
                    attribute: "POSITION".to_owned(),
                });
            }

            if let (None, Some(normal_texture)) = (
                primitive.get(&gltf::Semantic::Tangents),
                primitive.material().normal_texture(),
            ) {
                let blob = asset
                    .blob
                    .as_deref()
                    .ok_or(PreflightError::MissingBlob { buffer: 0 })?;
                let mut geometry = TangentGeometry::from_primitive(
                    mesh.index(),
                    &primitive,
                    blob,
                    normal_texture.tex_coord(),
                )?;
                if !mikktspace::generate_tangents(&mut geometry)
                    || geometry
                        .tangents
                        .iter()
                        .any(|tangent| !valid_encoded_tangent(*tangent))
                {
                    return Err(PreflightError::TangentGenerationFailed {
                        mesh: mesh.index(),
                        primitive: primitive_index,
                    });
                }
                generated_tangent_corners += geometry.tangents.len();
            }
        }
    }

    Ok(PreflightReport {
        meshes: asset.meshes().count(),
        primitives,
        generated_tangent_corners,
        source_cameras: asset.cameras().count(),
    })
}

fn valid_encoded_tangent(tangent: [f32; 4]) -> bool {
    let length_squared =
        tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2];
    tangent.iter().all(|component| component.is_finite())
        && (0.99..=1.01).contains(&length_squared)
        && (tangent[3].abs() - 1.0).abs() <= f32::EPSILON
}

struct TangentGeometry {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
    indices: Vec<u32>,
    tangents: Vec<[f32; 4]>,
}

impl TangentGeometry {
    fn from_primitive(
        mesh_index: usize,
        primitive: &gltf::Primitive<'_>,
        blob: &[u8],
        tex_coord_set: u32,
    ) -> Result<Self, PreflightError> {
        let primitive_index = primitive.index();
        let reader = primitive.reader(|buffer| match buffer.source() {
            gltf::buffer::Source::Bin => Some(blob),
            gltf::buffer::Source::Uri(_) => None,
        });
        let positions: Vec<_> = reader
            .read_positions()
            .ok_or(PreflightError::MissingAttribute {
                mesh: mesh_index,
                primitive: primitive_index,
                attribute: "POSITION".to_owned(),
            })?
            .collect();
        let normals: Vec<_> = reader
            .read_normals()
            .ok_or(PreflightError::MissingAttribute {
                mesh: mesh_index,
                primitive: primitive_index,
                attribute: "NORMAL".to_owned(),
            })?
            .collect();
        let tex_coords: Vec<_> = reader
            .read_tex_coords(tex_coord_set)
            .ok_or(PreflightError::MissingAttribute {
                mesh: mesh_index,
                primitive: primitive_index,
                attribute: format!("TEXCOORD_{tex_coord_set}"),
            })?
            .into_f32()
            .collect();
        let indices: Vec<u32> = reader
            .read_indices()
            .map(|values| values.into_u32().collect())
            .unwrap_or_else(|| (0..positions.len() as u32).collect());
        if indices.is_empty()
            || !indices.len().is_multiple_of(3)
            || indices
                .iter()
                .any(|index| *index as usize >= positions.len())
            || normals.len() != positions.len()
            || tex_coords.len() != positions.len()
        {
            return Err(PreflightError::InvalidTriangleIndices {
                mesh: mesh_index,
                primitive: primitive_index,
            });
        }
        Ok(Self {
            positions,
            normals,
            tex_coords,
            tangents: vec![[0.0; 4]; indices.len()],
            indices,
        })
    }

    fn source_index(&self, face: usize, vert: usize) -> usize {
        self.indices[face * 3 + vert] as usize
    }
}

impl Geometry for TangentGeometry {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.positions[self.source_index(face, vert)]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.normals[self.source_index(face, vert)]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.tex_coords[self.source_index(face, vert)]
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        self.tangents[face * 3 + vert] = tangent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: PreflightPolicy<'static> = PreflightPolicy {
        max_bytes: 1024 * 1024,
        supported_required_extensions: &[],
    };

    #[test]
    fn accepts_embedded_triangle_and_generates_missing_tangents() {
        let bytes = triangle_with_normal_map_glb();
        let report = preflight_glb(&bytes, POLICY).unwrap();
        assert_eq!(report.meshes, 1);
        assert_eq!(report.primitives, 1);
        assert_eq!(report.generated_tangent_corners, 3);
    }

    #[test]
    fn rejects_malformed_glb_without_panicking() {
        let mut bytes = triangle_with_normal_map_glb();
        bytes.truncate(19);
        assert!(matches!(
            preflight_glb(&bytes, POLICY),
            Err(PreflightError::InvalidGltf { .. })
        ));
    }

    #[test]
    fn reports_required_extension_by_name() {
        let bytes = make_glb(
            r#"{"asset":{"version":"2.0"},"extensionsUsed":["VENDOR_z","VENDOR_a"],"extensionsRequired":["VENDOR_z","VENDOR_a"]}"#,
            &[],
        );
        assert_eq!(
            preflight_glb(&bytes, POLICY),
            Err(PreflightError::UnsupportedRequiredExtensions {
                names: vec!["VENDOR_a".to_owned(), "VENDOR_z".to_owned()]
            })
        );
    }

    #[test]
    fn rejects_oversize_before_parser_allocation() {
        let bytes = triangle_with_normal_map_glb();
        let policy = PreflightPolicy {
            max_bytes: bytes.len() - 1,
            ..POLICY
        };
        assert_eq!(
            preflight_glb(&bytes, policy),
            Err(PreflightError::Oversize {
                actual: bytes.len(),
                max: bytes.len() - 1
            })
        );
    }

    #[test]
    fn rejects_escape_uri_without_touching_the_filesystem() {
        let bytes = make_glb(
            r#"{"asset":{"version":"2.0"},"buffers":[{"byteLength":4,"uri":"../escape.bin"}]}"#,
            &[],
        );
        assert_eq!(
            preflight_glb(&bytes, POLICY),
            Err(PreflightError::ExternalResource {
                kind: "buffer",
                index: 0,
                uri: "../escape.bin".to_owned()
            })
        );
    }

    #[test]
    fn rejects_json_gltf_at_the_glb_boundary() {
        let bytes = br#"{"asset":{"version":"2.0"}}"#;
        assert_eq!(
            preflight_glb(bytes, POLICY),
            Err(PreflightError::NotBinaryGlb)
        );
    }

    fn triangle_with_normal_map_glb() -> Vec<u8> {
        let mut bin = Vec::new();
        for value in [-1.0_f32, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0] {
            bin.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0.0_f32, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0] {
            bin.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0.0_f32, 0.0, 1.0, 0.0, 0.5, 1.0] {
            bin.extend_from_slice(&value.to_le_bytes());
        }
        for index in [0_u16, 1, 2] {
            bin.extend_from_slice(&index.to_le_bytes());
        }
        bin.extend_from_slice(&[0, 0]);
        bin.extend_from_slice(&[0x89, b'P', b'N', b'G']);

        let json = format!(
            r#"{{"asset":{{"version":"2.0"}},"buffers":[{{"byteLength":{}}}],"bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":36}},{{"buffer":0,"byteOffset":36,"byteLength":36}},{{"buffer":0,"byteOffset":72,"byteLength":24}},{{"buffer":0,"byteOffset":96,"byteLength":6}},{{"buffer":0,"byteOffset":104,"byteLength":4}}],"accessors":[{{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1,-1,0],"max":[1,1,0]}},{{"bufferView":1,"componentType":5126,"count":3,"type":"VEC3"}},{{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"}},{{"bufferView":3,"componentType":5123,"count":3,"type":"SCALAR"}}],"images":[{{"bufferView":4,"mimeType":"image/png"}}],"textures":[{{"source":0}}],"materials":[{{"normalTexture":{{"index":0}}}}],"meshes":[{{"primitives":[{{"attributes":{{"POSITION":0,"NORMAL":1,"TEXCOORD_0":2}},"indices":3,"material":0}}]}}],"nodes":[{{"mesh":0}}],"scenes":[{{"nodes":[0]}}],"scene":0}}"#,
            bin.len()
        );
        make_glb(&json, &bin)
    }

    fn make_glb(json: &str, bin: &[u8]) -> Vec<u8> {
        let mut json_chunk = json.as_bytes().to_vec();
        while !json_chunk.len().is_multiple_of(4) {
            json_chunk.push(b' ');
        }
        let mut bin_chunk = bin.to_vec();
        while !bin_chunk.len().is_multiple_of(4) {
            bin_chunk.push(0);
        }
        let total_len = 12 + 8 + json_chunk.len() + 8 + bin_chunk.len();
        let mut glb = Vec::with_capacity(total_len);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2_u32.to_le_bytes());
        glb.extend_from_slice(&(total_len as u32).to_le_bytes());
        glb.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534A_u32.to_le_bytes());
        glb.extend_from_slice(&json_chunk);
        glb.extend_from_slice(&(bin_chunk.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942_u32.to_le_bytes());
        glb.extend_from_slice(&bin_chunk);
        glb
    }
}
