//! Settings パネル枠。project / session / appearance の意味は波1。iced は引かない。
//! 見た目の正本は `mod.tokens`(裁定267: Ableton の identity は palette でなく形の文法。
//! 面/字/線/選択/accent は Live 12 `.ask` 実機抽出由来)。ここに生の hex を書かない —
//! 書いた瞬間、皮の差し替えがこの1枚だけ効かなくなる。
//! 形: 枠線・角丸・影なし。欄は窪み（暗面）で示す。行 + ラベル + 窪み値欄の縦積み。
//!
//! ## comp 設定(尺・fps・解像度)は本物(2026-08-28、S8 着地)
//!
//! PROJECT 帯の3行(Frame Rate / Resolution / Duration)は `ScrubValue`
//! (`inspector_surface.rs` が登録する型、footer の3つの約束 — scrub / タイプ /
//! Esc 取消 — をそのまま借りる)。値は `motolii-store` の `Composition` の投影で、
//! 確定は `Intent::SetComposition` として host へ運ぶ(丸ごと差し替え型なので、
//! host 側が今の値を読んで1欄だけ変える — 裁定271「名指していない欄は既定へ
//! 戻さない」)。**この widget は Document を持たない** — 投影は
//! `main.rs::install_settings`、書き込みの意図は [`SettingsSurfaceAction`] で
//! 外へ出すだけ(`BrowserEditAction`/`FxStackAction` と同じ形)。
//!
//! SESSION / APPEARANCE 帯(Autosave / Undo Depth / Theme / UI Scale)は
//! まだ見た目用ダミー(出典なし) — store 側にこれらの設定を持つ場所がまだ無い。
//! Chrome* 部品は参照しない（main.rs の読み込み順で本 mod が chrome より先）。
use crate::inspector_surface::{ScrubValue, ScrubValueAction};
use makepad_widgets::*;
use motolii_store::Composition;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.SettingsSurfaceBase = #(SettingsSurface::register_widget(vm))
    mod.widgets.SettingsSurface = set_type_default() do mod.widgets.SettingsSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // 頭帯 — Live の device title bar と同型(明帯 + 暗字 = 極性反転、橙の活性点)
        settings_head := SolidView{width: Fill height: mod.tokens.size.toolbar flow: Right spacing: mod.tokens.space.s3 align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.head
            power_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: mod.tokens.accent.on}
            title := InkLabel{text: "Settings" width: Fill draw_text.color: mod.tokens.ink.on_fill draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xl}}
        }
        head_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // PROJECT の3行(Frame Rate / Resolution / Duration)は Composition の投影
        // (`SettingsSurface::set_composition`)。SESSION / APPEARANCE は波1のダミーのまま
        project_band := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
            project_head := InkLabel{text: "PROJECT" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.sm}}
        }
        frame_rate_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Frame Rate" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 1.0 max: 1000.0 suffix: " fps" prop: "fps"}
            }
        }
        frame_rate_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        resolution_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Resolution" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            // 幅は Fit — 2つの ScrubValue + 区切りの合計に合わせる(inspector の
            // vx/vy/vz と同じ「値セルを並べる」形。104 という固定値を発明しない)
            well := SolidView{width: Fit height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} spacing: mod.tokens.space.s1 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                width_value := ScrubValue{width: 52 step: 1.0 precision: 0 min: 1.0 max: 16384.0 prop: "width"}
                times := InkLabel{text: "×" width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
                height_value := ScrubValue{width: 52 step: 1.0 precision: 0 min: 1.0 max: 16384.0 prop: "height"}
            }
        }
        resolution_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        duration_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Duration" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 1.0 max: 1000000.0 suffix: " F" prop: "duration"}
            }
        }
        duration_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        session_band := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
            session_head := InkLabel{text: "SESSION" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.sm}}
        }
        autosave_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Autosave" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "On" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        autosave_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        undo_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Undo Depth" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "200" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        undo_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        appearance_band := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
            appearance_head := InkLabel{text: "APPEARANCE" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.sm}}
        }
        theme_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Theme" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "Live Dark" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        theme_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        scale_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "UI Scale" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "100 %" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        scale_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 残余は素の面（#4f4f4f）。枠・影・角丸は置かない
        settings_fill := View{width: Fill height: Fill}
    }
}

/// Settings が外へ出す唯一の口(`BrowserEditAction`/`FxStackAction` と同じ形)。
/// **どの comp かは言わない** — Document は1つしか無い。`field` は
/// `ScrubValue::prop` から来る名前(`"fps"`/`"width"`/`"height"`/`"duration"`)で、
/// 書き先の対応表は `main.rs::BackendBridge::apply_settings_action` に1つだけ持つ。
#[derive(Clone, Debug, Default)]
pub enum SettingsSurfaceAction {
    #[default]
    None,
    /// 確定値だけ(`ScrubValueAction::Changed` はここへ来ない — 1ジェスチャ = 1書き込み)。
    SetField { field: &'static str, value: f64 },
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct SettingsSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl SettingsSurface {
    /// comp の現在値を投影する(`FxStack::set_model`/`BrowserSurface::set_catalog` と
    /// 同じ形)。**この widget は Document を持たない** — 呼ぶのは
    /// `main.rs::install_settings`。
    pub fn set_composition(&mut self, cx: &mut Cx, composition: &Composition) {
        if let Some(mut cell) = self
            .view
            .widget(cx, ids!(frame_rate_row.well.value))
            .borrow_mut::<ScrubValue>()
        {
            cell.set_value(cx, composition.fps.as_f64());
        }
        if let Some(mut cell) = self
            .view
            .widget(cx, ids!(resolution_row.well.width_value))
            .borrow_mut::<ScrubValue>()
        {
            cell.set_value(cx, composition.width as f64);
        }
        if let Some(mut cell) = self
            .view
            .widget(cx, ids!(resolution_row.well.height_value))
            .borrow_mut::<ScrubValue>()
        {
            cell.set_value(cx, composition.height as f64);
        }
        if let Some(mut cell) = self
            .view
            .widget(cx, ids!(duration_row.well.value))
            .borrow_mut::<ScrubValue>()
        {
            cell.set_value(cx, composition.duration_frames as f64);
        }
        self.view.redraw(cx);
    }
}

impl WidgetNode for SettingsSurface {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for SettingsSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 子の action を一度受け取ってから流し直す(`BrowserSurface`/`FxStack` と
        // 同じ形)。4つのセルは動的な列ではなく宣言に名前で居るので、`FlatList` の
        // `items_with_actions` は要らない — 名前で直接引く。
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));

        let fps_cell = self.view.widget(cx, ids!(frame_rate_row.well.value));
        for action in actions.filter_widget_actions(fps_cell.widget_uid()) {
            if let ScrubValueAction::Committed { value, .. } = action.cast() {
                cx.widget_action(self.uid, SettingsSurfaceAction::SetField { field: "fps", value });
            }
        }

        let width_cell = self.view.widget(cx, ids!(resolution_row.well.width_value));
        for action in actions.filter_widget_actions(width_cell.widget_uid()) {
            if let ScrubValueAction::Committed { value, .. } = action.cast() {
                cx.widget_action(self.uid, SettingsSurfaceAction::SetField { field: "width", value });
            }
        }

        let height_cell = self.view.widget(cx, ids!(resolution_row.well.height_value));
        for action in actions.filter_widget_actions(height_cell.widget_uid()) {
            if let ScrubValueAction::Committed { value, .. } = action.cast() {
                cx.widget_action(self.uid, SettingsSurfaceAction::SetField { field: "height", value });
            }
        }

        let duration_cell = self.view.widget(cx, ids!(duration_row.well.value));
        for action in actions.filter_widget_actions(duration_cell.widget_uid()) {
            if let ScrubValueAction::Committed { value, .. } = action.cast() {
                cx.widget_action(self.uid, SettingsSurfaceAction::SetField { field: "duration", value });
            }
        }

        cx.extend_actions(actions);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
