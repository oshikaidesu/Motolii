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
//! SESSION 帯(Autosave / Undo Depth)と APPEARANCE 帯の Theme は
//! まだ見た目用ダミー(出典なし) — store 側にこれらの設定を持つ場所がまだ無い。
//! **UI Scale だけは本物** — 正本は `App::ui_scale_percent`(`main.rs::set_ui_scale`
//! が唯一の書き手)で、`SettingsSurface::set_ui_scale` がその投影。
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
        // (`SettingsSurface::set_composition`)。SESSION と APPEARANCE の Theme は
        // 波1のダミーのまま(UI Scale だけ `set_ui_scale` で本物)
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

        // TIMELINE 帯 — timeline_surface.rs の #[live] 数値をここから直接動かす
        // (利用者裁定: いちいち口頭で指示するより自分で触って決めたい)。
        // 色系(レーンパレット・M/S/L意味色)はまだ #[live] 化していないので出さない。
        timeline_band := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
            timeline_head := InkLabel{text: "TIMELINE" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.sm}}
        }
        row_height_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Row Height" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 14.0 max: 60.0 value: 26.0 suffix: " px" prop: "timeline_row_height"}
            }
        }
        row_height_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        rail_width_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Rail Width" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 60.0 max: 400.0 value: 150.0 suffix: " px" prop: "timeline_rail_width"}
            }
        }
        rail_width_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        ruler_height_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Ruler Height" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 12.0 max: 60.0 value: 22.0 suffix: " px" prop: "timeline_ruler_height"}
            }
        }
        ruler_height_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        trim_handle_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Trim Handle" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 0.5 precision: 1 min: 2.0 max: 20.0 value: 6.0 suffix: " px" prop: "timeline_trim_handle_width"}
            }
        }
        trim_handle_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        tick_floor_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Tick Floor" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 1.0 precision: 0 min: 10.0 max: 100.0 value: 40.0 suffix: " px" prop: "timeline_tick_row_floor"}
            }
        }
        tick_floor_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        band_alpha_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Zebra Alpha" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 0.005 precision: 3 min: 0.0 max: 0.2 value: 0.030 prop: "timeline_band_alpha"}
            }
        }
        band_alpha_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        tick_fade_from_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Fade From" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 0.5 precision: 1 min: 0.0 max: 30.0 value: 9.0 prop: "timeline_tick_fade_from"}
            }
        }
        tick_fade_from_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        tick_fade_to_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Fade To" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 0.5 precision: 1 min: 0.0 max: 40.0 value: 18.0 prop: "timeline_tick_fade_to"}
            }
        }
        tick_fade_to_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        playhead_scale_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Playhead Scale" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := ScrubValue{width: Fill step: 0.1 precision: 1 min: 0.5 max: 4.0 value: 1.5 suffix: "x" prop: "timeline_playhead_scale"}
            }
        }
        playhead_scale_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

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

    /// APPEARANCE 帯の UI Scale 欄(`main.rs::set_ui_scale` が唯一の書き手 —
    /// `⌘+`/`⌘-`/`⌘0` を押すたびここも一緒に引き直す)。SESSION/APPEARANCE の
    /// 残り(Autosave/Undo Depth/Theme)はまだ store に置き場が無い波1のダミーの
    /// ままだが、UI Scale は既に `App::ui_scale_percent` が正本を持っている。
    pub fn set_ui_scale(&mut self, cx: &mut Cx, percent: i32) {
        self.view
            .widget(cx, ids!(scale_row.well.value))
            .as_label()
            .set_text(cx, &format!("{percent} %"));
    }

    /// TIMELINE 帯の9欄。正本は `TimelineSurface` 自身の `#[live]` フィールド —
    /// ここは常にそこから読んで映すだけ(`set_composition` と同じ形)。
    pub fn set_timeline_tuning(&mut self, cx: &mut Cx, get: impl Fn(&str) -> Option<f64>) {
        macro_rules! project {
            ($path:expr, $field:expr) => {
                if let Some(value) = get($field) {
                    if let Some(mut cell) = self.view.widget(cx, $path).borrow_mut::<ScrubValue>() {
                        cell.set_value(cx, value);
                    }
                }
            };
        }
        project!(ids!(row_height_row.well.value), "timeline_row_height");
        project!(ids!(rail_width_row.well.value), "timeline_rail_width");
        project!(ids!(ruler_height_row.well.value), "timeline_ruler_height");
        project!(ids!(trim_handle_row.well.value), "timeline_trim_handle_width");
        project!(ids!(tick_floor_row.well.value), "timeline_tick_row_floor");
        project!(ids!(band_alpha_row.well.value), "timeline_band_alpha");
        project!(ids!(tick_fade_from_row.well.value), "timeline_tick_fade_from");
        project!(ids!(tick_fade_to_row.well.value), "timeline_tick_fade_to");
        project!(ids!(playhead_scale_row.well.value), "timeline_playhead_scale");
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

        // TIMELINE 帯の9欄。field 名がそのまま TimelineSurface::set_tuning_value の
        // キーになる(main.rs の作法と同じ — 対応表はそこ1箇所)。
        macro_rules! capture_tuning {
            ($path:expr, $field:expr) => {
                let cell = self.view.widget(cx, $path);
                for action in actions.filter_widget_actions(cell.widget_uid()) {
                    if let ScrubValueAction::Committed { value, .. } = action.cast() {
                        cx.widget_action(
                            self.uid,
                            SettingsSurfaceAction::SetField { field: $field, value },
                        );
                    }
                }
            };
        }
        capture_tuning!(ids!(row_height_row.well.value), "timeline_row_height");
        capture_tuning!(ids!(rail_width_row.well.value), "timeline_rail_width");
        capture_tuning!(ids!(ruler_height_row.well.value), "timeline_ruler_height");
        capture_tuning!(ids!(trim_handle_row.well.value), "timeline_trim_handle_width");
        capture_tuning!(ids!(tick_floor_row.well.value), "timeline_tick_row_floor");
        capture_tuning!(ids!(band_alpha_row.well.value), "timeline_band_alpha");
        capture_tuning!(ids!(tick_fade_from_row.well.value), "timeline_tick_fade_from");
        capture_tuning!(ids!(tick_fade_to_row.well.value), "timeline_tick_fade_to");
        capture_tuning!(ids!(playhead_scale_row.well.value), "timeline_playhead_scale");

        cx.extend_actions(actions);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
