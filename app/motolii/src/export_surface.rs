//! Export パネル枠。範囲・形式の意味は波1。iced は引かない。
//! 見た目の正本は `mod.tokens`(裁定267: Ableton の identity は palette でなく形の文法。
//! 面/字/線/選択/accent は Live 12 `.ask` 実機抽出由来)。ここに生の hex を書かない —
//! 書いた瞬間、皮の差し替えがこの1枚だけ効かなくなる。
//! 進捗の塗りは `.ask` の `TransportProgress`(= 琥珀)。以前の青緑は出典なしの即興だった。
//! 形: 枠線・角丸・影なし。欄は窪み（暗面）で示す。進捗は細い溝 + 明るい塗り。
//! 空域は「Drop Audio Effects Here」調（面のまま中央に薄字だけ）。
//! 進捗読取・状態帯の幾何は chrome/parts の ChromeProgressReadout / ChromeStatus を
//! 手本に写した（main.rs の読み込み順で本 mod が chrome より先のため直接参照しない）。
//! 行の値(Range/Format/Audio)は見た目用ダミー（出典なし、波1の範囲）。進捗
//! (done/total/pct/塗り幅)と Destination と状態帯は host(main.rs)が
//! `ExportSurface::set_progress`/`set_destination`/`set_export_status` で書く —
//! Document は直接読まない(host が Engine/Document を持つ、この widget は器)。
use makepad_widgets::*;
use std::path::PathBuf;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 起動直後の idle 見た目だけの初期値(0/300)。エクスポート中の実値は
    // `ExportSurface::set_progress` が widget を直接書き換える(このDSL式は
    // 再評価されない — hot reload で宣言状態へ戻った直後だけこの初期値に戻る)。
    let export_done_frames = 0
    let export_total_frames = 300
    let export_percent = 100 * export_done_frames / export_total_frames

    mod.widgets.ExportSurfaceBase = #(ExportSurface::register_widget(vm))
    mod.widgets.ExportSurface = set_type_default() do mod.widgets.ExportSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // 頭帯 — Live の device title bar と同型(明帯 + 暗字 = 極性反転、橙の活性点)
        export_head := SolidView{width: Fill height: mod.tokens.size.toolbar flow: Right spacing: mod.tokens.space.s3 align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.head
            power_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: mod.tokens.accent.on}
            title := InkLabel{text: "Export" width: Fill draw_text.color: mod.tokens.ink.on_fill draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xl}}
        }
        head_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 以下の欄値はダミー（出典なし）。意味書きは波1
        range_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Range" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "0 – 300 F" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        range_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        format_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Format" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "MP4 · H.264" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        format_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        audio_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Audio" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "AAC · 48 kHz" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        audio_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        destination_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Destination" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "motolii.mp4" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        destination_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 動詞 — Export/Still は先が dialog(選ぶはここ、回すは host)。Cancel は
        // 実行中かどうかを問わず押せる(host が「実行中でなければ無視」を判定)。
        action_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right spacing: mod.tokens.space.s3 align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            export_button := ChromeButton{text: "Export"}
            still_button := ChromeButton{text: "Still"}
            cancel_button := ChromeButton{text: "Cancel"}
        }
        action_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 進捗読取 — `0 / 300 (0%)`（ChromeProgressReadout の並びを手本）。文字は式で作る
        progress_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} spacing: 2 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            progress_label := InkLabel{text: "Progress" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            done := InkLabel{text: "" + export_done_frames width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            sep := InkLabel{text: "/" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            total := InkLabel{text: "" + export_total_frames width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            pct := InkLabel{text: "(" + export_percent + "%)" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
        }
        // 細い溝 + 明るい塗り。塗り幅は進捗比そのもの: `Fill{weight}` は兄弟間の相対配分
        // なので、済み:残り = done:(total-done) と書けば溝の幅に依らず比が保たれる。
        // 定数 px を置くと読取と食い違う(それが直前の欠陥だった)
        progress_track := SolidView{width: Fill height: 3 flow: Right margin: Inset{left: 8 right: 8 bottom: 8} show_bg: true draw_bg.color: mod.tokens.face.display
            progress_fill := SolidView{width: Fill{weight: export_done_frames} height: Fill show_bg: true draw_bg.color: mod.tokens.accent.on}
            progress_rest := View{width: Fill{weight: export_total_frames - export_done_frames} height: Fill}
        }

        // 空域 — 面のまま中央に薄字だけ（Drop Audio Effects Here 調）
        export_empty := SolidView{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            empty_hint := InkLabel{text: "No Export Running" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
        }

        // 状態帯 — ChromeStatus の幾何（高 28・一行）を手本。文言はダミー（出典なし）
        status_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        export_status := SolidView{width: Fill height: 28 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} spacing: 8 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            status_label := InkLabel{text: "Ready" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct ExportSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for ExportSurface {
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

impl Widget for ExportSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // browser_surface.rs の Import ボタンと同じ形: 「選ぶ」(OS dialog)はここ、
        // 「回す」(Engine/Document を触る)は host(main.rs)。
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));

        if self
            .view
            .button(cx, ids!(action_row.export_button))
            .clicked(&actions)
        {
            if let Some(path) = pick_export_path() {
                cx.widget_action(self.uid, ExportSurfaceAction::StartExport(path));
            }
        }
        if self
            .view
            .button(cx, ids!(action_row.still_button))
            .clicked(&actions)
        {
            if let Some(path) = pick_still_path() {
                cx.widget_action(self.uid, ExportSurfaceAction::StartStill(path));
            }
        }
        if self
            .view
            .button(cx, ids!(action_row.cancel_button))
            .clicked(&actions)
        {
            cx.widget_action(self.uid, ExportSurfaceAction::Cancel);
        }

        cx.extend_actions(actions);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ExportSurface {
    /// 進捗の実値。`done`/`total`/`pct` の読取と、塗り幅(`progress_fill`/
    /// `progress_rest` の `Fill{weight}`)を両方書き換える — 読取と塗りが
    /// 食い違っていた旧欠陥(ダミー値時代)と同じ形にしない。
    pub fn set_progress(&mut self, cx: &mut Cx, done: i64, total: i64) {
        let done = done.max(0);
        let total = total.max(done);
        let percent = if total > 0 { 100 * done / total } else { 0 };

        self.view
            .widget(cx, ids!(progress_row.done))
            .as_label()
            .set_text(cx, &done.to_string());
        self.view
            .widget(cx, ids!(progress_row.total))
            .as_label()
            .set_text(cx, &total.to_string());
        self.view
            .widget(cx, ids!(progress_row.pct))
            .as_label()
            .set_text(cx, &format!("({percent}%)"));

        let rest = (total - done).max(0);
        if let Some(mut fill) = self
            .view
            .widget(cx, ids!(progress_track.progress_fill))
            .borrow_mut::<View>()
        {
            fill.walk.width = Size::Fill {
                weight: done as f64,
                min: None,
                max: None,
            };
        }
        if let Some(mut rest_view) = self
            .view
            .widget(cx, ids!(progress_track.progress_rest))
            .borrow_mut::<View>()
        {
            rest_view.walk.width = Size::Fill {
                weight: rest as f64,
                min: None,
                max: None,
            };
        }
        self.view.redraw(cx);
    }

    /// 状態帯(下段)の文言。
    pub fn set_export_status(&mut self, cx: &mut Cx, text: &str) {
        self.view
            .widget(cx, ids!(export_status.status_label))
            .as_label()
            .set_text(cx, text);
    }

    /// Destination 欄。選んだ後だけ host が呼ぶ(選ぶまではダミー文言のまま)。
    pub fn set_destination(&mut self, cx: &mut Cx, text: &str) {
        self.view
            .widget(cx, ids!(destination_row.well.value))
            .as_label()
            .set_text(cx, text);
    }
}

/// この widget が運ぶ意図。「選んだ path」だけを運ぶ — Engine/Document を触る
/// 判断は host(`main.rs`)の仕事(`BrowserSurfaceAction` と同じ分担)。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum ExportSurfaceAction {
    #[default]
    None,
    /// 動画書き出し開始。選んだ出力先。
    StartExport(PathBuf),
    /// 静止画(playhead のフレーム)書き出し開始。選んだ出力先。
    StartStill(PathBuf),
    /// 実行中の書き出しを中断(実行中が無ければ host が無視する)。
    Cancel,
}

/// OS の save dialog(`pick_media_path` と同じ流儀 — 裁定176、自前のファイル
/// ブラウザを作らない)。
fn pick_export_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export")
        .add_filter("MP4", &["mp4"])
        .set_file_name("motolii.mp4")
        .save_file()
}

fn pick_still_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export Still")
        .add_filter("PNG", &["png"])
        .set_file_name("motolii.png")
        .save_file()
}
