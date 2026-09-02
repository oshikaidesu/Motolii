#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum SpatialBoundsError {
    #[error("3D素材に点が1つも無い")]
    Empty,
    #[error("3D素材に有限でない座標がある")]
    NonFinite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl SpatialBounds {
    pub fn from_points(points: impl IntoIterator<Item = [f32; 3]>) -> Result<Self, SpatialBoundsError> {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        let mut any = false;
        for point in points {
            if !point.iter().all(|value| value.is_finite()) {
                return Err(SpatialBoundsError::NonFinite);
            }
            any = true;
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        any.then_some(Self { min, max })
            .ok_or(SpatialBoundsError::Empty)
    }

    pub fn center(self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn radius(self) -> f32 {
        let size = self.size();
        0.5 * (size[0] * size[0] + size[1] * size[1] + size[2] * size[2]).sqrt()
    }

    pub fn size(self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    pub fn size_xy(self) -> [f32; 2] {
        let size = self.size();
        [size[0].max(f32::EPSILON), size[1].max(f32::EPSILON)]
    }
}

impl From<macaw::BoundingBox> for SpatialBounds {
    fn from(bounds: macaw::BoundingBox) -> Self {
        Self {
            min: bounds.min.to_array(),
            max: bounds.max.to_array(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_bounds_contract_serves_meshes_and_points() {
        let bounds = SpatialBounds::from_points([
            [-1.0, -2.0, -3.0],
            [3.0, 4.0, 5.0],
            [0.0, 1.0, 2.0],
        ])
        .unwrap();
        assert_eq!(bounds.center(), [1.0, 1.0, 1.0]);
        assert_eq!(bounds.size(), [4.0, 6.0, 8.0]);
        assert!((bounds.radius() - (116.0f32).sqrt() * 0.5).abs() < 1e-6);
    }

    #[test]
    fn invalid_spatial_coordinates_are_typed_failures() {
        assert_eq!(
            SpatialBounds::from_points(std::iter::empty()),
            Err(SpatialBoundsError::Empty)
        );
        assert_eq!(
            SpatialBounds::from_points([[f32::NAN, 0.0, 0.0]]),
            Err(SpatialBoundsError::NonFinite)
        );
    }
}
