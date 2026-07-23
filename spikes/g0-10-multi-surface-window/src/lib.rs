use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowRole {
    Editor,
    DetachedPreview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostSnapshot {
    pub shape_id: &'static str,
    pub selected_id: &'static str,
    pub shape_count: u32,
}

impl Default for HostSnapshot {
    fn default() -> Self {
        Self {
            shape_id: "shape-spike-stable-0001",
            selected_id: "shape-spike-stable-0001",
            shape_count: 1,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RoleTelemetry {
    pub window_generation: u32,
    pub present_count: u64,
    pub injected_surface_lost_count: u32,
    pub actual_surface_reconfigure_count: u32,
    pub surface_reconfigure_count: u32,
    pub close_count: u32,
    pub reopen_count: u32,
    pub focus_gained_count: u32,
    pub focus_lost_count: u32,
    pub fullscreen_enter_observed_count: u32,
    pub fullscreen_exit_observed_count: u32,
    pub resize_event_count: u32,
    pub scale_factor_event_count: u32,
    pub layout_epoch: u64,
    pub last_scale_factor: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LifecycleLedger {
    pub host_snapshot: HostSnapshot,
    pub baseline_snapshot: HostSnapshot,
    pub editor: RoleTelemetry,
    pub detached_preview: RoleTelemetry,
    pub preview_open: bool,
    pub surface_count_peak: u32,
    pub editor_present_at_preview_close: u64,
}

impl Default for LifecycleLedger {
    fn default() -> Self {
        let host_snapshot = HostSnapshot::default();
        Self {
            baseline_snapshot: host_snapshot.clone(),
            host_snapshot,
            editor: RoleTelemetry::default(),
            detached_preview: RoleTelemetry::default(),
            preview_open: false,
            surface_count_peak: 0,
            editor_present_at_preview_close: 0,
        }
    }
}

impl LifecycleLedger {
    pub fn role(&self, role: WindowRole) -> &RoleTelemetry {
        match role {
            WindowRole::Editor => &self.editor,
            WindowRole::DetachedPreview => &self.detached_preview,
        }
    }

    pub fn role_mut(&mut self, role: WindowRole) -> &mut RoleTelemetry {
        match role {
            WindowRole::Editor => &mut self.editor,
            WindowRole::DetachedPreview => &mut self.detached_preview,
        }
    }

    pub fn opened(&mut self, role: WindowRole, scale_factor: f64) {
        let generation = {
            let telemetry = self.role_mut(role);
            telemetry.window_generation += 1;
            telemetry.last_scale_factor = scale_factor;
            telemetry.layout_epoch += 1;
            telemetry.window_generation
        };
        if role == WindowRole::DetachedPreview {
            self.preview_open = true;
            if generation > 1 {
                self.detached_preview.reopen_count += 1;
            }
        }
        let current = 1 + u32::from(self.preview_open);
        self.surface_count_peak = self.surface_count_peak.max(current);
    }

    pub fn close_preview(&mut self) {
        if self.preview_open {
            self.preview_open = false;
            self.detached_preview.close_count += 1;
            self.editor_present_at_preview_close = self.editor.present_count;
        }
    }

    pub fn record_present(&mut self, role: WindowRole) {
        self.role_mut(role).present_count += 1;
    }

    pub fn record_injected_loss(&mut self, role: WindowRole) {
        let telemetry = self.role_mut(role);
        telemetry.injected_surface_lost_count += 1;
        telemetry.surface_reconfigure_count += 1;
    }

    pub fn record_actual_reconfigure(&mut self, role: WindowRole) {
        let telemetry = self.role_mut(role);
        telemetry.actual_surface_reconfigure_count += 1;
        telemetry.surface_reconfigure_count += 1;
    }

    pub fn record_focus(&mut self, role: WindowRole, focused: bool) {
        let telemetry = self.role_mut(role);
        if focused {
            telemetry.focus_gained_count += 1;
        } else {
            telemetry.focus_lost_count += 1;
        }
    }

    pub fn record_fullscreen(&mut self, role: WindowRole, entered: bool) {
        let telemetry = self.role_mut(role);
        if entered {
            telemetry.fullscreen_enter_observed_count += 1;
        } else {
            telemetry.fullscreen_exit_observed_count += 1;
        }
    }

    pub fn record_resize(&mut self, role: WindowRole, scale_factor: f64) {
        let telemetry = self.role_mut(role);
        telemetry.resize_event_count += 1;
        telemetry.last_scale_factor = scale_factor;
        telemetry.layout_epoch += 1;
    }

    pub fn record_scale_factor(&mut self, role: WindowRole, scale_factor: f64) {
        let telemetry = self.role_mut(role);
        telemetry.scale_factor_event_count += 1;
        telemetry.last_scale_factor = scale_factor;
        telemetry.layout_epoch += 1;
    }

    pub fn host_state_preserved(&self) -> bool {
        self.host_snapshot == self.baseline_snapshot
    }

    pub fn fault_isolated_to_preview(&self) -> bool {
        self.detached_preview.injected_surface_lost_count == 1
            && self.detached_preview.surface_reconfigure_count >= 1
            && self.editor.injected_surface_lost_count == 0
            && self.editor.surface_reconfigure_count == 0
    }

    pub fn editor_presented_after_preview_close(&self) -> bool {
        self.editor.present_count > self.editor_present_at_preview_close
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_close_and_reopen_preserve_host_snapshot() {
        let mut ledger = LifecycleLedger::default();
        ledger.opened(WindowRole::Editor, 2.0);
        ledger.opened(WindowRole::DetachedPreview, 2.0);
        ledger.record_present(WindowRole::Editor);
        ledger.close_preview();
        ledger.record_present(WindowRole::Editor);
        ledger.opened(WindowRole::DetachedPreview, 2.0);

        assert!(ledger.host_state_preserved());
        assert!(ledger.editor_presented_after_preview_close());
        assert_eq!(ledger.detached_preview.close_count, 1);
        assert_eq!(ledger.detached_preview.reopen_count, 1);
        assert_eq!(ledger.detached_preview.window_generation, 2);
    }

    #[test]
    fn injected_preview_loss_does_not_reconfigure_editor() {
        let mut ledger = LifecycleLedger::default();
        ledger.record_injected_loss(WindowRole::DetachedPreview);

        assert!(ledger.fault_isolated_to_preview());
        assert_eq!(ledger.editor.surface_reconfigure_count, 0);
    }

    #[test]
    fn layout_epoch_tracks_resize_and_scale_without_touching_host_state() {
        let mut ledger = LifecycleLedger::default();
        ledger.opened(WindowRole::Editor, 1.0);
        ledger.record_resize(WindowRole::Editor, 1.0);
        ledger.record_scale_factor(WindowRole::Editor, 2.0);

        assert_eq!(ledger.editor.layout_epoch, 3);
        assert_eq!(ledger.editor.scale_factor_event_count, 1);
        assert_eq!(ledger.editor.last_scale_factor, 2.0);
        assert!(ledger.host_state_preserved());
    }
}
