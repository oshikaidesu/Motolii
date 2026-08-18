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
//!
//! **live のときは Timeline の選択がそのままここへ来る**(`seat_live`)。
//! Transform の Position / Opacity は直打ちでき、◇ で playhead へキーが打てる。
//! ただし **この面は Document を書かない**(single writer) — 出すのは
//! [`InspectorAction`] だけで、writer を持つ `TimelineEditor` が適用する。
//!
//! 旧 `inspector_blitz`(HTML/CSS + Blitz テクスチャ)は dump 器・oracle 源として残る。
//! 意味を持つ行だけを描く: read-model に無い行(Rotation / Scale / Fill 等の
//! モック行)はデータを発明してまで出さない。

mod read_model;
mod theme;

use std::collections::BTreeSet;
use std::path::Path;

pub use read_model::{
    project_inspector_live_model, project_inspector_read_model, InspectorEditParam,
    InspectorEditableRow, InspectorEffectDefinition, InspectorFlags, InspectorItemKind,
    InspectorKeyState, InspectorParam, InspectorPosition, InspectorReadModel,
    InspectorReadModelError, InspectorTarget, INSPECTOR_READ_MODEL_REVISION,
};

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use theme::mix;

/// decoder P1 と同じ fixture 選択(reference-document の "Reference group")。
pub const FIXTURE_TARGET_LAYER: u64 = 5;

/// 値セルの中に軸の字(X/Y/Z)を出す最小幅。これより狭いと字が数の桁を食う。
const AXIS_GLYPH_MIN_CELL_W: f32 = 46.0;

/// Inspector が出す編集要求。**Inspector は Document を書かない**(single writer) —
/// これを受けた呼び手が `TimelineEditor` の適用口へ渡す。
///
/// 相手の layer は「いま映している target」なので、ここには載せない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InspectorAction {
    /// 数値を掴んだ。ここから `EndEdit` までが1 gesture = 1 Undo。
    BeginEdit { param: InspectorEditParam },
    /// 成分1本の値が変わった(開いている gesture へ入る)。
    SetComponent {
        param: InspectorEditParam,
        component: usize,
        value: f64,
    },
    /// 掴みを離した。
    EndEdit,
    /// M / S を押した。**押下状態は Document が持つ**ので、ここは反転要求だけ。
    /// 呼び手は Timeline 行の M / S と同じ `toggle_item_flag` へ渡す(F-03 枝A)。
    ToggleFlag { flag: InspectorFlag },
    /// 共有 FX の ON/OFF を押した。相手は `EffectDefinition`
    /// (`enabled` は定義側に在り、評価がこの旗で effect を飛ばす)。
    SetEffectEnabled { definition_id: u64, enabled: bool },
    /// ◇ を押した。playhead へキーを打つ(既にあれば `components` の値へ更新)。
    KeyAtPlayhead {
        param: InspectorEditParam,
        /// 画面に出ている成分値。Position は `[x, y, _]`、Opacity は `[a, _, _]`。
        /// 長さは `len` が持つ(`Copy` のため固定長で運ぶ)。
        components: [f64; 3],
        len: usize,
    },
}

/// Inspector の M / S。Timeline 側の `ItemFlag` と一対一だが、**Inspector は
/// timeline の型を知らない**(投影と要求しか持たない層のまま)。写すのは呼び手。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorFlag {
    Mute,
    Solo,
}

impl InspectorAction {
    /// `KeyAtPlayhead` が運ぶ成分値。
    pub fn key_components(&self) -> &[f64] {
        match self {
            Self::KeyAtPlayhead { components, len, .. } => &components[..*len],
            _ => &[],
        }
    }
}

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
    /// 正本の footer は js の `remember()`(html:204-209)が**選択中の面で起きた事**を
    /// 書き込む席で、選択の有無そのものを言う席ではない。よって**空状態の理由は
    /// ここに来ない** — 座席が空なら空文字で、footer は帯だけになる。
    status: String,
    /// model が無いときに面の真ん中へ出す一言。**黙って空にしない**ための席で、
    /// 「No selection」「2 items selected」「読めない理由」がここに来る。
    /// 理由を出すのは**この1箇所だけ**(footer と二重に出さない)。
    empty_note: String,
    mode: Mode,
    /// 折り畳まれた section(0 = TRANSFORM、1.. = effect)。局所 UI 状態。
    collapsed: BTreeSet<usize>,
    // M / S と FX ON/OFF の**押下状態はここに無い**。正本は Document で、
    // 面は read-model(`flags` / `InspectorEffectDefinition::enabled`)を読む。
    // 局所 bool を戻すと「見た目だけ変わる」に逆戻りする(2026-08-18 外部診断 F-03)。
    /// いま掴んでいる数値(param と成分)。**1掴み = 1 gesture** の判定に使う。
    edit_hold: Option<(InspectorEditParam, usize)>,
    /// live 投影に使う plugin catalog。座席の writer と同じ `reference_catalog` を
    /// 1度だけ組んで持ち回る(毎フレーム組み直さない)。
    catalog: Option<std::sync::Arc<motolii_plugin::PluginCatalog>>,
}

impl InspectorPanel {
    pub fn from_read_model(model: InspectorReadModel) -> Self {
        let mut panel = Self::placeholder("read-model preview");
        panel.model = Some(model);
        panel.empty_note.clear();
        // 座席が埋まったので footer に報告する事ができた。
        panel.status = "read-model preview".to_owned();
        panel
    }

    /// read-model を作れない時の空面。理由は**面の本文**に出す(黙って空にしない)。
    /// footer は選択中の面の状態を言う席なので、ここでは空のままにする。
    pub fn placeholder(note: impl Into<String>) -> Self {
        Self {
            model: None,
            empty_note: note.into(),
            status: String::new(),
            mode: Mode::Effect,
            collapsed: BTreeSet::new(),
            edit_hold: None,
            catalog: None,
        }
    }

    /// model が無いときに面へ出る一言(読み)。
    pub fn empty_note(&self) -> &str {
        &self.empty_note
    }

    /// live の選択と playhead でこの面を座らせ直す。**毎フレーム呼んでよい** —
    /// 折り畳み・mode などの局所 UI 状態は残る。
    ///
    /// - 選択なし … 「No selection」の空状態(fixture へ落ちない)
    /// - 2つ以上 … 「N items selected」の要約(property は出さない)
    /// - 1つ … live 投影。失敗したら理由つきの空状態
    pub fn seat_live(
        &mut self,
        document: &motolii_doc::Document,
        selection: &[motolii_doc::LayerId],
        playhead: motolii_core::RationalTime,
    ) {
        match selection {
            [] => self.seat_empty("No selection — pick a layer in the Timeline"),
            [one] => {
                let catalog = match self.catalog.clone() {
                    Some(catalog) => catalog,
                    None => match motolii_plugin::reference::reference_catalog() {
                        Ok(catalog) => {
                            let catalog = std::sync::Arc::new(catalog);
                            self.catalog = Some(std::sync::Arc::clone(&catalog));
                            catalog
                        }
                        Err(error) => {
                            self.seat_empty(format!("plugin catalog を作れない: {error}"));
                            return;
                        }
                    },
                };
                match project_inspector_live_model(document, &catalog, one.get(), playhead) {
                    Ok(model) => {
                        self.model = Some(model);
                        self.empty_note.clear();
                        self.status = format!("layer {} · live", one.get());
                    }
                    Err(error) => self.seat_empty(format!("read-model を作れない: {error}")),
                }
            }
            many => self.seat_empty(format!(
                "{} items selected — pick one to edit",
                many.len()
            )),
        }
    }

    /// 空状態へ座らせ直す。理由は `empty_note`(本文)にだけ置き、footer は黙らせる。
    /// 両方へ書くと同じ一言が面に2度出る。
    fn seat_empty(&mut self, note: impl Into<String>) {
        self.model = None;
        self.edit_hold = None;
        self.status.clear();
        self.empty_note = note.into();
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
    ///
    /// 返すのは**編集要求**だけで、この面は Document を書かない(single writer)。
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> Vec<InspectorAction> {
        let mut actions = Vec::new();
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
                cursor = self.draw_summary(ui, &painter, rect, cursor, &mut actions);
                cursor = draw_column_header(&painter, rect, cursor);
                let body = Rect::from_min_max(
                    Pos2::new(rect.left(), cursor),
                    Pos2::new(rect.right(), footer_top),
                );
                self.draw_table(ui, body, &mut actions);
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
        actions
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
        actions: &mut Vec<InspectorAction>,
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
            // 空状態。**理由をそのまま出す**(「No selection」/「N items selected」/
            // 読めない理由)。黙って空の面にしない。
            painter.text(
                Pos2::new(summary.left() + theme::SUMMARY_PAD_X, summary.center().y),
                Align2::LEFT_CENTER,
                if self.empty_note.is_empty() {
                    "No selection"
                } else {
                    self.empty_note.as_str()
                },
                FontId::proportional(theme::FS_SUMMARY_SPAN),
                theme::TEXT_MUTED,
            );
            return summary.bottom();
        };
        // 押下状態は **Document から**(局所 bool を持たない。F-03)。
        let InspectorFlags { muted, solo } = model.flags;
        // css:86-93 shapeGlyph(border 2px role-shape)。muted なら css:104 opacity .42。
        let glyph = Rect::from_min_size(
            Pos2::new(
                summary.left() + theme::SUMMARY_PAD_X,
                summary.center().y - theme::GLYPH_H / 2.0,
            ),
            Vec2::new(theme::GLYPH_W, theme::GLYPH_H),
        );
        let glyph_color = if muted {
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
        let text_color = if muted {
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
        // css:98-103 M / S。**押下状態は Document(`model.flags`)**で、押すと
        // Timeline 行と同じ1手へ落ちる要求が出る(F-03 枝A)。
        let mut right = summary.right() - theme::SUMMARY_PAD_X;
        for (index, (label, pressed, flag)) in [
            ("S", solo, InspectorFlag::Solo),
            ("M", muted, InspectorFlag::Mute),
        ]
        .into_iter()
        .enumerate()
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
            // painter で描く面なので、名乗らせないと AccessKit の木に出ない
            // = 運転席(kittest)からも支援技術からも掴めない(Browser card と同じ理由)。
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Button, true, pressed, label)
            });
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
                actions.push(InspectorAction::ToggleFlag { flag });
            }
        }
        summary.bottom()
    }

    /// 本体表(TRANSFORM + FX stack)。縦に溢れたら scroll(css:108 `.tableScroller`)。
    fn draw_table(&mut self, ui: &mut egui::Ui, body: Rect, actions: &mut Vec<InspectorAction>) {
        let Some(model) = self.model.clone() else {
            return;
        };
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
        egui::ScrollArea::vertical()
            .id_salt("inspector-table")
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                self.draw_transform_section(ui, &model, actions);
                draw_fx_toolbar(ui, &model);
                for (index, definition) in model.effect_definitions.iter().enumerate() {
                    self.draw_effect_section(ui, index, definition, actions);
                }
            });
    }

    /// TRANSFORM section(html:27-50 の構造)。
    ///
    /// live 投影(`editable` が埋まっている)なら**直打ちできる行**を出し、
    /// fixture 投影ならこれまでどおり decoder の Position 要約だけを出す。
    fn draw_transform_section(
        &mut self,
        ui: &mut egui::Ui,
        model: &InspectorReadModel,
        actions: &mut Vec<InspectorAction>,
    ) {
        if model.editable.is_empty() {
            let keyed = matches!(model.position, InspectorPosition::Animated);
            let subtitle = if keyed { "1 · 1 keyed" } else { "1 · 0 keyed" };
            let collapsed =
                self.draw_section_header(ui, 0, "TRANSFORM", subtitle, theme::WAY_INSPECTOR);
            if collapsed {
                return;
            }
            self.draw_fixture_position_row(ui, model);
            return;
        }

        let keyed = model
            .editable
            .iter()
            .filter(|row| row.key_state != InspectorKeyState::Unkeyed)
            .count();
        let subtitle = format!("{} · {keyed} keyed", model.editable.len());
        let collapsed = self.draw_section_header(ui, 0, "TRANSFORM", &subtitle, theme::WAY_INSPECTOR);
        if collapsed {
            return;
        }
        for row in &model.editable {
            self.draw_editable_row(ui, row, actions);
        }
    }

    /// fixture 投影の Position 行(decoder D1 の要約そのまま。触れない)。
    fn draw_fixture_position_row(&mut self, ui: &mut egui::Ui, model: &InspectorReadModel) {
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

    /// live の1行。**数は直打ちでき、◇ でキーが打てる。**
    ///
    /// gesture の畳み: 変化があったフレームで掴んでいなければ `BeginEdit`、
    /// 掴んだまま動いているあいだ `SetComponent`、掴みが外れたら `EndEdit`。
    /// ドラッグの途中フレームは `BeginEdit` を出さないので、1ドラッグが1 Undo になる。
    fn draw_editable_row(
        &mut self,
        ui: &mut egui::Ui,
        row: &InspectorEditableRow,
        actions: &mut Vec<InspectorAction>,
    ) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), theme::ROW_H),
            Sense::hover(),
        );
        let band = match row.param {
            // css:298 vector / css:297 scalar
            InspectorEditParam::Position => theme::ROLE_VECTOR,
            InspectorEditParam::Opacity => theme::ROLE_DATA,
        };
        let icon = match row.param {
            InspectorEditParam::Position => IconKind::Vector,
            InspectorEditParam::Opacity => IconKind::Scalar,
        };
        {
            let painter = ui.painter().with_clip_rect(rect);
            draw_row_chrome(&painter, rect, band, !row.editable);
            draw_kind_icon(&painter, rect, band, icon);
            draw_row_name(&painter, rect, row.label(), theme::FS_ROW_NAME, 0.0);
        }

        let cols = value_columns(rect);
        let axes = row.param.axes();
        for (index, cell) in cols.iter().take(3).enumerate() {
            let Some(axis) = axes.get(index) else {
                // 持たない成分は `—`(html:89 emptyComponent)。値を発明しない。
                draw_empty_component(&ui.painter().with_clip_rect(*cell), *cell);
                continue;
            };
            let value = row.components.get(index).copied().unwrap_or_default();
            self.draw_value_field(ui, *cell, *axis, row, index, value, actions);
        }

        // ◇ / ◆。押すと playhead へキーが立つ(既にあれば値を更新)。
        let key_visual = match row.key_state {
            InspectorKeyState::Unkeyed => KeyVisual::Unkeyed,
            InspectorKeyState::Animated => KeyVisual::Unkeyed,
            InspectorKeyState::KeyedAtPlayhead => KeyVisual::Animated,
        };
        let key_cell = cols[3];
        let response = ui.interact(
            key_cell,
            ui.id().with(("inspector-key", row.label())),
            if row.editable {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        {
            let painter = ui.painter().with_clip_rect(key_cell);
            if response.hovered() && row.editable {
                painter.rect_filled(
                    key_cell,
                    0.0,
                    mix(theme::ACTION_ACTIVE, 18.0, theme::SURFACE_PANEL),
                );
            }
            draw_key_cell(&painter, key_cell, key_visual);
        }
        if response.clicked() && row.editable {
            let mut components = [0.0; 3];
            let len = row.components.len().min(3);
            components[..len].copy_from_slice(&row.components[..len]);
            actions.push(InspectorAction::KeyAtPlayhead {
                param: row.param,
                components,
                len,
            });
        }
    }

    /// 1成分の数値フィールド(css:369-414 `.valueCell` の席に `DragValue` を置く)。
    fn draw_value_field(
        &mut self,
        ui: &mut egui::Ui,
        cell: Rect,
        axis: Option<&'static str>,
        row: &InspectorEditableRow,
        index: usize,
        value: f64,
        actions: &mut Vec<InspectorAction>,
    ) {
        {
            let painter = ui.painter().with_clip_rect(cell);
            // css:375 bg = mix(surface-app 74%, panel)
            painter.rect_filled(cell, 0.0, mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL));
            painter.vline(
                cell.right(),
                cell.y_range(),
                Stroke::new(1.0, mix(theme::BORDER_DEFAULT, 58.0, theme::SURFACE_PANEL)),
            );
            // css:410 の軸の字。**狭い pane では出さない** — 列見出しが既に X/Y/Z を
            // 言っているので、字を残して数の桁を削るほうが失う情報が大きい。
            if let Some(axis) = axis.filter(|_| cell.width() >= AXIS_GLYPH_MIN_CELL_W) {
                painter.text(
                    Pos2::new(cell.left() + 5.0, cell.center().y),
                    Align2::LEFT_CENTER,
                    axis,
                    FontId::monospace(7.0),
                    theme::TEXT_MUTED,
                );
            }
        }
        if !row.editable {
            // 触れない行は値だけ出す(触れそうで触れないものを置かない)。
            let painter = ui.painter().with_clip_rect(cell);
            painter.text(
                Pos2::new(cell.right() - 6.0, cell.center().y),
                Align2::RIGHT_CENTER,
                format_value(value, 3),
                FontId::monospace(theme::FS_VALUE),
                theme::TEXT_MUTED,
            );
            return;
        }

        // 軸ラベルの右から右端までが入力欄。**cell からはみ出さない** —
        // 桁が切れると数が読めなくなるので、字も枠も cell に合わせて縮める。
        let axis_inset = if axis.is_some() && cell.width() >= AXIS_GLYPH_MIN_CELL_W {
            11.0
        } else {
            2.0
        };
        let field = Rect::from_min_max(
            Pos2::new(cell.left() + axis_inset, cell.top() + 2.0),
            Pos2::new(cell.right() - 2.0, cell.bottom() - 2.0),
        );
        let mut edited = value;
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
        // 値の字は css:394-407 の mono。DragValue の既定枠は cell の地に溶かして、
        // 「セルの中の数」に見せる(触れる面は hover / focus で分かる)。
        child.style_mut().override_font_id = Some(FontId::monospace(theme::FS_VALUE));
        child.style_mut().spacing.button_padding = Vec2::new(2.0, 0.0);
        child.style_mut().spacing.interact_size = Vec2::new(0.0, field.height());
        let cell_fill = mix(theme::SURFACE_APP, 74.0, theme::SURFACE_PANEL);
        let visuals = &mut child.style_mut().visuals.widgets;
        visuals.inactive.weak_bg_fill = cell_fill;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.inactive.fg_stroke = Stroke::new(1.0, theme::TEXT_PRIMARY);
        visuals.hovered.weak_bg_fill = mix(theme::ACTION_ACTIVE, 14.0, cell_fill);
        visuals.hovered.bg_stroke = Stroke::new(1.0, theme::BORDER_STRONG);
        visuals.active.weak_bg_fill = mix(theme::ACTION_ACTIVE, 22.0, cell_fill);
        let response = child.put(
            field,
            egui::DragValue::new(&mut edited)
                // 正準座標(原点中央・高さ1.0)も不透明度も 0..1 桁の世界なので刻みは細かく。
                .speed(0.005)
                .max_decimals(3),
        );

        let key = (row.param, index);
        let changed = response.changed();
        // 掴んでいる間 = ドラッグ中か、テキスト入力に focus がある間。
        let holding = response.dragged() || response.has_focus();
        if changed {
            if self.edit_hold != Some(key) {
                // 別の欄を掴み直したら前の gesture を先に閉じる。
                if self.edit_hold.is_some() {
                    actions.push(InspectorAction::EndEdit);
                }
                actions.push(InspectorAction::BeginEdit { param: row.param });
                self.edit_hold = Some(key);
            }
            actions.push(InspectorAction::SetComponent {
                param: row.param,
                component: index,
                value: edited,
            });
        }
        if self.edit_hold == Some(key) && !holding {
            actions.push(InspectorAction::EndEdit);
            self.edit_hold = None;
        }
    }

    /// FX 1 section(css:199-223 `.effectSection`)。
    fn draw_effect_section(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        definition: &InspectorEffectDefinition,
        actions: &mut Vec<InspectorAction>,
    ) {
        let effect_color = theme::WAY_PLUGINS;
        // OFF の出所は Document(`EffectDefinition.enabled`)。局所集合は持たない。
        let disabled = !definition.enabled;
        let collapsed =
            self.draw_effect_header(ui, index, definition, effect_color, disabled, actions);
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
        actions: &mut Vec<InspectorAction>,
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
        // css:217 ON/OFF。**押下状態は Document(`definition.enabled`)**で、
        // 押すと共有 recipe の `enabled` を書く要求が出る(F-03 枝A)。
        let enabled = definition.enabled;
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
        // painter で描く面なので名乗らせる(M / S と同じ理由)。
        on_response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                true,
                enabled,
                if enabled { "ON" } else { "OFF" },
            )
        });
        if on_response.clicked() {
            actions.push(InspectorAction::SetEffectEnabled {
                definition_id: definition.definition_id,
                enabled: !enabled,
            });
        }
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
        // 正本の footer は `.statusDot` + `#preview-status` が必ず対で出る(html:177)。
        // 報告する状態が無いとき(= 座席が空)は点だけ残さず、帯だけにする。
        // 空の理由は summary 行が既に言っている — ここへ写すと同じ一言が2度出る。
        if self.status.is_empty() {
            return;
        }
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
    use std::sync::Arc;

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
        // 理由は面の本文(`empty_note`)にだけ立つ。footer は同じ一言を繰り返さない。
        assert!(panel.empty_note().contains("/no/such/motolii-doc.json"));
        assert!(
            panel.status.is_empty(),
            "空状態の理由が footer にも出ている: {:?}",
            panel.status
        );
    }

    #[test]
    fn an_empty_seat_says_its_reason_once() {
        let panel = InspectorPanel::placeholder("No selection — pick a layer in the Timeline");
        assert_eq!(
            panel.empty_note(),
            "No selection — pick a layer in the Timeline"
        );
        assert!(panel.status.is_empty());
    }

    #[test]
    fn a_seated_model_reports_in_the_footer() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/mocks-ui/fixtures/reference-document.json");
        let panel = InspectorPanel::from_document_path(&path, FIXTURE_TARGET_LAYER);
        assert!(panel.read_model().is_some());
        assert!(panel.empty_note().is_empty());
        assert!(!panel.status.is_empty());
    }

    #[test]
    fn negative_zero_formats_as_zero() {
        assert_eq!(format_value(-0.0, 3), "0.000");
        assert_eq!(format_value(-0.075, 3), "-0.075");
    }

    // ---- F-03: 見えているトグルは押すと意味へ届く ----

    fn reference_document() -> motolii_doc::Document {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/mocks-ui/fixtures/reference-document.json");
        motolii_doc::load_document(&path).expect("reference document")
    }

    /// 書き換え用の writer。**catalog が投影側と違うのは fixture の都合**で、
    /// `reference-document.json` の `core.filter.opacity` は instance param を
    /// `opacity` で持つのに first-party 契約の param 名は `amount` である。
    /// first-party を渡すと writer が契約違反で立たないので、その id を持たない
    /// reference catalog(未知 plugin として通す既存経路)で立てる。
    /// **この食い違いは本レーンの主題ではない**(投影も評価も従来どおり)。
    fn writer_over_the_fixture() -> motolii_doc::DocumentWriter {
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        motolii_doc::DocumentWriter::new(reference_document(), catalog)
            .expect("writer over the reference document")
    }

    /// 面を1つ座らせる(`from_document_path` と同じ catalog = first-party)。
    /// `flags` / `enabled` は live 投影と共通の `project` が埋めるので、
    /// ここで見れば live 側の出所も同じである。
    fn seated(document: &motolii_doc::Document) -> InspectorPanel {
        let catalog =
            motolii_plugins_firstparty::first_party_catalog().expect("first party catalog");
        let model = project_inspector_read_model(document, &catalog, FIXTURE_TARGET_LAYER)
            .expect("reference read-model");
        InspectorPanel::from_read_model(model)
    }

    /// 対象 layer を mute / solo にした Document を **writer 経由で**作る。
    /// `&mut Document` は motolii-doc の中だけ(single writer。`mut_document_deny`)。
    fn document_with_flags(visible: bool, solo: bool) -> Arc<motolii_doc::Document> {
        let mut writer = writer_over_the_fixture();
        let target = motolii_doc::LayerId::from_raw(FIXTURE_TARGET_LAYER);
        for prepared in [
            writer.prepare_set_item_visible(target, visible),
            writer.prepare_set_item_solo(target, solo),
        ] {
            if let Some(command) = prepared.expect("prepared flag command") {
                let gesture = writer.begin_gesture();
                writer.apply_command(gesture, command).expect("apply");
            }
        }
        writer.snapshot()
    }

    /// **M / S の正本は Document。** 面はそれを映すだけで、局所 bool を持たない
    /// (2026-08-18 外部診断 F-03)。同じ面へ違う Document を座らせれば表示が変わる。
    #[test]
    fn mute_and_solo_come_from_the_document_not_from_the_panel() {
        let plain = document_with_flags(true, false);
        let flags = seated(&plain).read_model().expect("seated").flags;
        assert!(!flags.muted, "visible な layer が mute に見えている");
        assert!(!flags.solo);

        let muted = document_with_flags(false, true);
        let flags = seated(&muted).read_model().expect("seated").flags;
        assert!(flags.muted, "visible=false は mute として映る");
        assert!(flags.solo);
    }

    /// FX の ON/OFF も Document の `EffectDefinition.enabled` が正本。
    /// OFF の Document も **writer 経由**で作る(single writer)。
    #[test]
    fn effect_on_off_comes_from_the_document() {
        let mut writer = writer_over_the_fixture();
        let definitions: Vec<_> = writer
            .snapshot()
            .effect_definitions
            .iter()
            .map(|definition| definition.id)
            .collect();
        assert!(!definitions.is_empty(), "fixture に effect が無い");
        for definition in definitions {
            let command = writer
                .prepare_set_effect_enabled(definition, false)
                .expect("prepared")
                .expect("ON → OFF は値が変わる");
            let gesture = writer.begin_gesture();
            writer.apply_command(gesture, command).expect("apply");
        }

        let document = writer.snapshot();
        let panel = seated(&document);
        let model = panel.read_model().expect("seated");
        for definition in &model.effect_definitions {
            assert!(
                !definition.enabled,
                "Document が OFF の effect が ON に見えている: {}",
                definition.plugin_id
            );
        }
    }

    /// 名前で1つ押して、そのフレームに出た要求を集める。
    ///
    /// **名前で掴めること自体が合否の一部**。painter で描く面は `widget_info` を
    /// 出さないと AccessKit の木に載らず、運転席からも支援技術からも触れない。
    fn actions_after_clicking(
        document: &motolii_doc::Document,
        label: &str,
    ) -> Vec<InspectorAction> {
        use egui_kittest::kittest::Queryable;
        let mut harness = egui_kittest::Harness::builder()
            .with_size([320.0, 620.0])
            .build_ui_state(
                |ui, state: &mut (InspectorPanel, Vec<InspectorAction>)| {
                    state.1 = state.0.show(ui);
                },
                (seated(document), Vec::new()),
            );
        harness.run();
        // FX は複数あるので**最初の1つ**を押す(名前で掴めない時は `get_all_*` が落ちる)。
        harness
            .get_all_by_label(label)
            .next()
            .expect("at least one match")
            .click();
        harness.step();
        harness.state().1.clone()
    }

    /// M を押したら**要求が出る**。局所 bool を反転して終わらない。
    #[test]
    fn clicking_mute_asks_for_a_document_write() {
        let document = reference_document();
        assert_eq!(
            actions_after_clicking(&document, "M"),
            vec![InspectorAction::ToggleFlag {
                flag: InspectorFlag::Mute
            }]
        );
        assert_eq!(
            actions_after_clicking(&document, "S"),
            vec![InspectorAction::ToggleFlag {
                flag: InspectorFlag::Solo
            }]
        );
    }

    /// FX の ON も同じ。押した先は Document の `enabled`。
    #[test]
    fn clicking_an_effect_toggle_asks_for_a_document_write() {
        let document = reference_document();
        let definition_id = document
            .effect_definitions
            .first()
            .expect("fixture effect")
            .id
            .get();
        let actions = actions_after_clicking(&document, "ON");
        assert_eq!(
            actions,
            vec![InspectorAction::SetEffectEnabled {
                definition_id,
                enabled: false,
            }],
            "FX の ON/OFF が局所集合のままになっている"
        );
    }
}
