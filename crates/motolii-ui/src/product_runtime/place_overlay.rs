//! place中の矩形overlay。Documentには触れず表示用頂点だけを作る。

use motolii_core::CanonicalPoint;

use crate::app::canonical_drop_from_ndc;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RectanglePlaceOverlay {
    pub(super) vertices: [[f32; 2]; 6],
}

impl RectanglePlaceOverlay {
    pub(super) fn vertex_bytes(&self) -> [u8; 48] {
        let mut bytes = [0_u8; 48];
        for (index, component) in self.vertices.iter().flatten().enumerate() {
            let start = index * 4;
            bytes[start..start + 4].copy_from_slice(&component.to_ne_bytes());
        }
        bytes
    }
}

pub(super) fn rectangle_place_overlay(
    camera: motolii_core::CompCamera,
    ndc: [f64; 2],
) -> Option<RectanglePlaceOverlay> {
    let center = canonical_drop_from_ndc(camera, ndc)?;
    let corners = [
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] + 0.1,
        },
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] + 0.1,
        },
    ];
    let mut projected = [[0.0_f32; 2]; 4];
    for (target, corner) in projected.iter_mut().zip(corners) {
        let (x, y) = camera.world_to_ndc(corner).ok()?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        *target = [x as f32, y as f32];
    }
    Some(RectanglePlaceOverlay {
        vertices: [
            projected[0],
            projected[1],
            projected[2],
            projected[0],
            projected[2],
            projected[3],
        ],
    })
}
