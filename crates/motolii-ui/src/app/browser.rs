//! Browser Hostのplace候補をStage dropまで運び、表示中cameraで正準座標へ戻す。

use motolii_core::{CanonicalPoint, CompCamera};

use crate::document_edit_runtime::PlaceRectangleRequest;
use crate::host_pointer_capture::HostPointerCandidate;
use crate::layout_runtime_adapter::StageDropTerminal;

use super::MotoliiApp;

impl MotoliiApp {
    pub(super) fn begin_browser_place(&mut self) {
        if self.active_browser_place.is_some() || self.browser_host_failure.is_some() {
            return;
        }
        let Some(host) = &self.browser_host else {
            return;
        };
        let generation = self.browser_place_generation.wrapping_add(1);
        match host.take_place_intent(generation) {
            Ok(Some((intent, generation))) => {
                self.browser_place_generation = generation;
                self.active_browser_place = Some(intent);
            }
            Ok(None) => {}
            Err(error) => self.browser_host_failure = Some(error.to_string()),
        }
    }

    pub(super) fn poll_browser_place_pointer(&mut self) {
        if self.active_browser_place.is_none() || self.browser_host_failure.is_some() {
            return;
        }
        let Some(host) = &self.browser_host else {
            return;
        };
        match host.poll_pointer_candidate() {
            Ok(Some(HostPointerCandidate::Moved { .. })) => {
                // WebKit tracking loop外でもreleaseを観測できるよう、active中はpollを継続する。
                self.repaint_context.request_repaint();
            }
            Ok(Some(HostPointerCandidate::Released { .. }))
            | Ok(Some(HostPointerCandidate::Cancelled { .. })) => {
                // 本粒はcandidate取得まで。Stage admissionとD2は後続責任へ渡す。
                self.active_browser_place = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.active_browser_place = None;
                self.browser_host_failure = Some(error.to_string());
            }
        }
    }

    pub(super) fn finish_browser_place(&mut self, terminal: StageDropTerminal) {
        let Some(_intent) = self.active_browser_place.take() else {
            return;
        };
        let StageDropTerminal::Commit { ndc } = terminal else {
            return;
        };
        let Some(camera) = self.latest_camera else {
            self.browser_host_failure =
                Some("Rectangle drop has no displayed camera projection".to_owned());
            return;
        };
        let Some(position) = canonical_drop_from_ndc(camera, ndc) else {
            self.browser_host_failure =
                Some("Rectangle drop could not be converted to canonical coordinates".to_owned());
            return;
        };
        self.document_queue
            .push_place_rectangle(PlaceRectangleRequest {
                position,
                playhead: self.render_request_template.evaluation_time.timeline_time,
            });
        self.repaint_context.request_repaint();
    }
}

pub(crate) fn canonical_drop_from_ndc(camera: CompCamera, ndc: [f64; 2]) -> Option<[f64; 2]> {
    if !ndc[0].is_finite() || !ndc[1].is_finite() {
        return None;
    }
    let qx =
        ndc[0] * camera.aspect_num() as f64 / camera.aspect_den() as f64 * camera.height() / 2.0;
    let qy = ndc[1] * camera.height() / 2.0;
    let cos_r = camera.roll_radians().cos();
    let sin_r = camera.roll_radians().sin();
    let center = camera.center();
    let point = CanonicalPoint {
        x: center.x + cos_r * qx - sin_r * qy,
        y: center.y + sin_r * qx + cos_r * qy,
    };
    let projected = camera.world_to_ndc(point).ok()?;
    if !point.x.is_finite()
        || !point.y.is_finite()
        || (projected.0 - ndc[0]).abs() > 1e-9
        || (projected.1 - ndc[1]).abs() > 1e-9
    {
        return None;
    }
    Some([point.x, point.y])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_ndc_drop_roundtrips_through_the_displayed_camera() {
        let camera =
            CompCamera::try_new(CanonicalPoint { x: 0.3, y: -0.1 }, 0.4, 1.5, 16, 9).unwrap();
        let ndc = [0.25, -0.5];
        let position = canonical_drop_from_ndc(camera, ndc).expect("canonical position");
        let projected = camera
            .world_to_ndc(CanonicalPoint {
                x: position[0],
                y: position[1],
            })
            .unwrap();
        assert!((projected.0 - ndc[0]).abs() < 1e-9);
        assert!((projected.1 - ndc[1]).abs() < 1e-9);
    }
}
