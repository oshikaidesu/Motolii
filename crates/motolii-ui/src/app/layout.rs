//! shell chromeとpanel配置。Document意味やpreview世代はここでは決めない。

use egui_tiles::{Behavior, EditAction, Tile, TileId, Tiles, UiResponse};

use crate::layout::{LayoutAction, LayoutConstraints, PanelRole, SeparatorAction};
use crate::layout_authority::RuntimeFrameEdit;
use crate::layout_runtime::{RuntimeLayout, RuntimeSeparator};
use crate::layout_runtime_adapter::{
    read_layout_cancel, read_safety_interrupt, read_separator_action, read_stage_drop_terminal,
    StageDropTerminal,
};
use crate::static_preview::StaticPreview;
use crate::{ImeGateState, NormalizedInput};

use super::lifecycle::{LifecycleInvariantError, ShellLifecycleInput};
use super::MotoliiApp;

const DEFAULT_STAGE_MIN_POINTS: f32 = 320.0;

impl MotoliiApp {
    pub(super) fn paint_shell(&mut self, ui: &mut egui::Ui) {
        self.paint_count = self.paint_count.saturating_add(1);
        let available = ui.available_size();
        for input in [
            ShellLifecycleInput::Resized([available.x, available.y]),
            ShellLifecycleInput::ScaleFactorChanged(ui.ctx().pixels_per_point()),
        ] {
            if self.projection.observe(input, &self.preview).is_err() {
                self.record_smoke_failure(LifecycleInvariantError.to_string());
            }
        }

        let mut requested_action = None;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("View", |ui| {
                for role in PanelRole::AUXILIARY {
                    let visible = self.layout_authority.intent().is_visible(role);
                    let (_, action) = view_role_button(ui, role, visible);
                    if let Some(action) = action {
                        requested_action = Some(action);
                        ui.close();
                    }
                }
                if ui.button("Reset layout").clicked() {
                    requested_action = Some(LayoutAction::ResetPreset);
                    ui.close();
                }
            });
        });

        egui::Panel::bottom("motolii-status")
            .resizable(false)
            .show(ui, |ui| {
                ui.label("Status");
            });

        if self.browser_host.is_some() {
            let browser_panel = egui::Panel::left("motolii-browser-host")
                .resizable(false)
                .default_size(420.0)
                .min_size(420.0)
                .max_size(420.0)
                .show(ui, |_ui| {});
            if self.browser_host_failure.is_none() {
                if let Some(Err(error)) = self.browser_host.as_ref().map(|host| {
                    let rect = browser_panel.response.rect;
                    host.set_bounds(
                        u64::from(self.paint_count),
                        crate::native_host_layout::LogicalRect {
                            x: f64::from(rect.left()),
                            y: f64::from(rect.top()),
                            width: f64::from(rect.width()),
                            height: f64::from(rect.height()),
                        },
                    )
                }) {
                    self.browser_host_failure = Some(error.to_string());
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let constraints = layout_constraints(ui.available_width());
            let layout_action_requested = requested_action.is_some();
            if let Some(action) = requested_action.take() {
                if let Err(error) = self.layout_authority.apply(action, constraints) {
                    self.observe_layout_failure(error);
                }
            }

            let safety = read_safety_interrupt(ui);
            if let Some(safety) = safety {
                if let Err(error) = self
                    .input_router
                    .route(NormalizedInput::SafetyInterrupt(safety))
                {
                    self.observe_layout_failure_message(error.to_string());
                }
            }
            let cancel_runtime_frame = safety.is_some()
                || read_layout_cancel(ui, self.layout_authority.gesture_in_flight(), self.ime_gate);

            let timeline_projection =
                crate::timeline_egui::project_for_egui(&self.current_document);
            let timeline_projection_error =
                timeline_projection.as_ref().err().map(ToString::to_string);
            let (timeline_intents, edits, visibility_edited, stage_rect) = {
                let paint_projection = self
                    .timeline_preview
                    .as_ref()
                    .or(timeline_projection.as_ref().ok());
                let mut behavior = PanelBehavior {
                    preview: &self.preview,
                    texture_id: self.texture_id,
                    edits: Vec::new(),
                    visibility_edited: false,
                    stage_rect: None,
                    timeline_document: &self.current_document,
                    timeline_projection: paint_projection,
                    timeline_projection_error,
                    timeline_primary: &mut self.primary,
                    timeline_playhead: &mut self
                        .render_request_template
                        .evaluation_time
                        .timeline_time,
                    timeline_intents: Vec::new(),
                };
                self.layout_authority
                    .runtime_mut()
                    .tree_mut()
                    .ui(&mut behavior, ui);
                (
                    std::mem::take(&mut behavior.timeline_intents),
                    std::mem::take(&mut behavior.edits),
                    behavior.visibility_edited,
                    behavior.stage_rect,
                )
            };
            for intent in timeline_intents {
                match intent {
                    crate::timeline_egui::TimelineIntent::Pointer {
                        phase, time, hit, ..
                    } => self.handle_timeline_pointer(
                        phase,
                        time,
                        hit,
                        timeline_projection.as_ref().ok(),
                    ),
                    crate::timeline_egui::TimelineIntent::Command { command, .. } => {
                        let mapped = match command {
                            crate::timeline_egui::TimelineCommand::Undo => {
                                Some(crate::timeline_intent_adapter::TimelineIntent::Undo)
                            }
                            crate::timeline_egui::TimelineCommand::Redo => {
                                Some(crate::timeline_intent_adapter::TimelineIntent::Redo)
                            }
                            crate::timeline_egui::TimelineCommand::Duplicate => self.primary.map(
                                crate::timeline_intent_adapter::TimelineIntent::DuplicateLayer,
                            ),
                            _ => None,
                        };
                        if let Some(mapped) = mapped {
                            let _ = crate::timeline_intent_adapter::enqueue_timeline_intent(
                                &mut self.document_queue,
                                mapped,
                            );
                        }
                    }
                    _ => {}
                }
            }
            if !self.layout_evidence_logged {
                eprintln!(
                    "U1A2_LAYOUT signature={}",
                    self.layout_authority.intent().canonical_signature()
                );
                self.layout_evidence_logged = true;
            }

            let runtime_edit = if edits
                .iter()
                .any(|edit| matches!(edit, EditAction::TileResized | EditAction::TileDragged))
            {
                RuntimeFrameEdit::Continuous
            } else if !edits.is_empty() || visibility_edited {
                RuntimeFrameEdit::Commit
            } else {
                RuntimeFrameEdit::None
            };
            let gesture_finished =
                edits.contains(&EditAction::TileDropped) || ui.ctx().drag_stopped_id().is_some();

            if cancel_runtime_frame {
                if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                    true,
                    runtime_edit,
                    gesture_finished,
                    constraints,
                ) {
                    self.observe_layout_failure(error);
                }
                return;
            }

            let separator_actions =
                collect_separator_actions(ui, self.layout_authority.runtime(), self.ime_gate);
            let separator_consumed_runtime_edit = !separator_actions.is_empty();
            if self.active_browser_place.is_some() {
                let layout_changed = runtime_edit != RuntimeFrameEdit::None
                    || layout_action_requested
                    || separator_consumed_runtime_edit;
                let terminal = if layout_changed {
                    Some(StageDropTerminal::Cancel)
                } else {
                    stage_rect.and_then(|rect| read_stage_drop_terminal(ui, rect, self.ime_gate))
                };
                if let Some(terminal) = terminal {
                    self.finish_browser_place(terminal);
                }
            }
            for (separator, action) in separator_actions {
                if action == SeparatorAction::Cancel {
                    if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                        true,
                        RuntimeFrameEdit::None,
                        false,
                        constraints,
                    ) {
                        self.observe_layout_failure(error);
                    }
                    continue;
                }
                if let Err(error) = self.layout_authority.apply(
                    LayoutAction::Separator {
                        path: separator.path,
                        boundary: separator.boundary,
                        action,
                    },
                    constraints,
                ) {
                    self.observe_layout_failure(error);
                }
            }
            if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                false,
                runtime_edit_after_separator_action(runtime_edit, separator_consumed_runtime_edit),
                gesture_finished,
                constraints,
            ) {
                self.observe_layout_failure(error);
            }
        });
    }

    fn observe_layout_failure(&mut self, error: crate::layout::LayoutError) {
        self.observe_layout_failure_message(error.to_string());
    }

    fn observe_layout_failure_message(&mut self, message: String) {
        eprintln!("U1A2_LAYOUT_REJECT error={message}");
        self.layout_failure = Some(message);
    }
}

fn runtime_edit_after_separator_action(
    runtime_edit: RuntimeFrameEdit,
    separator_action_consumed: bool,
) -> RuntimeFrameEdit {
    if separator_action_consumed {
        RuntimeFrameEdit::None
    } else {
        runtime_edit
    }
}

struct PanelBehavior<'a> {
    preview: &'a StaticPreview,
    texture_id: egui::TextureId,
    edits: Vec<EditAction>,
    visibility_edited: bool,
    stage_rect: Option<egui::Rect>,
    timeline_document: &'a motolii_doc::Document,
    timeline_projection: Option<&'a crate::timeline_projection::TimelineProjection>,
    timeline_projection_error: Option<String>,
    timeline_primary: &'a mut Option<motolii_doc::LayerId>,
    timeline_playhead: &'a mut motolii_core::RationalTime,
    timeline_intents: Vec<crate::timeline_egui::TimelineIntent>,
}

impl Behavior<PanelRole> for PanelBehavior<'_> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut PanelRole) -> UiResponse {
        match pane {
            PanelRole::Stage => {
                self.stage_rect = Some(paint_stage(ui, self.preview, self.texture_id));
            }
            PanelRole::Timeline => {
                let output = crate::timeline_egui::paint_timeline(
                    ui,
                    self.timeline_document,
                    self.timeline_projection,
                    self.timeline_projection_error.as_deref(),
                    self.timeline_primary,
                    self.timeline_playhead,
                );
                self.timeline_intents.extend(output.intents);
            }
            role => {
                let response = ui.add(egui::Label::new(role.title()).sense(egui::Sense::drag()));
                if response.drag_started() {
                    return UiResponse::DragStarted;
                }
            }
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &PanelRole) -> egui::WidgetText {
        pane.title().into()
    }

    fn is_tab_closable(&self, tiles: &Tiles<PanelRole>, tile_id: TileId) -> bool {
        tiles
            .get_pane(&tile_id)
            .is_some_and(|role| role.is_auxiliary())
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<PanelRole>, tile_id: TileId) -> bool {
        if tiles
            .get_pane(&tile_id)
            .is_some_and(|role| role.is_auxiliary())
        {
            tiles.set_visible(tile_id, false);
            self.visibility_edited = true;
        }
        false
    }

    fn is_tile_draggable(&self, tiles: &Tiles<PanelRole>, tile_id: TileId) -> bool {
        matches!(
            tiles.get(tile_id),
            Some(Tile::Pane(role)) if role.is_auxiliary()
        )
    }

    fn on_edit(&mut self, edit_action: EditAction) {
        self.edits.push(edit_action);
    }
}

fn paint_stage(
    ui: &mut egui::Ui,
    preview: &StaticPreview,
    texture_id: egui::TextureId,
) -> egui::Rect {
    let desc = preview.slot().desc();
    let source_size = egui::vec2(desc.width as f32, desc.height as f32);
    let target_size = fit_inside(source_size, ui.available_size());
    let rect = egui::Rect::from_center_size(ui.max_rect().center(), target_size);
    ui.push_id("motolii-stage-viewport", |ui| {
        ui.put(
            rect,
            egui::Image::from_texture((texture_id, source_size)).fit_to_exact_size(target_size),
        );
    });
    rect
}

fn layout_constraints(viewport_width: f32) -> LayoutConstraints {
    let safe_width = viewport_width.max(2.0);
    LayoutConstraints {
        viewport_width: safe_width,
        stage_min_width: DEFAULT_STAGE_MIN_POINTS.min(safe_width * 0.75),
    }
}

fn view_role_button(
    ui: &mut egui::Ui,
    role: PanelRole,
    visible: bool,
) -> (egui::Response, Option<LayoutAction>) {
    let response = ui.button(if visible {
        format!("Hide {}", role.title())
    } else {
        format!("Restore {}", role.title())
    });
    let action = response.clicked().then_some(if visible {
        LayoutAction::Hide(role)
    } else {
        LayoutAction::Restore(role)
    });
    (response, action)
}

fn collect_separator_actions(
    ui: &mut egui::Ui,
    runtime: &RuntimeLayout,
    ime_gate: ImeGateState,
) -> Vec<(RuntimeSeparator, SeparatorAction)> {
    let mut actions = Vec::new();
    for separator in runtime.separators().iter().cloned() {
        let Some(response) = runtime.separator_response(ui, &separator) else {
            continue;
        };
        if let Some(action) = read_separator_action(ui, &response, separator.axis, ime_gate) {
            actions.push((separator, action));
        }
    }
    actions
}

fn fit_inside(source: egui::Vec2, available: egui::Vec2) -> egui::Vec2 {
    if source.x <= 0.0 || source.y <= 0.0 || available.x <= 0.0 || available.y <= 0.0 {
        return egui::Vec2::ZERO;
    }
    let scale = (available.x / source.x).min(available.y / source.y);
    source * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_authority::LayoutAuthority;

    #[test]
    fn fit_inside_preserves_aspect_without_window_state() {
        assert_eq!(
            fit_inside(egui::vec2(16.0, 9.0), egui::vec2(800.0, 600.0)),
            egui::vec2(800.0, 450.0)
        );
        assert_eq!(
            fit_inside(egui::vec2(16.0, 9.0), egui::vec2(320.0, 100.0)),
            egui::vec2(1600.0 / 9.0, 100.0)
        );
    }

    #[test]
    fn hidden_role_restores_through_the_product_view_button_with_enter() {
        let constraints = layout_constraints(1_000.0);
        let mut authority = LayoutAuthority::built_in().unwrap();
        authority
            .apply(LayoutAction::Hide(PanelRole::Browser), constraints)
            .unwrap();
        let context = egui::Context::default();
        let _ = context.run_ui(Default::default(), |ui| {
            let (response, action) = view_role_button(ui, PanelRole::Browser, false);
            assert!(action.is_none());
            response.request_focus();
        });
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let _ = context.run_ui(input, |ui| {
            let (_, action) = view_role_button(ui, PanelRole::Browser, false);
            authority.apply(action.unwrap(), constraints).unwrap();
        });
        assert!(authority.intent().is_visible(PanelRole::Browser));
    }

    #[test]
    fn native_double_click_reset_suppresses_tiles_mean_proposal() {
        assert_eq!(
            runtime_edit_after_separator_action(RuntimeFrameEdit::Continuous, true),
            RuntimeFrameEdit::None
        );
        assert_eq!(
            runtime_edit_after_separator_action(RuntimeFrameEdit::Continuous, false),
            RuntimeFrameEdit::Continuous
        );
    }
}
