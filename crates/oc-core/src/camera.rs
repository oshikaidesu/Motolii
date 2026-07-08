use serde::{Deserialize, Serialize};

/// コンポ全体で共有されるv1カメラ文脈。
///
/// 3D系レイヤーソースはこの値を参照して内部で投影し、最終的には
/// premultiplied RGBAテクスチャを返す。レイヤーごとのカメラはv1では持たない。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompCamera {
    pub position: [f64; 3],
    pub target: [f64; 3],
    /// 垂直画角。単位は度。
    pub fov_y_degrees: f64,
    /// カメラのロール。単位は度。
    pub roll_degrees: f64,
}

impl CompCamera {
    pub const DEFAULT: Self = Self {
        position: [0.0, 0.0, 2.0],
        target: [0.0, 0.0, 0.0],
        fov_y_degrees: 45.0,
        roll_degrees: 0.0,
    };

    pub fn validate(&self) -> Result<(), String> {
        if !self.position.iter().all(|v| v.is_finite())
            || !self.target.iter().all(|v| v.is_finite())
            || !self.fov_y_degrees.is_finite()
            || !self.roll_degrees.is_finite()
        {
            return Err("camera contains non-finite value".into());
        }
        if !(0.0 < self.fov_y_degrees && self.fov_y_degrees < 180.0) {
            return Err("camera fov_y_degrees must be in (0, 180)".into());
        }
        if self.position == self.target {
            return Err("camera position and target must differ".into());
        }
        Ok(())
    }
}

impl Default for CompCamera {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera_is_valid() {
        assert!(CompCamera::DEFAULT.validate().is_ok());
    }

    #[test]
    fn camera_rejects_degenerate_view() {
        let camera = CompCamera {
            position: [0.0, 0.0, 0.0],
            target: [0.0, 0.0, 0.0],
            ..CompCamera::DEFAULT
        };
        assert!(camera.validate().is_err());
    }

    #[test]
    fn camera_rejects_invalid_fov() {
        let camera = CompCamera {
            fov_y_degrees: 180.0,
            ..CompCamera::DEFAULT
        };
        assert!(camera.validate().is_err());
    }
}
