use glam::{vec3a, Vec3A};
use obvhs::{
    cwbvh::builder::build_cwbvh_from_tris,
    ray::{Ray, RayHit},
    triangle::Triangle,
    BvhBuildParams,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionToken {
    pub generation: u64,
    pub semantic_id: SemanticId,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PickingError {
    #[error("empty primitive set")]
    EmptyPrimitives,
    #[error("stale readback generation")]
    StaleGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub semantic_id: SemanticId,
    pub distance: f32,
}

pub fn build_dense_fixture(count: usize) -> Result<(Vec<Triangle>, Vec<SemanticId>), PickingError> {
    if count == 0 {
        return Err(PickingError::EmptyPrimitives);
    }
    let mut triangles = Vec::with_capacity(count);
    let mut ids = Vec::with_capacity(count);
    for index in 0..count {
        let x = (index % 100) as f32 * 0.2 - 9.9;
        let y = (index / 100) as f32 * 0.2 - (count / 100) as f32 * 0.1;
        triangles.push(Triangle {
            v0: vec3a(x - 0.07, y - 0.07, 0.0),
            v1: vec3a(x + 0.07, y - 0.07, 0.0),
            v2: vec3a(x, y + 0.07, 0.0),
        });
        ids.push(SemanticId(index as u64 * 17 + 3));
    }
    Ok((triangles, ids))
}

pub fn flat_pick(triangles: &[Triangle], ids: &[SemanticId], ray: Ray) -> Option<Hit> {
    triangles
        .iter()
        .zip(ids)
        .filter_map(|(triangle, id)| {
            let distance = triangle.intersect(&ray);
            distance.is_finite().then_some(Hit {
                semantic_id: *id,
                distance,
            })
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
}

pub fn obvhs_pick(triangles: &[Triangle], ids: &[SemanticId], ray: Ray) -> Option<Hit> {
    let mut build_time = std::time::Duration::default();
    let bvh = build_cwbvh_from_tris(triangles, BvhBuildParams::medium_build(), &mut build_time);
    let mut ray_hit = RayHit::none();
    if !bvh.ray_traverse(ray, &mut ray_hit, |candidate_ray, primitive_index| {
        triangles[bvh.primitive_indices[primitive_index] as usize].intersect(candidate_ray)
    }) {
        return None;
    }
    let primitive = bvh.primitive_indices[ray_hit.primitive_id as usize] as usize;
    Some(Hit {
        semantic_id: ids[primitive],
        distance: ray_hit.t,
    })
}

pub fn accept_readback(
    requested_generation: u64,
    current_generation: u64,
    token: SelectionToken,
) -> Result<SemanticId, PickingError> {
    if requested_generation != current_generation || token.generation != current_generation {
        return Err(PickingError::StaleGeneration);
    }
    Ok(token.semantic_id)
}

pub fn ray_to_index(index: usize, count: usize) -> Ray {
    let x = (index % 100) as f32 * 0.2 - 9.9;
    let y = (index / 100) as f32 * 0.2 - (count / 100) as f32 * 0.1;
    Ray::new_inf(vec3a(x, y, 2.0), vec3a(0.0, 0.0, -1.0))
}

pub fn _vec3a_is_finite(value: Vec3A) -> bool {
    value.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bvh_matches_owned_flat_reference_for_dense_scene() {
        let (triangles, ids) = build_dense_fixture(10_000).unwrap();
        for index in [0, 1, 99, 100, 4_999, 9_999] {
            let ray = ray_to_index(index, triangles.len());
            let flat = flat_pick(&triangles, &ids, ray).expect("flat hit");
            let bvh = obvhs_pick(&triangles, &ids, ray).expect("bvh hit");
            assert_eq!(bvh.semantic_id, flat.semantic_id);
            assert!((bvh.distance - flat.distance).abs() < 0.0001);
        }
    }

    #[test]
    fn stale_generation_is_rejected_without_readback_wait() {
        let token = SelectionToken {
            generation: 4,
            semantic_id: SemanticId(99),
        };
        assert_eq!(accept_readback(4, 4, token), Ok(SemanticId(99)));
        assert_eq!(
            accept_readback(4, 5, token),
            Err(PickingError::StaleGeneration)
        );
        assert_eq!(
            accept_readback(5, 5, token),
            Err(PickingError::StaleGeneration)
        );
    }

    #[test]
    fn semantic_ids_are_not_array_indices() {
        let (_, ids) = build_dense_fixture(4).unwrap();
        assert_ne!(ids[0].0, 0);
        assert_ne!(ids[1].0, 1);
    }
}
