//! Stage 表示枠。画素経路（Shared / import / render_into）は持たない。
//!
//! 正本: reference/mocks/stage-semantics.html(v5)。プロースそのものが仕様。
//!   上縁タブ = 視点の identity(何の視点か)。帯 = アイコンの言葉、値が意味の物だけ文字。
//!   letterbox(カメラ外の暗幕)は Camera 視点で comp 枠線を引かない(AE/Resolve 無枠)。
use makepad_widgets::*;

// 正本: Ableton Live 12 Dark 実画面（2026-08-26 添付）からのサンプル値。記憶で埋めない。
//   バー #3d3d3d / 面 #4f4f4f / 縁1px #2d2d2d / 窪み #282828
//   明字 #dddddd / 墨 #ababab / 琥珀 #c49a38
// 形の言語: フラット暗面・角丸ゼロ・影なし。縁は 1px 暗線か明度差だけ。数値は窪み矩形。
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 表示倍率の既定(%)。**帯の倍率はこの1つからしか作られない** —
    // 以前は宣言の "62%"、タブ切替が書く "USER VIEW · 62%"、⌂ が書く "100%" の
    // 3箇所が独立していて、⌂ を押すと片方だけ古いまま残った。
    // 値はここ、文字はここからの式、以後の書き換えは `StageChrome::zoom_percent`。
    let stage_zoom_percent = 62

    let IconButton = ButtonFlatterIcon{
        margin: 0
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: 0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: mod.tokens.ink.body}
    }

    let IconFlatButton = ButtonFlatIcon{
        margin: 0
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: 0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: mod.tokens.ink.strong}
    }

    // 窪み矩形 — バー上に沈む数値欄。動作なし、見た目だけ
    let ValueWell = SolidView{
        width: Fit
        height: 16
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 5 right: 5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.well
    }

    // 字形だけのボタン。SVG 資産が無い記号(⌂)向け。ValueWell と高さを合わせて帯に馴染む
    let GlyphButton = ButtonFlat{
        margin: 0
        width: Fit
        height: mod.tokens.size.chip
        padding: Inset{left: mod.tokens.space.s2 right: mod.tokens.space.s2}
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: mod.tokens.face.well
        draw_text.color: mod.tokens.accent.on
        draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}
    }

    // 視点タブ — 上縁 = これは何かの視点か(canon 冒頭: 「上縁タブ= 何の視点か」)。
    // 選択は draw_bg.color を直接書かず、押し込み面を instance shader で作る:
    // draw_bg.color は draw call 共有の uniform で兄弟ごとに効かない
    // (makepad-surface-colors-are-uniform の裁定と同根)。browser_surface.rs の
    // TabIcon/RailRow と同じ instance(self.active/self.hover/self.down)方式を踏襲し、
    // 文字色は反転しない(発注の明示指示 — 選択は面の押し込みだけで語る)
    let ViewTab = RadioButtonTabFlat{
        width: Fit
        height: mod.tokens.size.bar
        flow: Right
        align: Align{x: 0.5 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: mod.tokens.space.s2 right: 0}}
        // 反応は即時。ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sunk = max(self.active, self.down)
                let face = mod.tokens.face.bar.mix(mod.tokens.face.hover, self.hover).mix(mod.tokens.face.pressed, sunk)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                // 押し込みは縁で語る: 上が暗く、下が明るい。枠線では囲まない
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.area, sunk))
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.raised, sunk))
                return sdf.result
            }
        }
        draw_icon +: {color: mod.tokens.ink.glyph}
        draw_text.color: mod.tokens.ink.strong
        draw_text.color_active: mod.tokens.ink.strong
        draw_text.color_hover: mod.tokens.ink.strong
        draw_text.color_down: mod.tokens.ink.strong
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    mod.widgets.StageChromeBase = #(StageChrome::register_widget(vm))
    mod.widgets.StageChrome = set_type_default() do mod.widgets.StageChromeBase{
        zoom_percent: stage_zoom_percent
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.down

        // 視点タブ帯 — カメラレイヤーが増えるとタブが増える(canon: 拡張性がタブ採用の理由)。
        // UI 文字は English(english-first)。排他選択は StageChrome::handle_event が
        // RadioButtonSet::selected で持つ(main.rs の browser_radio_groups と同型、
        // ここでは StageChrome 内で完結させて main.rs には漏らさない)
        stage_tabs := SolidView{width: Fill height: mod.tokens.size.bar flow: Right show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            camera_tab := ViewTab{text: "Camera" draw_icon +: {svg: crate_resource("self://resources/icons/camera.svg")}}
            user_tab := ViewTab{text: "User View" draw_icon +: {svg: crate_resource("self://resources/icons/user_view.svg")}}
            tabs_spacer := SolidView{width: Fill height: mod.tokens.rule.size}
            tool_select := IconFlatButton{width: 30 height: 20 draw_bg.color: mod.tokens.face.well icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/select.svg") color: mod.tokens.accent.on}}
            tool_shape := IconButton{width: 30 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg")}}
            tool_pen := IconButton{width: 30 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/pen.svg")}}
        }
        tabs_edge := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.face.down}
        // letterbox = カメラ外の暗幕(canon: 「letterbox = カメラ外の暗幕」)。
        // Camera 視点では comp 枠線を引かない(canon S0: AE/Resolve 無枠)。
        // 旧 comp_frame(1px 縁の入れ子)は撤去 — 枠を描かずに letterbox が comp に直に接する
        stage_void := SolidView{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4 top: mod.tokens.space.s4 bottom: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.desktop
            comp := SolidView{width: 720 height: 405 flow: Overlay show_bg: true new_batch: true draw_bg.color: #x000000
                // min_width/min_height: SharedPresentable textures have no
                // vec_width_height(), so Image falls back to these (default 0
                // = zero-sized quad = invisible stage).
                // #x000000 = 映像の無信号黒。letterbox の面トークンとは別物(絶対黒)なので
                // トークン化しない
                stage_frame := Image{width: Fill height: Fill fit: ImageFit.Smallest}
                stage_error := InkLabel{width: Fill height: Fill align: Align{x: 0.5 y: 0.5} text: "" draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        band_edge := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.face.down}
        // 状態帯 = アイコンの言葉(canon: 「帯の言葉はアイコン(文字で説明しない — 値が意味の
        // 物だけ文字)」)。解像度/fps/倍率は値そのものが意味なので文字のまま。
        // 高さはトークンから(比の注記「帯高:pane高 ≈ 0.04」に名前で対応する size.status)
        stage_band := SolidView{width: Fill height: mod.tokens.size.status flow: Right spacing: 8 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            // ▦ = 市松(canon: 「帯の ▦(AE の透明グリッドアイコンと同型)」)。本体機能、予約地ではない
            check := IconButton{width: mod.tokens.size.bar height: mod.tokens.size.well icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/checker.svg")}}
            // 予約地の入口 — 方眼シート束(方眼/三分割/黄金比)+ Safe areas は帯のアイコン1個から
            // (canon: 「予約地: 方眼シート束... + Safe areas。入口は帯のアイコン(View 系)」)。
            // browser_surface.rs の RailRowReserved と同じ扱い: 薄字(ink.faint)・on_click なし
            reserved_view := IconButton{width: mod.tokens.size.bar height: mod.tokens.size.well icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/safe.svg") color: mod.tokens.ink.faint}}
            source_well := ValueWell{
                live_dot := SolidView{width: 5 height: 5 margin: Inset{right: mod.tokens.space.s1} draw_bg.color: mod.tokens.accent.on}
                live_source := InkLabel{text: "RERUN" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.xs}}
            }
            // 視点依存の情報だけ(canon: User View 中は倍率+⌂)。Camera 中は空 = タブと同じ語を繰り返さない。
            // **倍率はここに書かない** — 倍率の居場所は zoom_well 1箇所(F3)
            stage_mode := InkLabel{text: "" width: Fit padding: Inset{left: mod.tokens.space.s2} draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: mod.tokens.text.xs}}
            resolution_well := ValueWell{
                resolution := InkLabel{text: "1920 × 1080" width: 76 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: 8}}
            }
            frame_rate_well := ValueWell{
                frame_rate := InkLabel{text: "30 fps" width: 42 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: 8}}
            }
            off_frame_dot := SolidView{width: mod.tokens.space.s2 height: mod.tokens.space.s2 draw_bg.color: mod.tokens.accent.on}
            selection_state := InkLabel{text: "CHORUS LYRICS · OFF FRAME" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 8}}
            stage_band_spacer := SolidView{width: Fill height: mod.tokens.rule.size}
            // User View 中はここに倍率+⌂ 復帰(canon: 「User View 中はここに倍率+⌂ 復帰」)
            zoom_well := ValueWell{
                zoom := InkLabel{text: "" + stage_zoom_percent + "%" width: 30 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: 8}}
            }
            // ⌂ = home/auto 復帰。svg 資産が無いので字形ボタン。押せば動く実の操作にする
            // (Q0: 触れそうで触れない物を作らない)。
            // splash の on_click で `ui.zoom.set_text` を書くと「文字を書く」が操作の
            // 本体になり、状態が文字列側へ逃げる。押下は Rust 側で受けて
            // `zoom_percent` を動かし、文字はそこから作り直す(F3)
            home_zoom := GlyphButton{text: "⌂"}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct StageChrome {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    // 視点タブの既定選択(Camera)を初回イベントで一度だけ立てる。RadioButton の
    // active は instance でありスクリプト側の宣言的な既定選択が無いため
    #[rust]
    tabs_selected_once: bool,
    /// 表示倍率(%)。**帯の倍率表示の唯一の持ち主**。既定は宣言の
    /// `stage_zoom_percent`(裁定269: 調整する値は script_mod! 側)。
    #[live]
    zoom_percent: u32,
    /// 視点タブの選択。0 = Camera / 1 = User View。帯の視点名はここから導出する。
    #[rust]
    stage_view: usize,
}

impl StageChrome {
    /// 帯の文字はすべてここで作る。持っているのは `stage_view` と `zoom_percent` の
    /// 2つだけで、`stage_mode` も `zoom` もその投影 — どちらかへ直接文字を書く道を
    /// 残すと、Home を押しても片方だけ古いまま残る(F3 の再発)。
    fn project_band(&self, cx: &mut Cx) {
        self.view
            .widget(cx, ids!(stage_band.stage_mode))
            .as_label()
            .set_text(cx, if self.stage_view == 0 { "" } else { "USER VIEW" });
        self.view
            .widget(cx, ids!(stage_band.zoom_well.zoom))
            .as_label()
            .set_text(cx, &format!("{}%", self.zoom_percent));
    }
}

impl WidgetNode for StageChrome {
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

impl Widget for StageChrome {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 視点タブ(stage_tabs.camera_tab / user_tab)の排他は RadioButtonSet::selected の
        // 担当 — main.rs の browser_radio_groups と同型だが、ここは StageChrome の中だけで
        // 完結させる(main.rs には stage_tabs の存在すら要らない)
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        let mut band_dirty = false;
        if let Some(index) = self
            .view
            .radio_button_set(cx, ids_array!(stage_tabs.camera_tab, stage_tabs.user_tab))
            .selected(cx, &actions)
        {
            // 帯の視点名はタブ選択の結果。splash の `on_click` は RadioButton に無いので
            // (Browser の TabIcon で踏んだのと同じ)、識別は Rust 側が書く。
            // 書くのは**状態**で、文字はこの後の `project_band` が作る
            self.stage_view = index;
            band_dirty = true;
        }

        // ⌂ = home/auto 復帰。倍率という1つの状態を動かすだけで、文字には触らない
        if self
            .view
            .widget(cx, ids!(stage_band.home_zoom))
            .as_button()
            .clicked(&actions)
        {
            self.zoom_percent = 100;
            band_dirty = true;
        }

        if !self.tabs_selected_once {
            self.tabs_selected_once = true;
            // 宣言側の文字はまだ既定のままなので、状態からの投影を一度通す
            band_dirty = true;
            // 既定は Camera(canon: 上縁タブ=視点の identity。書き出しと同一のカメラ視点が既定)
            if let Some(camera_tab) = self
                .view
                .radio_button_set(cx, ids_array!(stage_tabs.camera_tab, stage_tabs.user_tab))
                .iter()
                .next()
            {
                camera_tab.set_active(cx, true, Animate::No);
            }
        }

        if band_dirty {
            self.project_band(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
