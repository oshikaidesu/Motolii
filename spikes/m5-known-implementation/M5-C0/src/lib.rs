use glam::{camera::rh::proj::opengl, Mat4, Vec3};
use thiserror::Error;

/// M5-C0の意味を検証するためだけのprivate fixture型。製品schema/APIではない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionClass {
    PlanarCompatibility,
    Perspective,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthConvention {
    OpenGlMinusOneToOne,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub projection: ProjectionClass,
    pub world_to_clip: Mat4,
    pub depth: DepthConvention,
    pub aspect: f32,
}

impl Observation {
    pub fn project(self, point: Vec3) -> Result<Vec3, ObservationError> {
        if !point.is_finite() {
            return Err(ObservationError::NonFinitePoint);
        }
        let clip = self.world_to_clip * point.extend(1.0);
        if !clip.is_finite() || clip.w.abs() <= f32::EPSILON {
            return Err(ObservationError::InvalidClipW);
        }
        let ndc = clip.truncate() / clip.w;
        if !ndc.is_finite() {
            return Err(ObservationError::InvalidNdc);
        }
        Ok(ndc)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObservationRequest {
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Projective,
}

#[derive(Debug, Error, PartialEq)]
pub enum ObservationError {
    #[error("provider `{provider}` does not support capability {capability:?}")]
    MissingCapability {
        provider: &'static str,
        capability: Capability,
    },
    #[error("provider `{provider}` is missing")]
    ProviderMissing { provider: &'static str },
    #[error("provider `{provider}` version {actual} does not satisfy requested {requested}")]
    ProviderVersionMismatch {
        provider: &'static str,
        requested: u32,
        actual: u32,
    },
    #[error("observation output dimensions must be non-zero")]
    InvalidOutputDimensions,
    #[error("observation clip range is invalid")]
    InvalidClipRange,
    #[error("world point is non-finite")]
    NonFinitePoint,
    #[error("clip w is invalid")]
    InvalidClipW,
    #[error("NDC output is non-finite")]
    InvalidNdc,
}

pub trait Provider {
    fn id(&self) -> &'static str;
    fn version(&self) -> u32;
    fn supports(&self, capability: Capability) -> bool;
    fn observe(&self, request: ObservationRequest) -> Result<Observation, ObservationError>;
}

fn validate_request(request: ObservationRequest) -> Result<f32, ObservationError> {
    if request.width == 0 || request.height == 0 {
        return Err(ObservationError::InvalidOutputDimensions);
    }
    if !request.near.is_finite()
        || !request.far.is_finite()
        || request.near <= 0.0
        || request.far <= request.near
    {
        return Err(ObservationError::InvalidClipRange);
    }
    Ok(request.width as f32 / request.height as f32)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlanarProvider;

impl Provider for PlanarProvider {
    fn id(&self) -> &'static str {
        "fixture.camera.planar"
    }

    fn version(&self) -> u32 {
        1
    }

    fn supports(&self, capability: Capability) -> bool {
        matches!(capability, Capability::Projective)
    }

    fn observe(&self, request: ObservationRequest) -> Result<Observation, ObservationError> {
        let aspect = validate_request(request)?;
        let half_height = 1.0;
        let half_width = half_height * aspect;
        Ok(Observation {
            projection: ProjectionClass::PlanarCompatibility,
            world_to_clip: opengl::orthographic(
                -half_width,
                half_width,
                -half_height,
                half_height,
                request.near,
                request.far,
            ),
            depth: DepthConvention::OpenGlMinusOneToOne,
            aspect,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PerspectiveProvider {
    pub fov_y_radians: f32,
}

impl Provider for PerspectiveProvider {
    fn id(&self) -> &'static str {
        "fixture.camera.perspective"
    }

    fn version(&self) -> u32 {
        1
    }

    fn supports(&self, capability: Capability) -> bool {
        matches!(capability, Capability::Projective)
    }

    fn observe(&self, request: ObservationRequest) -> Result<Observation, ObservationError> {
        let aspect = validate_request(request)?;
        if !self.fov_y_radians.is_finite() || self.fov_y_radians <= 0.0 {
            return Err(ObservationError::InvalidClipRange);
        }
        Ok(Observation {
            projection: ProjectionClass::Perspective,
            world_to_clip: opengl::perspective(
                self.fov_y_radians,
                aspect,
                request.near,
                request.far,
            ),
            depth: DepthConvention::OpenGlMinusOneToOne,
            aspect,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderPin {
    pub id: &'static str,
    pub version: u32,
}

pub fn resolve_observation(
    provider: Option<&dyn Provider>,
    pin: ProviderPin,
    request: ObservationRequest,
) -> Result<Observation, ObservationError> {
    let provider = provider.ok_or(ObservationError::ProviderMissing { provider: pin.id })?;
    if provider.id() != pin.id || provider.version() != pin.version {
        return Err(ObservationError::ProviderVersionMismatch {
            provider: pin.id,
            requested: pin.version,
            actual: provider.version(),
        });
    }
    if !provider.supports(Capability::Projective) {
        return Err(ObservationError::MissingCapability {
            provider: pin.id,
            capability: Capability::Projective,
        });
    }
    provider.observe(request)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub object: u64,
    pub provider: ProviderPin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingTransaction {
    current: Binding,
    undo: Vec<Binding>,
}

impl BindingTransaction {
    pub fn new(current: Binding) -> Self {
        Self {
            current,
            undo: Vec::new(),
        }
    }

    pub fn current(&self) -> Binding {
        self.current
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn replace(
        &mut self,
        provider: Option<&dyn Provider>,
        next: Binding,
        request: ObservationRequest,
    ) -> Result<Observation, ObservationError> {
        let observation = resolve_observation(provider, next.provider, request)?;
        self.undo.push(self.current);
        self.current = next;
        Ok(observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ObservationRequest {
        ObservationRequest {
            width: 1600,
            height: 900,
            near: 0.1,
            far: 100.0,
        }
    }

    #[test]
    fn planar_preserves_compatibility_projection_without_parallax() {
        let observation = PlanarProvider.observe(request()).unwrap();
        assert_eq!(observation.projection, ProjectionClass::PlanarCompatibility);
        let near = observation.project(Vec3::new(0.5, 0.0, -1.0)).unwrap();
        let far = observation.project(Vec3::new(0.5, 0.0, -2.0)).unwrap();
        assert!((near.x - far.x).abs() < 1e-6);
    }

    #[test]
    fn perspective_is_projective_and_exhibits_depth_parallax() {
        let provider = PerspectiveProvider {
            fov_y_radians: 60.0_f32.to_radians(),
        };
        let observation = provider.observe(request()).unwrap();
        assert_eq!(observation.projection, ProjectionClass::Perspective);
        let near = observation.project(Vec3::new(0.5, 0.0, -1.0)).unwrap();
        let far = observation.project(Vec3::new(0.5, 0.0, -2.0)).unwrap();
        assert!(near.x > far.x);
    }

    #[test]
    fn missing_provider_and_version_mismatch_are_typed_failures() {
        let pin = ProviderPin {
            id: "fixture.camera.perspective",
            version: 2,
        };
        assert_eq!(
            resolve_observation(None, pin, request()),
            Err(ObservationError::ProviderMissing { provider: pin.id })
        );
        let provider = PerspectiveProvider { fov_y_radians: 1.0 };
        assert_eq!(
            resolve_observation(Some(&provider), pin, request()),
            Err(ObservationError::ProviderVersionMismatch {
                provider: pin.id,
                requested: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn failed_provider_swap_does_not_mutate_binding_or_undo() {
        let planar = PlanarProvider;
        let current = Binding {
            object: 7,
            provider: ProviderPin {
                id: planar.id(),
                version: planar.version(),
            },
        };
        let mut transaction = BindingTransaction::new(current);
        let failed = Binding {
            object: 7,
            provider: ProviderPin {
                id: "fixture.camera.perspective",
                version: 9,
            },
        };
        assert!(transaction.replace(None, failed, request()).is_err());
        assert_eq!(transaction.current(), current);
        assert_eq!(transaction.undo_len(), 0);

        let perspective = PerspectiveProvider { fov_y_radians: 1.0 };
        let next = Binding {
            object: 7,
            provider: ProviderPin {
                id: perspective.id(),
                version: perspective.version(),
            },
        };
        transaction
            .replace(Some(&perspective), next, request())
            .unwrap();
        assert_eq!(transaction.current(), next);
        assert_eq!(transaction.undo_len(), 1);
    }

    #[test]
    fn invalid_request_is_rejected_before_provider_output() {
        let provider = PlanarProvider;
        let invalid = ObservationRequest {
            width: 0,
            ..request()
        };
        assert_eq!(
            provider.observe(invalid),
            Err(ObservationError::InvalidOutputDimensions)
        );
    }
}
