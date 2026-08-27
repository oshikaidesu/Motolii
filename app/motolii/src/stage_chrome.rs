//! Stage 表示枠。画素経路（Shared / import / render_into）は持たない。
//!
//! 正本: reference/mocks/stage-semantics.html(v5)。プロースそのものが仕様。
//!   上縁タブ = 視点の identity(何の視点か)。帯 = アイコンの言葉、値が意味の物だけ文字。
//!   letterbox(カメラ外の暗幕)は Camera 視点で comp 枠線を引かない(AE/Resolve 無枠)。
//!
//! **User View の観測視点もここが持つ**(裁定157/272)。出力カメラではないので
//! store へ書かない — front ローカル状態である。ここが持つのは「どこから見るか」
//! だけで、それを world 画素へ換算して engine の
//! `render_frame_into_with_view_camera` へ渡すのは comp の寸法を知っている
//! main.rs(継ぎ目)の仕事。
use crate::gesture_input::{GestureDevice, GesturePhase, GestureSample};
use makepad_widgets::*;

// 正本: Ableton Live 12 Dark 実画面（2026-08-26 添付）からのサンプル値。記憶で埋めない。
//   バー #3d3d3d / 面 #4f4f4f / 縁1px #2d2d2d / 窪み #282828
//   明字 #dddddd / 墨 #ababab / 琥珀 #c49a38
// 形の言語: フラット暗面・角丸ゼロ・影なし。縁は 1px 暗線か明度差だけ。数値は窪み矩形。
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 初期倍率 = **fit**(裁定274)。観測カメラの zoom = 1.0 は「comp がちょうど
    // 画面に収まる画角」(`motolii_core::camera_projection` の base_distance が
    // comp.height を垂直画角いっぱいに取る)なので、fit は 100% と同義である。
    //
    // 帯の倍率は宣言では**初期表示だけ**を作る。走っている間の正本は
    // `StageChrome::view_camera.zoom` 1箇所で、文字はその投影
    // (`project_band`)。以前は宣言の "62%"、タブ切替が書く文字、⌂ が書く
    // "100%" の3箇所が独立していて、⌂ を押すと片方だけ古いまま残った。
    let stage_zoom_fit_percent = 100

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
        // 見回しの手触り。--hot が拾えるのは script_mod! だけで、Rust の const は
        // 再ビルドしないと変わらない(裁定269)。
        // スクロール何画素で倍率が2倍になるか。
        view_zoom_octave_px: 240.0
        // 倍率の下限/上限。AE のビューア(1%〜1600%)と同じ桁を意図している。
        view_zoom_min: 0.05
        view_zoom_max: 32.0
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
                zoom := InkLabel{text: "" + stage_zoom_fit_percent + "%" width: 30 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: 8}}
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

/// User View の観測視点。**Document へ書かない**(裁定272 — 出力カメラではない)。
///
/// 単位は **comp の寸法に対する比**。`motolii_engine::ObservationCamera` は world
/// 画素で持つが、StageChrome は comp の画素数を知らない(意味は store の持ち物で、
/// chrome は面しか持たない)ので、ここでは寸法に依らない形で持ち、world 画素への
/// 換算は comp を知っている継ぎ目(main.rs `try_present_shared`)が最後に1回だけ行う。
/// **既にやっている物**: `motolii_engine::ObservationCamera` 本体(pan+zoom、roll 無し)
/// — この型はその front 側の単位違いの写しであって、新しい概念ではない。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageViewCamera {
    /// comp 中心からのパン量。`[0]` は comp 幅、`[1]` は comp 高さに対する比。
    /// 符号は `ResolvedCamera::center` と同じ(x 右・y 下が正)。
    pub pan_fraction: [f64; 2],
    /// `1.0` = fit(comp がちょうど画角に収まる、裁定274)。大きいほど拡大。
    pub zoom: f64,
}

impl Default for StageViewCamera {
    /// 初期視点は fit(裁定274)。`ObservationCamera::default()` と同じ姿。
    fn default() -> Self {
        Self {
            pan_fraction: [0.0, 0.0],
            zoom: 1.0,
        }
    }
}

/// Stage 面が外へ出す唯一の意図。**視点が変わった**とだけ言い、絵を描き直すかどうかは
/// 継ぎ目(main.rs)が決める(timeline の `TimelineSurfaceAction` と同型)。
///
/// タブの切替も観測カメラの移動も同じ1本で運ぶ — どちらも「Stage に出る絵が変わった」
/// という同じ帰結しか持たないので、口を2本にすると呼び手が同じ処理を2回書くことになる。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum StageChromeAction {
    #[default]
    None,
    /// 視点(タブ or 観測カメラ)が変わった。Stage の絵を描き直す必要がある。
    ViewChanged,
}

/// 掴んで見回している最中の指。**1本だけ**(観測視点に多点は要らない)。
#[derive(Clone, Copy, Debug)]
struct StageViewDrag {
    last_x: f64,
    last_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StageScrollMode {
    Pan,
    Zoom,
}

/// 1本の phased トラックパッド流を持つ。`timeline_surface::TimelineScrollGesture` と
/// **同じ形**(モードを最初に1回だけ決め、OS の慣性まで持ち主を保つ)。
///
/// 軸ロックを持たない点だけが違う: timeline は横=時間・縦=レーンで**別の動詞**なので
/// 途中で入れ替わると事故になるが、Stage は両軸とも同じ動詞(パン)なので、
/// 斜めの指はそのまま斜めに動くのが正しい。
#[derive(Clone, Copy, Debug, Default)]
struct StageViewScrollGesture {
    mode: Option<StageScrollMode>,
    owns_momentum: bool,
}

/// スクロール流1サンプルの意味。ズームは**倍率**(加算ではなく乗算)で運ぶ。
#[derive(Clone, Copy, Debug, PartialEq)]
enum StageScroll {
    /// 画面画素でのスクロール量。
    Pan([f64; 2]),
    /// 現在の zoom に掛ける倍率。
    Zoom(f64),
}

impl StageViewScrollGesture {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn update_sample(&mut self, sample: GestureSample, octave_px: f64) -> Option<StageScroll> {
        match sample.phase {
            GesturePhase::Catch | GesturePhase::MomentumEnd | GesturePhase::Cancel => {
                self.reset();
                return None;
            }
            GesturePhase::Begin => self.reset(),
            GesturePhase::Momentum if !self.owns_momentum => return None,
            GesturePhase::Instant => self.reset(),
            GesturePhase::Update | GesturePhase::End | GesturePhase::Momentum => {}
        }

        let native_scale = (sample.scale_ratio - 1.0).abs() > f64::EPSILON;
        let mode = *self
            .mode
            .get_or_insert(if native_scale || sample.modifiers.alt {
                StageScrollMode::Zoom
            } else {
                StageScrollMode::Pan
            });

        let action = match mode {
            StageScrollMode::Zoom => {
                let factor = if native_scale {
                    sample.scale_ratio
                } else {
                    // 支配的な成分ぶんだけ倍率へ写す。上へ回す(= 負)と拡大 —
                    // 下の Pan と**同じ1つの約束**(画面を押し上げる向きが正)から出る。
                    let delta = if sample.translation[1].abs() >= sample.translation[0].abs() {
                        sample.translation[1]
                    } else {
                        sample.translation[0]
                    };
                    if delta.abs() <= f64::EPSILON || octave_px <= 0.0 {
                        1.0
                    } else {
                        2f64.powf(-delta / octave_px)
                    }
                };
                ((factor - 1.0).abs() > f64::EPSILON).then_some(StageScroll::Zoom(factor))
            }
            StageScrollMode::Pan => {
                // 縦入力を横パンへ振り替える shift は timeline と同じ語彙。
                // ホイールしか無いマウスでも横を見回せるようにするため。
                let scroll = if sample.modifiers.shift && sample.device == GestureDevice::Wheel {
                    [sample.translation[1], 0.0]
                } else {
                    sample.translation
                };
                (scroll[0].abs() > f64::EPSILON || scroll[1].abs() > f64::EPSILON)
                    .then_some(StageScroll::Pan(scroll))
            }
        };

        if matches!(
            sample.phase,
            GesturePhase::Begin | GesturePhase::Update | GesturePhase::End
        ) {
            self.owns_momentum |= action.is_some();
        }
        if sample.phase == GesturePhase::Instant {
            self.reset();
        }
        action
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
    /// 視点タブの選択。0 = Camera / 1 = User View。帯の視点名はここから導出する。
    #[rust]
    stage_view: usize,
    /// **User View の観測視点の唯一の持ち主**。`#[rust]` なので live edit で
    /// 宣言状態へ戻らない(見回した先が hot reload で飛ばない)。
    #[rust]
    view_camera: StageViewCamera,
    #[rust]
    view_drag: Option<StageViewDrag>,
    #[rust]
    view_scroll: StageViewScrollGesture,
    // 手触りのチューニング値。const だと --hot で拾えず再ビルドが要るので、
    // script_mod! の type-default から埋まる #[live] フィールドとして持つ(裁定269)。
    /// スクロール何画素で倍率が2倍になるか。
    #[live(240.0)]
    view_zoom_octave_px: f64,
    /// 倍率の下限/上限。AE のビューア(1%〜1600%)と同じ桁を意図している。
    #[live(0.05)]
    view_zoom_min: f64,
    #[live(32.0)]
    view_zoom_max: f64,
}

impl StageChrome {
    /// User View タブが選ばれているか。0 = Camera / 1 = User View。
    fn is_user_view(&self) -> bool {
        self.stage_view == 1
    }

    /// 継ぎ目(main.rs)が読む口。**Camera 視点では `None`** — 出力カメラは front から
    /// 動かさないので、観測カメラを渡す相手が居ない(裁定157)。
    pub fn view_camera(&self) -> Option<StageViewCamera> {
        self.is_user_view().then_some(self.view_camera)
    }

    fn zoom_percent(&self) -> u32 {
        (self.view_camera.zoom * 100.0).round().max(1.0) as u32
    }

    /// 帯の文字はすべてここで作る。持っているのは `stage_view` と `view_camera` の
    /// 2つだけで、`stage_mode` も `zoom` もその投影 — どちらかへ直接文字を書く道を
    /// 残すと、⌂ を押しても片方だけ古いまま残る(F3 の再発)。
    ///
    /// 倍率と ⌂ は **User View の間だけ**出す(canon: 「User View 中はここに倍率+⌂ 復帰」)。
    /// Camera 視点では倍率は利用者の持ち物ではないので、押しても何も起きない ⌂ を
    /// 置いておくのは Q0(触れそうで触れない)違反になる。
    fn project_band(&self, cx: &mut Cx) {
        let user_view = self.is_user_view();
        self.view
            .widget(cx, ids!(stage_band.stage_mode))
            .as_label()
            .set_text(cx, if user_view { "USER VIEW" } else { "" });
        self.view
            .widget(cx, ids!(stage_band.zoom_well.zoom))
            .as_label()
            .set_text(cx, &format!("{}%", self.zoom_percent()));
        self.view
            .widget(cx, ids!(stage_band.zoom_well))
            .set_visible(cx, user_view);
        self.view
            .widget(cx, ids!(stage_band.home_zoom))
            .set_visible(cx, user_view);
    }

    /// comp の板(黒い矩形)の Area。見回しの座標系はこの矩形が全部で、
    /// 帯・タブ・letterbox の上では見回しが始まらない。
    ///
    /// **前提**: 板いっぱいに comp が映っていること。`comp` は宣言で 720×405 固定、
    /// 中の `stage_frame` は `ImageFit.Smallest` なので、comp が 16:9 でない時だけ
    /// 板の中にさらに余白が出て、指1画素あたりの換算がその余白ぶんずれる。
    /// **板が comp の縦横比を取らないのは既存の性質**(この面はまだ comp の寸法を
    /// 知らない)で、見回しが持ち込んだ話ではない — 直すなら板の側を直す。
    fn comp_area(&self, cx: &mut Cx) -> Area {
        self.view.widget(cx, ids!(stage_void.comp)).area()
    }

    /// 指の移動を視点の移動へ。**画の中身が指に付いてくる**向き(掴んで引きずる)。
    ///
    /// 換算は comp 板の実寸だけで閉じる: 板の幅ぜんぶが「zoom=1 のとき comp 幅ぜんぶ」
    /// なので、板の幅に対する比 ÷ zoom が、そのまま comp 幅に対する比になる。
    /// comp の画素数を知らなくても正しいのはこのため。
    fn pan_by_pixels(&mut self, delta: [f64; 2], rect: Rect) {
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        let zoom = self.view_camera.zoom;
        self.view_camera.pan_fraction[0] -= delta[0] / (rect.size.x * zoom);
        self.view_camera.pan_fraction[1] -= delta[1] / (rect.size.y * zoom);
    }

    /// 倍率を掛ける。**ポインタの下の点を動かさない**(AE のコンポパネルのホイールと同じ)。
    ///
    /// 固定するのは z=0 平面上の点。憲法2 の世界は一つなので z≠0 の層は
    /// 完全には貼り付かないが、2D の住所である z=0 を基準に取るのが約束
    /// (`motolii_core::camera_projection` の base_distance も z=0 で立っている)。
    fn zoom_by(&mut self, factor: f64, pointer: [f64; 2], rect: Rect) {
        if !factor.is_finite() || factor <= 0.0 || rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        let before = self.view_camera.zoom;
        let after = (before * factor).clamp(self.view_zoom_min, self.view_zoom_max);
        if (after - before).abs() <= f64::EPSILON {
            return;
        }
        // 板の中心からの比(±0.5 が板の端)。
        let u = [
            (pointer[0] - rect.pos.x - rect.size.x * 0.5) / rect.size.x,
            (pointer[1] - rect.pos.y - rect.size.y * 0.5) / rect.size.y,
        ];
        for axis in 0..2 {
            let under_pointer = self.view_camera.pan_fraction[axis] + u[axis] / before;
            self.view_camera.pan_fraction[axis] = under_pointer - u[axis] / after;
        }
        self.view_camera.zoom = after;
    }

    /// User View の見回し。**Camera 視点では一切反応しない** — 出力カメラを front から
    /// 動かす道は無い(裁定157: 見回しても書き出される絵は変わらない、が分離の意味)。
    ///
    /// 返り値 = 視点が動いたか。
    fn handle_view_gestures(&mut self, cx: &mut Cx, event: &Event) -> bool {
        let area = self.comp_area(cx);
        if area == Area::Empty {
            return false;
        }
        let rect = area.rect(cx);
        let user_view = self.is_user_view();
        let before = self.view_camera;
        match event.hits(cx, area) {
            Hit::FingerHoverIn(_) | Hit::FingerHoverOver(_) => {
                // 掴める所でだけ掴める手を出す。Camera 視点で Grab を名乗ると
                // 「触れそうで触れない」になる(Q0)。
                if user_view {
                    cx.set_cursor(MouseCursor::Grab);
                }
            }
            Hit::FingerHoverOut(_) => cx.set_cursor(MouseCursor::Default),
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                if user_view {
                    self.view_drag = Some(StageViewDrag {
                        last_x: fe.abs.x,
                        last_y: fe.abs.y,
                    });
                    cx.set_cursor(MouseCursor::Grabbing);
                }
            }
            Hit::FingerMove(fe) => {
                if let Some(drag) = self.view_drag.as_mut() {
                    let delta = [fe.abs.x - drag.last_x, fe.abs.y - drag.last_y];
                    drag.last_x = fe.abs.x;
                    drag.last_y = fe.abs.y;
                    self.pan_by_pixels(delta, rect);
                }
            }
            Hit::FingerUp(fe) => {
                // 板の外で離したら掴む手を残さない(そこはもう掴める場所ではない)。
                if self.view_drag.take().is_some() {
                    cx.set_cursor(if user_view && fe.is_over {
                        MouseCursor::Grab
                    } else {
                        MouseCursor::Default
                    });
                }
            }
            Hit::FingerScroll(fs) => {
                if user_view {
                    self.apply_scroll(GestureSample::from_makepad_scroll(&fs), rect);
                }
            }
            Hit::FingerGesture(fe) => {
                if user_view {
                    self.apply_scroll(GestureSample::from_makepad_gesture(&fe), rect);
                }
            }
            _ => {}
        }
        self.view_camera != before
    }

    /// スクロール/ピンチ1サンプル。向きの約束は**1つだけ**: 指を上へ動かす入力
    /// (= 負の縦成分)は「画を手前へ引く」— 中身が下りてきて、ズームでは拡大になる。
    /// パンとズームで別々に符号を決めない。
    fn apply_scroll(&mut self, sample: GestureSample, rect: Rect) {
        let centroid = sample.centroid;
        match self
            .view_scroll
            .update_sample(sample, self.view_zoom_octave_px)
        {
            Some(StageScroll::Pan(scroll)) => {
                // ドラッグ(掴んで引きずる)とスクロール(面を送る)は互いに逆向きの
                // 慣用なので、同じ `pan_by_pixels` へ符号を反転して渡す。
                self.pan_by_pixels([-scroll[0], -scroll[1]], rect);
            }
            Some(StageScroll::Zoom(factor)) => self.zoom_by(factor, centroid, rect),
            None => {}
        }
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
        // 視点が変わったか。タブ切替と観測カメラの移動はどちらも「Stage に出る絵が
        // 変わった」に畳まれるので、1本の action で外へ出す。
        let mut view_changed = false;
        if let Some(index) = self
            .view
            .radio_button_set(cx, ids_array!(stage_tabs.camera_tab, stage_tabs.user_tab))
            .selected(cx, &actions)
        {
            // 帯の視点名はタブ選択の結果。splash の `on_click` は RadioButton に無いので
            // (Browser の TabIcon で踏んだのと同じ)、識別は Rust 側が書く。
            // 書くのは**状態**で、文字はこの後の `project_band` が作る
            if self.stage_view != index {
                view_changed = true;
            }
            self.stage_view = index;
            band_dirty = true;
        }

        // ⌂ = home/auto 復帰。視点という1つの状態を初期(fit・パン無し)へ戻すだけで、
        // 文字には触らない。**倍率だけでなくパンも戻す** — 見失った時に1押しで
        // 帰れる場所が「画角は元だが画面外」だと、帰れたことにならない
        if self
            .view
            .widget(cx, ids!(stage_band.home_zoom))
            .as_button()
            .clicked(&actions)
        {
            if self.view_camera != StageViewCamera::default() {
                self.view_camera = StageViewCamera::default();
                view_changed = true;
            }
            band_dirty = true;
        }

        // 見回し(User View のみ)。タブ・⌂ の後に置く — 同じイベントで両方が
        // 動くことは無いが、状態の持ち主が1つである順序を読める形にしておく
        if self.handle_view_gestures(cx, event) {
            view_changed = true;
            band_dirty = true;
        }

        // live edit は宣言側の文字を既定へ戻す。状態は `#[rust]` なので残っており、
        // 投影し直さないと帯だけが古い値を名乗る
        if matches!(event, Event::LiveEdit) {
            band_dirty = true;
        }

        if view_changed {
            cx.widget_action(self.uid, StageChromeAction::ViewChanged);
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
