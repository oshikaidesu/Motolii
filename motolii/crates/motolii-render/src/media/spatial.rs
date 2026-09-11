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

    /// The 8 corners, bit 0 = x max, bit 1 = y max, bit 2 = z max.
    pub fn corners(self) -> [glam::Vec3; 8] {
        std::array::from_fn(|i| glam::Vec3::from_array(std::array::from_fn(|a| if i & (1 << a) == 0 { self.min[a] } else { self.max[a] })))
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

/// The few points that decide a shape's outline from any direction: for each of
/// `SILHOUETTE_DIRECTIONS` spread evenly over the sphere, the point farthest that way.
/// A frame fitted to these is at most ~0.5% of the radius inside the true silhouette,
/// whatever the vertex count. Computed once at load.
pub const SILHOUETTE_DIRECTIONS: usize = 256;
pub fn silhouette_points(points: impl IntoIterator<Item = glam::Vec3>) -> Vec<glam::Vec3> {
    let points: Vec<glam::Vec3> = points.into_iter().filter(|p| p.is_finite()).collect();
    if points.len() <= SILHOUETTE_DIRECTIONS {
        return points;
    }
    // Directions are spread over the sphere of the shape's own proportions, so a flat or
    // long shape is sampled as evenly as a round one.
    let min = points.iter().fold(glam::Vec3::INFINITY, |m, p| m.min(*p));
    let max = points.iter().fold(glam::Vec3::NEG_INFINITY, |m, p| m.max(*p));
    let size = (max - min).max(glam::Vec3::splat(1e-6));
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let mut kept: Vec<glam::Vec3> = Vec::with_capacity(SILHOUETTE_DIRECTIONS);
    for i in 0..SILHOUETTE_DIRECTIONS {
        let z = 1.0 - 2.0 * (i as f32 + 0.5) / SILHOUETTE_DIRECTIONS as f32;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let phi = golden * i as f32;
        let direction = glam::vec3(r * phi.cos(), r * phi.sin(), z) / size;
        let farthest = points.iter().copied().max_by(|a, b| a.dot(direction).total_cmp(&b.dot(direction))).unwrap();
        if !kept.contains(&farthest) {
            kept.push(farthest);
        }
    }
    kept
}

#[cfg(test)]
mod silhouette_tests {
    use super::*;

    /// 頂点が幾つあっても点は 256 以下で、どの向きから写しても広がりは真の値の 0.5% 以内。
    #[test]
    fn a_few_extreme_points_keep_the_extent_from_every_direction() {
        let mut points = Vec::new();
        for i in 0..20_000u32 {
            let t = i as f32 * 0.618_034;
            let z = 1.0 - 2.0 * (i as f32 + 0.5) / 20_000.0;
            let r = (1.0 - z * z).sqrt();
            points.push(glam::vec3(r * (t * 6.283).cos() * 100.0, r * (t * 6.283).sin() * 60.0, z * 30.0));
        }
        let kept = silhouette_points(points.iter().copied());
        assert!(kept.len() <= SILHOUETTE_DIRECTIONS && kept.len() > 100, "{}", kept.len());
        for direction in [glam::Vec3::X, glam::Vec3::Y, glam::Vec3::Z, glam::vec3(1.0, 1.0, 1.0).normalize(), glam::vec3(-0.3, 0.8, 0.5).normalize()] {
            let full = points.iter().map(|p| p.dot(direction)).fold(f32::MIN, f32::max);
            let few = kept.iter().map(|p| p.dot(direction)).fold(f32::MIN, f32::max);
            let radius = points.iter().map(|p| p.dot(direction).abs()).fold(0.0, f32::max);
            assert!(full - few <= 0.01 * radius, "{direction:?}: {full} vs {few}");
        }
    }
}
