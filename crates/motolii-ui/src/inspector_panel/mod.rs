//! Inspector native pane(egui)。
//!
//! UX の正本は `docs/mocks-ui/public/inspector-library.html`(+ `.css`)。
//! 構造(header / mode tabs / selection summary / property table / footer)と
//! 色・寸法はそこから写し、出典は `theme.rs` が `// inspector-library.css:<line>` で持つ。
//!
//! データは `read_model.rs` の `InspectorReadModel` **だけ**を食う。
//! read-model の意味の正本は guard-tests
//! (`docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`)で、
//! fixture(reference-document)からでも live Document の snapshot からでも同じに描ける。
//! **selection → ここ の live 配線は後続レーン**(このレーンは型と描画まで)。
//!
//! 旧 `inspector_blitz`(HTML/CSS + Blitz テクスチャ)は dump 器・oracle 源として残る。
//! 意味を持つ行だけを描く: read-model に無い行(Rotation / Scale / Fill 等の
//! モック行)はデータを発明してまで出さない。

mod read_model;
mod theme;

use std::collections::BTreeSet;
use std::path::Path;

pub use read_model::{
    project_inspector_read_model, InspectorEffectDefinition, InspectorItemKind, InspectorParam,
    InspectorPosition, InspectorReadModel, InspectorReadModelError, InspectorTarget,
    INSPECTOR_READ_MODEL_REVISION,
};

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use theme::mix;

/// decoder P1 と同じ fixture 選択(reference-document の "Reference group")。
pub const FIXTURE_TARGET_LAYER: u64 = 5;

/// Inspector の mode tab(inspector-library.html:19 `Effect` / `Custom`)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Effect,
    Custom,
}

/// Inspector native pane 1面。
pub struct InspectorPanel {
    model: Option<InspectorReadModel>,
    /// footer に出す状態文(inspector-library.html:177 の `#preview-status` の席)。
    status: String,
    mode: Mode,
    /// 折り畳まれた section(0 = TRANSFORM、1.. = effect)。局所 UI 状態。
    collapsed: BTreeSet<usize>,
    /// M / S の局所視覚状態(書き込み routeは後続。html:22 の aria-pressed に対応)。
    muted: bool,
    solo: bool,
    /// FX の ON/OFF 局所視覚状態(definition_id の集合が OFF)。
    disabled_effects: BTreeSet<u64>,
}

impl InspectorPanel {
    pub fn from_read_model(model: InspectorReadModel) -> Self {
        Self {
            model: Some(model),
            status: "read-model preview · selection wiring pending".to_owned(),
            mode: Mode::Effect,
            collapsed: BTreeSet::new(),
            muted: false,
            solo: false,
            disabled_effects: BTreeSet::new(),
        }
    }

    /// read-model を作れない時の空面。理由は footer に出す(黙って空にしない)。
    pub fn placeholder(status: impl Into<String>) -> Self {
        Self {
            model: None,
            status: status.into(),
            mode: Mode::Effect,
            collapsed: BTreeSet::new(),
            muted: false,
            solo: false,
            disabled_effects: BTreeSet::new(),
        }
    }

    /// fixture(または任意の Document file)から read-model を組む。
    /// 失敗しても panic せず placeholder に落とす(毎フレーム経路から呼ばれるため)。
    pub fn from_document_path(path: &Path, target_layer: u64) -> Self {
        let document = match motolii_doc::load_document(path) {
            Ok(document) => document,
            Err(error) => {
                return Self::placeholder(format!("{} を読めない: {error}", path.display()))
            }
        };
        let catalog = match motolii_plugins_firstparty::first_party_catalog() {
            Ok(catalog) => catalog,
            Err(error) => return Self::placeholder(format!("plugin catalog を作れない: {error}")),
        };
        match project_inspector_read_model(&document, &catalog, target_layer) {
            Ok(model) => Self::from_read_model(model),
            Err(error) => Self::placeholder(format!("read-model を作れない: {error}")),
        }
    }

    pub fn read_model(&self) -> Option<&InspectorReadModel> {
        self.model.as_ref()
    }

    /// pane いっぱいに描く。値も構造も read-model と theme 以外から持ち込まない。
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter().with_clip_rect(rect);
        // inspector-library.css:27 `.inspectorShell { background: var(--mock-role-surface-panel) }`
        painter.rect_filled(rect, 0.0, theme::SURFACE_PANEL);

        let mut cursor = rect.top();
        cursor = self.draw_header(ui, &painter, rect, cursor);
        cursor = self.draw_mode_tabs(ui, &painter, rect, cursor);

        let footer_top = rect.bottom() - theme::FOOTER_H;
        match self.mode {
            Mode::Effect => {
                cursor = self.draw_summary(ui, &painter, rect, cursor);
                cursor = draw_column_header(&painter, rect, cursor);
                let body = Rect::from_min_max(
                    Pos2::new(rect.left(), cursor),
                    Pos2::new(rect.right(), footer_top),
                );
                self.draw_table(ui, body);
            }
            Mode::Custom => {
                // Custom(extensions)は後続レーン。空面に理由だけ出す(無反応にしない)。
                painter.text(
                    Pos2::new(rect.left() + 12.0, cursor + 18.0),
                    Align2::LEFT_CENTER,
                    "Extensions are a later lane",
                    FontId::proportional(theme::FS_SUMMARY_SPAN),
                    theme::TEXT_MUTED,
                );
            }
        }
        self.draw_footer(&painter, rect, footer_top);
        // 借用の都合で使わない変数を残さない。
        let _ = cursor;
    }

    /// inspector-library.css:29-47 `.panelHeader`。
    fn draw_header(&self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, top: f32) -> f32 {
        let header = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::HEADER_H),
        );
        // css:36 border-bottom 1px border-default
        painter.hline(
            header.x_range(),
            header.bottom(),
            Stroke::new(1.0, theme::BORDER_DEFAULT),
        );
        // css:39-44 accent bar(way-inspector)
        let accent = Rect::from_min_size(
            Pos2::new(
                header.left() + theme::HEADER_PAD_X,
                header.center().y - theme::HEADER_ACCENT_H / 2.0,
            ),
            Vec2::new(theme::HEADER_ACCENT_W, theme::HEADER_ACCENT_H),
        );
        painter.rect_filled(accent, 0.0, theme::WAY_INSPECTOR);
        painter.text(
            Pos2::new(accent.right() + 7.0, header.center().y),
            Align2::LEFT_CENTER,
            "Inspector",
            FontId::proportional(theme::FS_TITLE),
            theme::TEXT_PRIMARY,
        );
        // css:47 右端の詳細(html:18 `Rectangle · 3D` の席に target 要約)。
        if let Some(model) = &self.model {
            let detail = format!(
                "{} · {}",
                model.target.layer_name,
                model.target.item_kind.label()
            );
            painter.text(
                Pos2::new(header.right() - theme::HEADER_PAD_X, header.center().y),
                Align2::RIGHT_CENTER,
                detail,
                FontId::proportional(theme::FS_HEADER_SPAN),
                theme::TEXT_MUTED,
            );
        }
        let _ = ui;
        header.bottom()
    }

    /// inspector-library.css:49-71 `.modeTabs`。
    fn draw_mode_tabs(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
    ) -> f32 {
        let tabs = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::TABS_H),
        );
        painter.hline(
            tabs.x_range(),
            tabs.bottom(),
            Stroke::new(1.0, theme::BORDER_DEFAULT),
        );
        let half = tabs.width() / 2.0;
        for (index, (label, mode)) in [("Effect", Mode::Effect), ("Custom", Mode::Custom)]
            .into_iter()
            .enumerate()
        {
            let tab = Rect::from_min_size(
                Pos2::new(tabs.left() + half * index as f32, tabs.top()),
                Vec2::new(half, theme::TABS_H),
            );
            let response = ui.interact(
                tab,
                ui.id().with(("inspector-mode", index)),
                Sense::click(),
            );
            let selected = self.mode == mode;
            if selected {
                // css:67-70 selected: bg surface-app + 下線2px way-inspector
                painter.rect_filled(tab, 0.0, theme::SURFACE_APP);
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(tab.left(), tab.bottom() - 2.0),
                        Pos2::new(tab.right(), tab.bottom()),
                    ),
                    0.0,
                    theme::WAY_INSPECTOR,
                );
            }
            painter.text(
                tab.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(theme::FS_TAB),
                if selected {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_MUTED
                },
            );
            if response.clicked() {
                self.mode = mode;
            }
        }
        tabs.bottom()
    }

    /// inspector-library.css:76-106 `.selectionSummary`。
    fn draw_summary(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
    ) -> f32 {
        let summary = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::SUMMARY_H),
        );
        // css:83 bg surface-raised / css:82 border-bottom
        painter.rect_filled(summary, 0.0, theme::SURFACE_RAISED);
        painter.hline(
            summary.x_range(),
            summary.bottom(),
            Stroke::new(1.0, theme::BORDER_DEFAULT),
        );
        let Some(model) = &self.model else {
            painter.text(
                Pos2::new(summary.left() + theme::SUMMARY_PAD_X, summary.center().y),
                Align2::LEFT_CENTER,
                "No selection read-model",
                FontId::proportional(theme::FS_SUMMARY_SPAN),
                theme::TEXT_MUTED,
            );
            return summary.bottom();
        };
        // css:86-93 shapeGlyph(border 2px role-shape)。muted なら css:104 opacity .42。
        let glyph = Rect::from_min_size(
            Pos2::new(
                summary.left() + theme::SUMMARY_PAD_X,
                summary.center().y - theme::GLYPH_H / 2.0,
            ),
            Vec2::new(theme::GLYPH_W, theme::GLYPH_H),
        );
        let glyph_color = if self.muted {
            theme::ROLE_SHAPE.gamma_multiply(0.42)
        } else {
            theme::ROLE_SHAPE
        };
        painter.rect_stroke(
            glyph,
            0.0,
            Stroke::new(2.0, glyph_color),
            egui::StrokeKind::Inside,
        );
        let text_color = if self.muted {
            theme::TEXT_MUTED
        } else {
            theme::TEXT_PRIMARY
        };
        let name_x = glyph.right() + 8.0;
        painter.text(
            Pos2::new(name_x, summary.center().y - 8.0),
            Align2::LEFT_CENTER,
            &model.target.layer_name,
            FontId::proportional(theme::FS_SUMMARY),
            text_color,
        );
        let meta = match model.target.child_count {
            Some(children) => format!(
                "{} · {} children · {} shared FX",
                model.target.item_kind.label(),
                children,
                model.effect_definitions.len()
            ),
            None => format!(
                "{} · {} shared FX",
                model.target.item_kind.label(),
                model.effect_definitions.len()
            ),
        };
        painter.text(
            Pos2::new(name_x, summary.center().y + 8.0),
            Align2::LEFT_CENTER,
            meta,
            FontId::proportional(theme::FS_SUMMARY_SPAN),
            theme::TEXT_MUTED,
        );
        // css:98-103 M / S(局所視覚状態のみ。Document へは書かない)。
        let mut right = summary.right() - theme::SUMMARY_PAD_X;
        for (index, (label, pressed)) in [("S", self.solo), ("M", self.muted)].into_iter().enumerate()
        {
            let button = Rect::from_min_size(
                Pos2::new(
                    right - theme::LAYER_STATE_W,
                    summary.center().y - theme::LAYER_STATE_H / 2.0,
                ),
                Vec2::new(theme::LAYER_STATE_W, theme::LAYER_STATE_H),
            );
            right = button.left() - 3.0;
            let response = ui.interact(
                button,
                ui.id().with(("layer-state", index)),
                Sense::click(),
            );
            // css:99 既定 / css:102-103 pressed(mute=text-muted 18%、solo=action-active 18%)
            let accent = if label == "S" {
                theme::ACTION_ACTIVE
            } else {
                theme::TEXT_MUTED
            };
            let (border, fill, fg) = if pressed {
                (accent, mix(accent, 18.0, theme::SURFACE_APP), accent)
            } else if response.hovered() {
                // css:100-101 hover: border-strong / surface-panel / text-primary
                (theme::BORDER_STRONG, theme::SURFACE_PANEL, theme::TEXT_PRIMARY)
            } else {
                (theme::BORDER_DEFAULT, theme::SURFACE_APP, theme::TEXT_MUTED)
            };
            painter.rect_filled(button, 0.0, fill);
            painter.rect_stroke(button, 0.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);
            painter.text(
                button.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::monospace(theme::FS_TAB),
                fg,
            );
            if response.clicked() {
                if label == "S" {
                    self.solo = !self.solo;
                } else {
                    self.muted = !self.muted;
                }
            }
        }
        summary.bottom()
    }

    /// 本体表(TRANSFORM + FX stack)。縦に溢れたら scroll(css:108 `.tableScroller`)。
    fn draw_table(&mut self, ui: &mut egui::Ui, body: Rect) {
        let Some(model) = self.model.clone() else {
            return;
        };
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
        egui::ScrollArea::vertical()
            .id_salt("inspector-table")
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                self.draw_transform_section(ui, &model);
                draw_fx_toolbar(ui, &model);
                for (index, definition) in model.effect_definitions.iter().enumerate() {
                    self.draw_effect_section(ui, index, definition);
                }
            });
    }

    /// TRANSFORM section(html:27-50 の構造、read-model が持つ Position 行だけ)。
    fn draw_transform_section(&mut self, ui: &mut egui::Ui, model: &InspectorReadModel) {
        let keyed = matches!(model.position, InspectorPosition::Animated);
        let subtitle = if keyed { "1 · 1 keyed" } else { "1 · 0 keyed" };
        let collapsed = self.draw_section_header(ui, 0, "TRANSFORM", subtitle, theme::WAY_INSPECTOR);
        if collapsed {
            return;
        }
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), theme::ROW_H),
            Sense::hover(),
        );
        let painter = ui.painter().with_clip_rect(rect);
        // html:29 Position 行は `--property-color: var(--mock-role-data)`。
        draw_row_chrome(&painter, rect, theme::ROLE_DATA, false);
        draw_kind_icon(&painter, rect, theme::ROLE_DATA, IconKind::Vector);
        draw_row_name(&painter, rect, "Position", theme::FS_ROW_NAME, 0.0);
        let cols = value_columns(rect);
        match model.position {
            InspectorPosition::Const { x, y } => {
                // html:31-33 X/Y/Z cell。read-model は 2D なので Z は空欄(発明しない)。
                draw_value_cell(&painter, cols[0], Some("X"), &format_value(x, 3));
                draw_value_cell(&painter, cols[1], Some("Y"), &format_value(y, 3));
                draw_empty_component(&painter, cols[2]);
                draw_key_cell(&painter, cols[3], KeyVisual::Unkeyed);
            }
            InspectorPosition::Animated => {
                // N6: 値を開かない。`animated` の要約だけを出す。
                let span = Rect::from_min_max(cols[0].min, cols[2].max);
                draw_value_span(&painter, span, "animated");
                draw_key_cell(&painter, cols[3], KeyVisual::Animated);
            }
        }
    }

    /// FX 1 section(css:199-223 `.effectSection`)。
    fn draw_effect_section(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        definition: &InspectorEffectDefinition,
    ) {
        let effect_color = theme::WAY_PLUGINS;
        let disabled = self.disabled_effects.contains(&definition.definition_id);
        let collapsed = self.draw_effect_header(ui, index, definition, effect_color, disabled);
        if collapsed {
            return;
        }
        for param in &definition.params {
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), theme::ROW_H),
                Sense::hover(),
            );
            let painter = ui.painter().with_clip_rect(rect);
            let band = param_band_color(param);
            draw_row_chrome(&painter, rect, band, disabled);
            draw_kind_icon(&painter, rect, band, icon_kind_for(param));
            // css:278 effect 行の propertyName は padding-left 19px(帯ぶんの段差)。
            draw_row_name(&painter, rect, &param.id, theme::FS_FX_ROW_NAME, 8.0);
            let cols = value_columns(rect);
            match &param.default {
                motolii_plugin::Value::F64(value) => {
                    // css:415 scalar cell は3列ぶち抜き。
                    let span = Rect::from_min_max(cols[0].min, cols[2].max);
                    draw_value_span(&painter, span, &format_value(*value, 3));
                }
                motolii_plugin::Value::Vec2(v) => {
                    draw_value_cell(&painter, cols[0], Some("X"), &format_value(v[0], 3));
                    draw_value_cell(&painter, cols[1], Some("Y"), &format_value(v[1], 3));
                    draw_empty_component(&painter, cols[2]);
                }
                motolii_plugin::Value::Vec3(v) => {
                    draw_value_cell(&painter, cols[0], Some("X"), &format_value(v[0], 3));
                    draw_value_cell(&painter, cols[1], Some("Y"), &format_value(v[1], 3));
                    draw_value_cell(&painter, cols[2], Some("Z"), &format_value(v[2], 3));
                }
                motolii_plugin::Value::Color(rgba) => {
                    let span = Rect::from_min_max(cols[0].min, cols[2].max);
                    draw_color_cell(&painter, span, *rgba);
                }
                motolii_plugin::Value::AssetRef(id) => {
                    let span = Rect::from_min_max(cols[0].min, cols[2].max);
                    draw_value_span(&painter, span, &format!("asset {id}"));
                }
            }
            // params は plugin 契約の既定値で、key を持たない(css:458 keyPlaceholder)。
            draw_key_cell(&painter, cols[3], KeyVisual::Placeholder);
        }
    }

    /// section 見出し。クリックで折り畳み(css:164-185 `.sectionToggle` / `.collapsed`)。
    /// 返り値: 折り畳まれているか。
    fn draw_section_header(
        &mut self,
        ui: &mut egui::Ui,
        section: usize,
        title: &str,
        subtitle: &str,
        accent: Color32,
    ) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), theme::SECTION_H),
            Sense::click(),
        );
        if response.clicked() {
            if !self.collapsed.remove(&section) {
                self.collapsed.insert(section);
            }
        }
        let collapsed = self.collapsed.contains(&section);
        let painter = ui.painter().with_clip_rect(rect);
        // css:149 bg surface-app / css:148 border-bottom(68% mix)
        let fill = if response.hovered() {
            // css:182 hover: way-inspector 10% mix
            mix(theme::WAY_INSPECTOR, 10.0, theme::SURFACE_APP)
        } else {
            theme::SURFACE_APP
        };
        painter.rect_filled(rect, 0.0, fill);
        painter.hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 68.0, theme::SURFACE_APP)),
        );
        // css:181 開閉記号は property/effect 色。
        painter.text(
            Pos2::new(rect.left() + 10.0, rect.center().y),
            Align2::LEFT_CENTER,
            if collapsed { "›" } else { "⌄" },
            FontId::proportional(11.0),
            accent,
        );
        painter.text(
            Pos2::new(rect.left() + 22.0, rect.center().y),
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(theme::FS_SECTION),
            theme::TEXT_SECONDARY,
        );
        // css:156-162 右端の件数要約。
        painter.text(
            Pos2::new(rect.right() - 10.0, rect.center().y),
            Align2::RIGHT_CENTER,
            subtitle,
            FontId::proportional(theme::FS_SECTION),
            theme::TEXT_MUTED,
        );
        collapsed
    }

    /// effect 見出し(css:199 `.effectHeader` + badge + source + ON)。
    fn draw_effect_header(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        definition: &InspectorEffectDefinition,
        effect_color: Color32,
        disabled: bool,
    ) -> bool {
        let section = index + 1;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), theme::SECTION_H),
            Sense::click(),
        );
        let painter = ui.painter().with_clip_rect(rect);
        // css:199 bg = mix(effect 10%, surface-app)、左 inset 3px。
        painter.rect_filled(rect, 0.0, mix(effect_color, 10.0, theme::SURFACE_APP));
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            0.0,
            effect_color,
        );
        painter.hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 68.0, theme::SURFACE_APP)),
        );
        // css:207 FX badge。disabled なら css:223 で沈む。
        let badge_color = if disabled {
            theme::TEXT_MUTED
        } else {
            effect_color
        };
        let badge = Rect::from_min_size(
            Pos2::new(rect.left() + 8.0, rect.center().y - theme::BADGE_H / 2.0),
            Vec2::new(theme::BADGE_W, theme::BADGE_H),
        );
        painter.rect_filled(badge, 0.0, mix(badge_color, 18.0, theme::SURFACE_APP));
        painter.rect_stroke(
            badge,
            0.0,
            Stroke::new(1.0, badge_color),
            egui::StrokeKind::Inside,
        );
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            "FX",
            FontId::monospace(theme::FS_BADGE),
            badge_color,
        );
        let collapsed = self.collapsed.contains(&section);
        painter.text(
            Pos2::new(badge.right() + 6.0, rect.center().y),
            Align2::LEFT_CENTER,
            if collapsed { "›" } else { "⌄" },
            FontId::proportional(11.0),
            effect_color,
        );
        // css:214 effectTitle(大文字)。名前の正本は plugin_id(改名 route は後続)。
        let title = definition.plugin_id.to_uppercase();
        let title_pos = Pos2::new(badge.right() + 16.0, rect.center().y);
        painter.text(
            title_pos,
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(theme::FS_SECTION),
            theme::TEXT_SECONDARY,
        );
        // css:217 ON/OFF(局所視覚状態)。
        let on_size = Vec2::new(25.0, 15.0);
        let on_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 7.0 - on_size.x, rect.center().y - on_size.y / 2.0),
            on_size,
        );
        let on_response = ui.interact(
            on_rect,
            ui.id().with(("effect-enable", definition.definition_id)),
            Sense::click(),
        );
        if on_response.clicked() {
            if !self.disabled_effects.remove(&definition.definition_id) {
                self.disabled_effects.insert(definition.definition_id);
            }
        }
        let enabled = !self.disabled_effects.contains(&definition.definition_id);
        if enabled {
            painter.rect_filled(on_rect, 0.0, mix(effect_color, 16.0, theme::SURFACE_APP));
            painter.rect_stroke(
                on_rect,
                0.0,
                Stroke::new(1.0, mix(effect_color, 72.0, theme::SURFACE_APP)),
                egui::StrokeKind::Inside,
            );
        } else {
            // css:220 aria-pressed=false: border-default / transparent / text-muted
            painter.rect_stroke(
                on_rect,
                0.0,
                Stroke::new(1.0, theme::BORDER_DEFAULT),
                egui::StrokeKind::Inside,
            );
        }
        painter.text(
            on_rect.center(),
            Align2::CENTER_CENTER,
            if enabled { "ON" } else { "OFF" },
            FontId::monospace(theme::FS_BADGE),
            if enabled { effect_color } else { theme::TEXT_MUTED },
        );
        // css:215 effectSource(`plugin · N params`)。
        painter.text(
            Pos2::new(on_rect.left() - 7.0, rect.center().y),
            Align2::RIGHT_CENTER,
            format!("{} params", definition.params.len()),
            FontId::proportional(theme::FS_FX_SOURCE),
            theme::TEXT_MUTED,
        );
        // 見出しクリック(ON の上以外)で折り畳み。
        if response.clicked() && !on_response.clicked() {
            if !self.collapsed.remove(&section) {
                self.collapsed.insert(section);
            }
        }
        self.collapsed.contains(&section)
    }

    /// inspector-library.css:519-520 `footer` + `.statusDot`。
    fn draw_footer(&self, painter: &egui::Painter, rect: Rect, top: f32) {
        let footer = Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), top + theme::FOOTER_H),
        );
        painter.rect_filled(footer, 0.0, theme::SURFACE_APP);
        painter.hline(
            footer.x_range(),
            footer.top(),
            Stroke::new(1.0, theme::BORDER_DEFAULT),
        );
        let dot = Pos2::new(footer.left() + 9.0 + theme::STATUS_DOT / 2.0, footer.center().y);
        painter.circle_filled(dot, theme::STATUS_DOT / 2.0, theme::WAY_INSPECTOR);
        painter.text(
            Pos2::new(dot.x + theme::STATUS_DOT, footer.center().y),
            Align2::LEFT_CENTER,
            &self.status,
            FontId::proportional(theme::FS_FOOTER),
            theme::TEXT_MUTED,
        );
    }
}

/// css:115-138 `.columnHeader`(Property / X / Y / Z / ◇)。
fn draw_column_header(painter: &egui::Painter, rect: Rect, top: f32) -> f32 {
    let header = Rect::from_min_size(
        Pos2::new(rect.left(), top),
        Vec2::new(rect.width(), theme::COLUMN_HEADER_H),
    );
    painter.rect_filled(header, 0.0, theme::SURFACE_APP);
    painter.hline(
        header.x_range(),
        header.bottom(),
        Stroke::new(1.0, theme::BORDER_DEFAULT),
    );
    painter.text(
        Pos2::new(header.left() + theme::NAME_PAD_LEFT, header.center().y),
        Align2::LEFT_CENTER,
        "PROPERTY",
        FontId::proportional(theme::FS_COLUMN),
        theme::TEXT_MUTED,
    );
    let cols = value_columns(header);
    for (cell, label) in cols.iter().take(3).zip(["X", "Y", "Z"]) {
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(theme::FS_COLUMN),
            theme::TEXT_MUTED,
        );
    }
    // css:138 最終列(key)は action-active。
    painter.text(
        cols[3].center(),
        Align2::CENTER_CENTER,
        "◇",
        FontId::monospace(theme::FS_COLUMN),
        theme::ACTION_ACTIVE,
    );
    header.bottom()
}

/// css:118 の grid 列(minmax(132,1fr) + 64 x3 + 26)を rect へ写す。
/// 返り値は [X, Y, Z, key] の4矩形。
fn value_columns(row: Rect) -> [Rect; 4] {
    let key_left = row.right() - theme::KEY_COL_W;
    let values_left = (key_left - theme::VALUE_COL_W * 3.0).max(row.left() + theme::NAME_COL_MIN);
    let value_w = (key_left - values_left) / 3.0;
    let cell = |index: f32| {
        Rect::from_min_max(
            Pos2::new(values_left + value_w * index, row.top()),
            Pos2::new(values_left + value_w * (index + 1.0), row.bottom()),
        )
    };
    [
        cell(0.0),
        cell(1.0),
        cell(2.0),
        Rect::from_min_max(Pos2::new(key_left, row.top()), row.max),
    ]
}

/// 行の下地: 背景 / 左の色帯(css:291) / 下線(css:286)。
fn draw_row_chrome(painter: &egui::Painter, rect: Rect, band: Color32, disabled: bool) {
    painter.rect_filled(rect, 0.0, theme::SURFACE_PANEL);
    let band = if disabled {
        // css:221-222 effectDisabled は行が opacity .48 で沈む。
        band.gamma_multiply(0.48)
    } else {
        band
    };
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(theme::ROW_BAR_W, rect.height())),
        0.0,
        band,
    );
    painter.hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 64.0, theme::SURFACE_PANEL)),
    );
}

enum IconKind {
    Scalar,
    Vector,
    Integer,
    Color([f64; 4]),
}

/// css:313-343 `.parameterKind`(型の記号)。
fn draw_kind_icon(painter: &egui::Painter, row: Rect, color: Color32, kind: IconKind) {
    let icon = Rect::from_min_size(
        Pos2::new(
            row.left() + theme::NAME_PAD_LEFT,
            row.center().y - theme::KIND_ICON / 2.0,
        ),
        Vec2::new(theme::KIND_ICON, theme::KIND_ICON),
    );
    // css:319 border = mix(property 66%, #242424)
    painter.rect_stroke(
        icon,
        0.0,
        Stroke::new(1.0, mix(color, 66.0, Color32::from_rgb(0x24, 0x24, 0x24))),
        egui::StrokeKind::Inside,
    );
    match kind {
        // css:338 scalar: 5px の丸。
        IconKind::Scalar => {
            painter.circle_filled(icon.center(), 2.5, color);
        }
        // css:328-336 vector: 3px 角の点を 2行x3列。
        IconKind::Vector => {
            for (ix, iy) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
                let dot = Rect::from_min_size(
                    Pos2::new(
                        icon.left() + 3.0 + ix as f32 * 3.0,
                        icon.top() + 3.0 + iy as f32 * 6.0,
                    ),
                    Vec2::splat(3.0),
                );
                painter.rect_filled(dot, 0.0, color);
            }
        }
        // css:340 integer: `#`。
        IconKind::Integer => {
            painter.text(
                icon.center(),
                Align2::CENTER_CENTER,
                "#",
                FontId::monospace(10.0),
                color,
            );
        }
        // css:339 color: 値そのもので塗る。
        IconKind::Color(rgba) => {
            painter.rect_filled(icon.shrink(1.0), 0.0, color32_from_rgba(rgba));
        }
    }
}

fn draw_row_name(painter: &egui::Painter, row: Rect, name: &str, size: f32, extra_indent: f32) {
    painter.text(
        Pos2::new(
            row.left() + theme::NAME_PAD_LEFT + theme::KIND_ICON + 7.0 + extra_indent,
            row.center().y,
        ),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(size),
        theme::TEXT_PRIMARY,
    );
}

/// css:369-414 `.valueCell`(X/Y/Z 1成分)。
fn draw_value_cell(painter: &egui::Painter, cell: Rect, axis: Option<&str>, value: &str) {
    // css:375 bg = mix(surface-app 74%, panel)
    painter.rect_filled(cell, 0.0, mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL));
    painter.vline(
        cell.right(),
        cell.y_range(),
        Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 58.0, theme::SURFACE_PANEL)),
    );
    if let Some(axis) = axis {
        // css:410 軸の字は左端 5px、property 色…だが read-model 行の帯色は呼び手が持つので
        // ここでは muted に落とす(帯色は左帯と icon が既に示している)。
        painter.text(
            Pos2::new(cell.left() + 5.0, cell.center().y),
            Align2::LEFT_CENTER,
            axis,
            FontId::monospace(7.0),
            theme::TEXT_MUTED,
        );
    }
    // css:394-407 右寄せ mono。
    painter.text(
        Pos2::new(cell.right() - 6.0, cell.center().y),
        Align2::RIGHT_CENTER,
        value,
        FontId::monospace(theme::FS_VALUE),
        theme::TEXT_PRIMARY,
    );
}

/// css:415 scalar は3列ぶち抜き(値は右寄せ)。
fn draw_value_span(painter: &egui::Painter, span: Rect, value: &str) {
    painter.rect_filled(span, 0.0, mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL));
    painter.vline(
        span.right(),
        span.y_range(),
        Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 58.0, theme::SURFACE_PANEL)),
    );
    painter.text(
        Pos2::new(span.right() - 6.0, span.center().y),
        Align2::RIGHT_CENTER,
        value,
        FontId::monospace(theme::FS_VALUE),
        theme::TEXT_PRIMARY,
    );
}

/// css:486-498 `.fillValue` 相当(swatch + 16進)。
fn draw_color_cell(painter: &egui::Painter, span: Rect, rgba: [f64; 4]) {
    painter.rect_filled(span, 0.0, mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL));
    let swatch = Rect::from_min_size(
        Pos2::new(span.left() + 6.0, span.center().y - 9.5),
        Vec2::new(26.0, 19.0),
    );
    painter.rect_filled(swatch, 0.0, color32_from_rgba(rgba));
    painter.rect_stroke(
        swatch,
        0.0,
        Stroke::new(1.0, theme::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    let hex = {
        let [r, g, b, _] = rgba;
        format!(
            "#{:02X}{:02X}{:02X}",
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8
        )
    };
    painter.text(
        Pos2::new(swatch.right() + 6.0, span.center().y),
        Align2::LEFT_CENTER,
        hex,
        FontId::monospace(8.0),
        theme::TEXT_SECONDARY,
    );
}

/// css:89 `.emptyComponent`(Z の無い成分は `—`)。
fn draw_empty_component(painter: &egui::Painter, cell: Rect) {
    painter.rect_filled(cell, 0.0, mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL));
    painter.text(
        cell.center(),
        Align2::CENTER_CENTER,
        "—",
        FontId::proportional(theme::FS_VALUE),
        theme::TEXT_MUTED,
    );
}

enum KeyVisual {
    Unkeyed,
    Animated,
    Placeholder,
}

/// css:442-458 key 列(◇ / ◆ / —)。
fn draw_key_cell(painter: &egui::Painter, cell: Rect, visual: KeyVisual) {
    painter.vline(
        cell.left(),
        cell.y_range(),
        Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 60.0, theme::SURFACE_PANEL)),
    );
    let (glyph, color, fill) = match visual {
        KeyVisual::Unkeyed => ("◇", theme::TEXT_MUTED, None),
        // css:456 animated: action-active 12% の下地 + action-active の ◆。
        KeyVisual::Animated => (
            "◆",
            theme::ACTION_ACTIVE,
            Some(mix(theme::ACTION_ACTIVE, 12.0, theme::SURFACE_PANEL)),
        ),
        KeyVisual::Placeholder => ("—", theme::TEXT_MUTED, None),
    };
    if let Some(fill) = fill {
        painter.rect_filled(cell, 0.0, fill);
    }
    painter.text(
        cell.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::monospace(theme::FS_KEY),
        color,
    );
}

/// css:189-197 `.effectStackToolbar`(FX STACK の帯)。操作(検索/Group)は後続レーン。
fn draw_fx_toolbar(ui: &mut egui::Ui, model: &InspectorReadModel) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), theme::FX_TOOLBAR_H),
        Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    // css:189 bg = mix(surface-app 88%, #000)
    painter.rect_filled(rect, 0.0, mix(theme::SURFACE_APP, 88.0, Color32::BLACK));
    painter.hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(1.0, theme::BORDER_DEFAULT),
    );
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        "FX STACK",
        FontId::monospace(theme::FS_FX_SOURCE),
        theme::TEXT_SECONDARY,
    );
    painter.text(
        Pos2::new(rect.right() - 8.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{} shared definitions", model.effect_definitions.len()),
        FontId::proportional(theme::FS_FX_SOURCE),
        theme::TEXT_MUTED,
    );
}

/// FX param 行の帯色(css:297-302 の data-param-kind 対応)。
fn param_band_color(param: &InspectorParam) -> Color32 {
    match param.value_type {
        // css:300 integer(F64 で domain.integer のもの)
        motolii_plugin::ValueType::F64
            if param.f64_domain.is_some_and(|domain| domain.integer) =>
        {
            theme::ROLE_SHAPE
        }
        // css:297 scalar
        motolii_plugin::ValueType::F64 => theme::ROLE_DATA,
        // css:298 vector
        motolii_plugin::ValueType::Vec2 | motolii_plugin::ValueType::Vec3 => theme::ROLE_VECTOR,
        // html:54 Fill 行は値の色そのものを帯に使う(--property-color: var(--fill-color))
        motolii_plugin::ValueType::Color => match &param.default {
            motolii_plugin::Value::Color(rgba) => color32_from_rgba(*rgba),
            _ => theme::ROLE_DATA,
        },
        // AssetRef は css:283 の propertyRow 既定色(text-secondary)。
        motolii_plugin::ValueType::AssetRef => theme::TEXT_SECONDARY,
    }
}

fn icon_kind_for(param: &InspectorParam) -> IconKind {
    match param.value_type {
        motolii_plugin::ValueType::F64
            if param.f64_domain.is_some_and(|domain| domain.integer) =>
        {
            IconKind::Integer
        }
        motolii_plugin::ValueType::F64 => IconKind::Scalar,
        motolii_plugin::ValueType::Vec2 | motolii_plugin::ValueType::Vec3 => IconKind::Vector,
        motolii_plugin::ValueType::Color => match &param.default {
            motolii_plugin::Value::Color(rgba) => IconKind::Color(*rgba),
            _ => IconKind::Scalar,
        },
        motolii_plugin::ValueType::AssetRef => IconKind::Integer,
    }
}

fn color32_from_rgba(rgba: [f64; 4]) -> Color32 {
    let ch = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(ch(rgba[0]), ch(rgba[1]), ch(rgba[2]), ch(rgba[3]))
}

/// 小数を桁数固定で出し、末尾の `-0.000` を `0.000` に寄せる。
/// html:31 `data-decimals="3"` の写し(位置系は3桁)。
fn format_value(value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    if text.starts_with('-') && text.trim_start_matches(['-', '0', '.']).is_empty() {
        text.trim_start_matches('-').to_owned()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_panel_carries_the_p1_read_model() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/mocks-ui/fixtures/reference-document.json");
        let panel = InspectorPanel::from_document_path(&path, FIXTURE_TARGET_LAYER);
        let model = panel.read_model().expect("fixture read-model");
        assert_eq!(model.target.layer_name, "Reference group");
        assert_eq!(model.effect_definitions.len(), 2);
    }

    #[test]
    fn missing_document_falls_back_to_a_placeholder_with_reason() {
        let panel = InspectorPanel::from_document_path(
            std::path::Path::new("/no/such/motolii-doc.json"),
            FIXTURE_TARGET_LAYER,
        );
        assert!(panel.read_model().is_none());
        assert!(panel.status.contains("/no/such/motolii-doc.json"));
    }

    #[test]
    fn negative_zero_formats_as_zero() {
        assert_eq!(format_value(-0.0, 3), "0.000");
        assert_eq!(format_value(-0.075, 3), "-0.075");
    }
}
