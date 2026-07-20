use crate::{components, theme};
use eframe::egui::{
    self, Align, Color32, FontId, Layout, RichText, Sense, Stroke, StrokeKind, Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InspectorEffect {
    EchoBloom,
    TypePulse,
    FoldField,
}

#[derive(Debug, Clone)]
pub(crate) struct InspectorState {
    pub(crate) selected_effect: InspectorEffect,
    pub(crate) intensity: f32,
    pub(crate) spread: f32,
    pub(crate) blend: BlendMode,
    pub(crate) intensity_automated: bool,
    pub(crate) spread_automated: bool,
    pub(crate) developer_info_open: bool,
    drag_original: Option<InspectorSnapshot>,
    drag_cancelled: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            selected_effect: InspectorEffect::EchoBloom,
            intensity: 0.64,
            spread: 0.42,
            blend: BlendMode::Screen,
            intensity_automated: true,
            spread_automated: false,
            developer_info_open: false,
            drag_original: None,
            drag_cancelled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InspectorSnapshot {
    intensity: f32,
    spread: f32,
    blend: BlendMode,
    intensity_automated: bool,
    spread_automated: bool,
}

impl InspectorState {
    pub(crate) fn snapshot(&self) -> InspectorSnapshot {
        InspectorSnapshot {
            intensity: self.intensity,
            spread: self.spread,
            blend: self.blend,
            intensity_automated: self.intensity_automated,
            spread_automated: self.spread_automated,
        }
    }

    pub(crate) fn restore(&mut self, snapshot: InspectorSnapshot) {
        self.intensity = snapshot.intensity;
        self.spread = snapshot.spread;
        self.blend = snapshot.blend;
        self.intensity_automated = snapshot.intensity_automated;
        self.spread_automated = snapshot.spread_automated;
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InspectorAction {
    Commit {
        label: String,
        before: InspectorSnapshot,
    },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlendMode {
    Screen,
    Add,
    Overlay,
}

impl BlendMode {
    fn label(self) -> &'static str {
        match self {
            Self::Screen => "Screen",
            Self::Add => "Add",
            Self::Overlay => "Overlay",
        }
    }
}

pub(crate) fn inspector_ui(
    ui: &mut egui::Ui,
    state: &mut InspectorState,
) -> Option<InspectorAction> {
    let before = state.snapshot();
    let mut action = None;
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        if let Some(original) = state.drag_original {
            state.restore(original);
            state.drag_cancelled = true;
            action = Some(InspectorAction::Cancelled);
        }
    }
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::PANEL);
    panel_header(ui);

    let fixture = effect_fixture(state.selected_effect);
    identity_section(ui, fixture);

    match state.selected_effect {
        InspectorEffect::EchoBloom => {
            let panel_action = echo_bloom_panel(ui, state, fixture, before);
            action = action.or(panel_action);
        }
        InspectorEffect::TypePulse => type_pulse_panel(ui, fixture),
        InspectorEffect::FoldField => fold_field_panel(ui, fixture),
    }

    developer_info(ui, state, fixture);
    action
}

fn panel_header(ui: &mut egui::Ui) {
    components::panel_header(ui, "Inspector", "", theme::WAY_INSPECTOR);
}

fn identity_section(ui: &mut egui::Ui, fixture: EffectFixture) {
    section_title(ui, "EDITING EFFECT", fixture.section_state, None);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 58.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add_space(9.0);
            let (icon, _) = ui.allocate_exact_size(Vec2::splat(38.0), Sense::hover());
            ui.painter().rect_stroke(
                icon,
                0.0,
                Stroke::new(1.0, fixture.accent),
                StrokeKind::Inside,
            );
            ui.painter().text(
                icon.center(),
                egui::Align2::CENTER_CENTER,
                fixture.glyph,
                FontId::monospace(12.0),
                fixture.accent,
            );
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.add_space(10.0);
                ui.label(RichText::new(fixture.name).font(theme::interface_bold_font(13.0)));
                ui.label(
                    RichText::new(fixture.subtitle)
                        .monospace()
                        .size(8.0)
                        .color(theme::TEXT_MUTED),
                );
            });
        },
    );
    horizontal_rule(ui);
}

fn echo_bloom_panel(
    ui: &mut egui::Ui,
    state: &mut InspectorState,
    fixture: EffectFixture,
    before: InspectorSnapshot,
) -> Option<InspectorAction> {
    section_title(ui, "ECHO BLOOM", "HOST PANEL", None);
    description(
        ui,
        "Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage.",
    );
    value_row(ui, "Input", "Pulse rings composite", Some("TEXTURE"));
    let mut action = scrub_row(
        ui,
        "Intensity",
        &mut state.intensity,
        &mut state.intensity_automated,
        &mut state.drag_original,
        &mut state.drag_cancelled,
        before,
    );
    let spread_action = scrub_row(
        ui,
        "Spread",
        &mut state.spread,
        &mut state.spread_automated,
        &mut state.drag_original,
        &mut state.drag_cancelled,
        before,
    );
    action = action.or(spread_action);
    if blend_row(ui, &mut state.blend) {
        action = Some(InspectorAction::Commit {
            label: "Blend".into(),
            before,
        });
    }
    let _ = fixture;
    action
}

fn type_pulse_panel(ui: &mut egui::Ui, fixture: EffectFixture) {
    section_title(ui, "TYPE PULSE", "HOST PANEL", None);
    description(
        ui,
        "Kinetic text motion for the selected object. The Browser selection is preview-only in this parity fixture.",
    );
    value_row(ui, "Input", "NIGHT DRIVE", Some("TEXT"));
    value_row(ui, "Motion", "Kinetic pulse", Some("12 KEYS"));
    value_row(ui, "Type", "Typography", Some("EFFECT"));
    notice(
        ui,
        "Installed",
        "Select or apply the effect to expose its editable host parameters.",
        fixture.accent,
    );
}

fn fold_field_panel(ui: &mut egui::Ui, fixture: EffectFixture) {
    section_title(ui, "DISCOVERY", "UNAVAILABLE", Some(theme::WARNING));
    notice(
        ui,
        "このHostでは評価できません",
        "要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。",
        theme::WARNING,
    );
    value_row(ui, "Project change", "NONE", None);
    value_row(ui, "Install", "NOT STARTED", None);
    value_row(ui, "Fallback", "REFUSED", None);
    let _ = fixture;
}

fn section_title(ui: &mut egui::Ui, left: &str, right: &str, right_color: Option<Color32>) {
    components::section_header(ui, left, right, right_color);
}

fn description(ui: &mut egui::Ui, text: &str) {
    let frame = egui::Frame::NONE
        .fill(theme::APP)
        .inner_margin(egui::Margin::symmetric(8, 7))
        .stroke(Stroke::new(0.0, Color32::TRANSPARENT));
    frame.show(ui, |ui| {
        ui.label(RichText::new(text).size(10.0).color(theme::TEXT_SECONDARY));
    });
    horizontal_rule(ui);
}

fn value_row(ui: &mut egui::Ui, label: &str, value: &str, tag: Option<&str>) {
    property_row(ui, |ui| {
        property_label(ui, label);
        value_box(ui, value);
        if let Some(tag) = tag {
            tag_label(ui, tag, theme::ACCENT);
        }
    });
}

fn scrub_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    automated: &mut bool,
    drag_original: &mut Option<InspectorSnapshot>,
    drag_cancelled: &mut bool,
    before: InspectorSnapshot,
) -> Option<InspectorAction> {
    let mut action = None;
    property_row(ui, |ui| {
        let (group, _) = ui.allocate_exact_size(Vec2::new(92.0, 21.0), Sense::hover());
        ui.painter().text(
            group.left_center(),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(10.0),
            theme::TEXT_SECONDARY,
        );
        let automation_offset = if label == "Spread" { 39.0 } else { 47.0 };
        let automation = egui::Rect::from_min_size(
            group.left_top() + Vec2::new(automation_offset, 1.5),
            Vec2::splat(18.0),
        );
        if components::automation_mark_at(
            ui,
            automation,
            *automated,
            &format!("{label} automation"),
        )
        .clicked()
        {
            *automated = !*automated;
            action = Some(InspectorAction::Commit {
                label: format!("{label} automation"),
                before,
            });
        }

        let slider_width = (ui.available_width() - 55.0).max(54.0);
        let response = scrub_control(ui, value, slider_width, !*drag_cancelled, label)
            .on_hover_text("左右へdragしてPreview · Escで取消");
        if response.drag_started() {
            *drag_original = Some(before);
            *drag_cancelled = false;
        }
        if response.drag_stopped() {
            if *drag_cancelled {
                *drag_cancelled = false;
                *drag_original = None;
            } else if let Some(original) = drag_original.take() {
                action = Some(InspectorAction::Commit {
                    label: format!("{label} {:.0}%", *value * 100.0),
                    before: original,
                });
            }
        }
        ui.label(
            RichText::new(if *automated { "AUTO ON" } else { "AUTO OFF" })
                .monospace()
                .size(7.0)
                .color(theme::TEXT_MUTED),
        );
    });
    action
}

fn scrub_control(
    ui: &mut egui::Ui,
    value: &mut f32,
    width: f32,
    allow_drag: bool,
    label: &str,
) -> egui::Response {
    components::scrub_control(ui, value, width, allow_drag, label)
}

fn blend_row(ui: &mut egui::Ui, blend: &mut BlendMode) -> bool {
    let before = *blend;
    property_row(ui, |ui| {
        property_label(ui, "Blend");
        egui::ComboBox::from_id_salt("inspector-blend-mode")
            .selected_text(
                RichText::new(blend.label())
                    .monospace()
                    .size(9.0)
                    .color(theme::TEXT),
            )
            .width((ui.available_width() - 9.0).max(80.0))
            .show_ui(ui, |ui| {
                for candidate in [BlendMode::Screen, BlendMode::Add, BlendMode::Overlay] {
                    ui.selectable_value(blend, candidate, candidate.label());
                }
            });
    });
    *blend != before
}

fn property_row(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    components::property_row(ui, contents);
}

fn property_label(ui: &mut egui::Ui, label: &str) {
    components::property_label(ui, label);
}

fn value_box(ui: &mut egui::Ui, value: &str) {
    components::value_box(ui, value);
}

fn tag_label(ui: &mut egui::Ui, text: &str, color: Color32) {
    components::tag(ui, text, color);
}

fn notice(ui: &mut egui::Ui, title: &str, body: &str, color: Color32) {
    let frame = egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(0.0);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        frame.show(ui, |ui| {
            ui.set_width((ui.available_width() - 16.0).max(100.0));
            ui.label(RichText::new(title).strong().size(9.0).color(color));
            ui.label(RichText::new(body).size(9.0).color(theme::TEXT_SECONDARY));
        });
    });
    ui.add_space(8.0);
    horizontal_rule(ui);
}

fn developer_info(ui: &mut egui::Ui, state: &mut InspectorState, fixture: EffectFixture) {
    horizontal_rule(ui);
    let response = ui
        .allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 28.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.add_space(9.0);
                let arrow = if state.developer_info_open { "v" } else { ">" };
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{arrow}  Developer info"))
                            .monospace()
                            .size(8.0)
                            .color(if state.developer_info_open {
                                theme::TEXT_SECONDARY
                            } else {
                                theme::TEXT_MUTED
                            }),
                    )
                    .sense(Sense::click()),
                )
            },
        )
        .inner;
    if response.clicked() {
        state.developer_info_open = !state.developer_info_open;
    }
    if state.developer_info_open {
        value_row(ui, "Package", fixture.package, None);
        value_row(ui, "Identity", fixture.identity, None);
    }
}

fn horizontal_rule(ui: &egui::Ui) {
    components::horizontal_rule(ui);
}

#[derive(Debug, Clone, Copy)]
struct EffectFixture {
    name: &'static str,
    subtitle: &'static str,
    glyph: &'static str,
    section_state: &'static str,
    package: &'static str,
    identity: &'static str,
    accent: Color32,
}

fn effect_fixture(effect: InspectorEffect) -> EffectFixture {
    match effect {
        InspectorEffect::EchoBloom => EffectFixture {
            name: "Echo Bloom",
            subtitle: "Pulse rings · Effect",
            glyph: "O",
            section_state: "ON OBJECT",
            package: "Vism (.vism)",
            identity: "demo.echo-bloom",
            accent: theme::ACCENT,
        },
        InspectorEffect::TypePulse => EffectFixture {
            name: "Type Pulse",
            subtitle: "Typography · Motion Kit",
            glyph: "T",
            section_state: "INSTALLED",
            package: "Motion Kit α",
            identity: "motion-kit.type-pulse",
            accent: theme::SHAPE,
        },
        InspectorEffect::FoldField => EffectFixture {
            name: "Fold Field",
            subtitle: "Effect plugin · local file",
            glyph: "F",
            section_state: "UNAVAILABLE",
            package: "Vism (.vism)",
            identity: "demo.fold-field",
            accent: theme::WARNING,
        },
    }
}
