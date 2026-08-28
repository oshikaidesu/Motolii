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
use motolii_store::LayerId;
// Stage ギズモ(S20)。借用先(supervisor 確定) — renderer 非依存、viewport 座標の
// 頂点列を返すだけ。`gmath` は glam の再輸出(transform-gizmo 自身の glam、workspace の
// glam とは別の写しなので、境界を跨ぐ時は薄い helper 関数だけを通す — 新しい数学型は作らない)。
// `TransformPivotPoint` だけ prelude に無いので `config` から直接引く(確認済み: 出典 =
// github.com/urholaukkarinen/transform-gizmo 0.5.0 タグの `crates/transform-gizmo/src/prelude.rs`)。
use transform_gizmo::config::TransformPivotPoint;
use transform_gizmo::math as gmath;
use transform_gizmo::{Gizmo, GizmoConfig, GizmoInteraction, GizmoMode, GizmoOrientation, GizmoResult};

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

    // Stage ギズモ(S20)。`transform-gizmo` の `Gizmo::draw()` が返す頂点列
    // (viewport 座標、三角形の羅列)を敷くための最小 shader。「既にやっている物」の
    // 継ぎ目 — `widgets/src/chart.rs` の `DrawChartSegment`(このリポジトリが引く
    // makepad fork に実在する、instance に生の座標を持たせて pixel shader で判定する型)
    // と同じ手口。三角形内外判定は符号付き面積(barycentric の符号)— 巻き順を問わない。
    set_type_default() do #(DrawGizmoTriangle::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let p = self.pos * self.rect_size
            let a = self.tri_v0
            let b = self.tri_v1
            let c = self.tri_v2
            let d1 = (p.x - b.x) * (a.y - b.y) - (a.x - b.x) * (p.y - b.y)
            let d2 = (p.x - c.x) * (b.y - c.y) - (b.x - c.x) * (p.y - c.y)
            let d3 = (p.x - a.x) * (c.y - a.y) - (c.x - a.x) * (p.y - a.y)
            let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0
            let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0
            if has_neg && has_pos {
                return #0000
            }
            return Pal.premul(self.tri_color)
        }
    }

    // ギズモの overlay(UI chrome — scene 内容ではない、共有 Surface へは触らない)。
    // 頂点は StageChrome が絶対 window 画素で計算し、ここは draw_abs で敷くだけ。
    mod.widgets.StageGizmoOverlayBase = #(StageGizmoOverlay::register_widget(vm))
    mod.widgets.StageGizmoOverlay = set_type_default() do mod.widgets.StageGizmoOverlayBase{
        width: Fill
        height: Fill
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
                // ギズモ overlay(S20)。同じ Overlay flow の最上段 — comp の絵の上に敷く。
                stage_gizmo := mod.widgets.StageGizmoOverlay{}
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
        failures_edge := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.face.down}
        // A05 隔離の読み出し口の常設帯(発注 S3、`Engine::layer_failures`)。**空なら
        // 帯ごと隠れる**(`set_failures` が `visible` を倒す — invisible な View は
        // 層の footprint を持たないので、隠れると帯そのものが無かったことになる)。
        // hover やメニューの奥に隠さない(Ableton 可視性原理) — 常設の帯で言う。
        failures_band := SolidView{width: Fill height: mod.tokens.size.status flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel visible: false
            failures_text := InkLabel{text: "" width: Fill draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_code{font_size: mod.tokens.text.xs}}
        }
    }
}

/// ギズモの三角形1枚ぶんの shader(`transform_gizmo::GizmoDrawData` の頂点列を
/// 敷くための最小型)。`v0`/`v1`/`v2` は **描く先の `draw_abs` rect に対する相対座標**
/// (`widgets/src/chart.rs::DrawChartSegment` の `seg_a`/`seg_b` と同じ約束 — `DrawQuad`
/// の pixel shader は `self.pos * self.rect_size` で rect ローカルの画素位置しか
/// 持たないため)。
#[derive(Script, ScriptHook, Debug)]
#[repr(C)]
pub struct DrawGizmoTriangle {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    pub tri_v0: Vec2f,
    #[live]
    pub tri_v1: Vec2f,
    #[live]
    pub tri_v2: Vec2f,
    #[live]
    pub tri_color: Vec4f,
}

/// 描く直前の形に均した1三角形(絶対 window 画素の外接矩形 + rect ローカルの頂点)。
/// `GizmoDrawData` の色は頂点ごとだが、`DrawGizmoTriangle` は面1枚に1色しか持てない
/// ので、頂点0の色を面の色として使う(近似 — 皮の詰めは後回し、EVIDENCE_GAP 参照)。
#[derive(Clone, Copy, Debug)]
struct GizmoScreenTriangle {
    bounds: Rect,
    v0: Vec2f,
    v1: Vec2f,
    v2: Vec2f,
    color: Vec4f,
}

/// ギズモの overlay。UI chrome であって scene 内容ではない — 共有 Surface へは触らず、
/// StageChrome が絶対 window 画素で計算した三角形を並べて敷くだけの葉ノード。
/// `widgets/src/perf_graph.rs::PerfGraph` と同型の最小 custom-draw widget
/// (`#[derive(Widget)]` が `WidgetNode` を作るので手で書かない)。
#[derive(Script, ScriptHook, Widget)]
pub struct StageGizmoOverlay {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[redraw]
    #[live]
    draw_tri: DrawGizmoTriangle,
    #[rust]
    triangles: Vec<GizmoScreenTriangle>,
}

impl StageGizmoOverlay {
    fn set_triangles(&mut self, cx: &mut Cx, triangles: Vec<GizmoScreenTriangle>) {
        self.triangles = triangles;
        self.redraw(cx);
    }
}

impl Widget for StageGizmoOverlay {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // 三角形はすべて絶対座標(`draw_abs`)で敷くので turtle の戻り矩形は使わないが、
        // 親レイアウトへの占有は必要(`PerfGraph::draw_walk` と同じ形)。
        let _ = cx.walk_turtle(walk);
        for tri in &self.triangles {
            self.draw_tri.tri_v0 = tri.v0;
            self.draw_tri.tri_v1 = tri.v1;
            self.draw_tri.tri_v2 = tri.v2;
            self.draw_tri.tri_color = tri.color;
            self.draw_tri.draw_abs(cx, tri.bounds);
        }
        DrawStep::done()
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

/// 選択レイヤーの world transform(ギズモの対象)。**main.rs が Document から
/// 解決して押し込む** — StageChrome は Document を持たない(module doc 冒頭)ので、
/// これが唯一の継ぎ目(`StageChrome::set_failures` と同じ形)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GizmoTarget {
    pub layer: LayerId,
    /// comp 空間(world、`ResolvedLayer.placement.transform` と同じ単位)の並進、画素。
    pub translation: [f32; 2],
    /// 度・時計回り(`property::ROTATION` と同じ約束)。
    pub rotation_degrees: f32,
    pub scale: [f32; 2],
}

/// ドラッグを離した結果、Document へ書くべき値。**触った成分だけ `Some`** —
/// 1回のドラッグは translate/rotate/scale のうち1種類のハンドルしか動かさない
/// (canon: 「離した時に `Intent::SetTrack` 1発」)。main.rs が Document を持つので、
/// 実際に書くのは main.rs の仕事 — StageChrome はここまでしか運ばない。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GizmoCommit {
    pub layer: LayerId,
    pub translation: Option<[f32; 2]>,
    pub rotation_degrees: Option<f32>,
    pub scale: Option<[f32; 2]>,
}

/// Stage 面が外へ出す唯一の意図。**視点が変わった**とだけ言い、絵を描き直すかどうかは
/// 継ぎ目(main.rs)が決める(timeline の `TimelineSurfaceAction` と同型)。
///
/// タブの切替も観測カメラの移動も同じ1本で運ぶ — どちらも「Stage に出る絵が変わった」
/// という同じ帰結しか持たないので、口を2本にすると呼び手が同じ処理を2回書くことになる。
// f32 を運ぶ variant(`StagePicked`)を足したので `Eq` は落とす(f32 は `Eq` を
// 持たない) — 既存の読み手は `matches!`/`PartialEq` しか使っていない。
#[derive(Clone, Debug, Default, PartialEq)]
pub enum StageChromeAction {
    #[default]
    None,
    /// 視点(タブ or 観測カメラ)が変わった。Stage の絵を描き直す必要がある。
    ViewChanged,
    /// ギズモのドラッグが確定した(release)。`take_gizmo_commit` で1回だけ取り出す —
    /// action 自体は「何かが確定した」の通知だけで、値は運ばない(`set_failures` と
    /// 同じ「通知は action・値は getter」の分離)。
    GizmoCommitted,
    /// 空きクリック(ギズモの上ではない、`comp` 板の上)。当たり判定は Document を
    /// 持つ main.rs の仕事(`stage_pick` 室) — ここは comp 空間の点を運ぶだけ。
    StagePicked {
        comp_point: [f32; 2],
        additive: bool,
    },
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
    // --- Stage ギズモ(S20) ---
    /// comp の寸法。**選択の有無によらず**毎フレーム main.rs から渡す —
    /// pick(TARGET5)は選択が無くても comp 空間の点を作れる必要があるため、
    /// `gizmo_target` にだけ乗せると空きクリックの入口が塞がる。
    #[rust]
    comp_dims: Option<(f32, f32)>,
    /// main.rs が押し込んだ「今どのレイヤーを、どこに描くか」。`set_stage_gizmo` の
    /// 唯一の書き手。
    #[rust]
    gizmo_target: Option<GizmoTarget>,
    /// `transform-gizmo` 本体。`Gizmo: Default` なので `#[rust]` フィールドとして持てる。
    #[rust]
    gizmo: Gizmo,
    /// ドラッグ中だけ true。**この間だけ**カメラの pan/zoom(`handle_view_gestures`)を
    /// 同じイベントで動かさない(TARGET6)。
    #[rust]
    gizmo_drag_active: bool,
    /// release で1回だけ積む書き込み予約。`take_gizmo_commit` が取り出すと空になる。
    #[rust]
    gizmo_pending_commit: Option<GizmoCommit>,
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

    /// A05 隔離の読み出し口の投影(発注 S3)。継ぎ目(main.rs)が frame を引いた
    /// 後に `Engine::layer_failures()` をそのまま渡す唯一の口。
    ///
    /// **`failures` が空なら帯ごと隠す**(裁定済みの置き場所)。非空なら engine が
    /// 溜めた文字列を**加工せず**列挙する — 意味を作らない、要約もしない。
    /// 件数が多い時だけ、読める分量に収めるため先頭数件 + 残り件数を足す
    /// (足すのは「あと何件」という数だけで、個々の文字列は変えない)。
    pub fn set_failures(&mut self, cx: &mut Cx, failures: &[String]) {
        const SHOWN_LIMIT: usize = 3;
        let band = self.view.widget(cx, ids!(failures_band));
        band.set_visible(cx, !failures.is_empty());
        if failures.is_empty() {
            return;
        }
        let mut text = failures
            .iter()
            .take(SHOWN_LIMIT)
            .cloned()
            .collect::<Vec<_>>()
            .join("  ·  ");
        let hidden = failures.len().saturating_sub(SHOWN_LIMIT);
        if hidden > 0 {
            text.push_str(&format!("  ·  +{hidden} more"));
        }
        self.view
            .widget(cx, ids!(failures_band.failures_text))
            .as_label()
            .set_text(cx, &text);
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

    // --- Stage ギズモ(S20) ---

    /// main.rs → StageChrome の唯一の継ぎ目(`set_failures` と同じ形)。`comp_dims`
    /// は選択の有無によらず毎回渡す — pick(TARGET5)は選択が無くても comp 空間の
    /// 点を作れる必要があるので、`gizmo_target` にだけ乗せると空きクリックの
    /// 入口が塞がる。
    pub fn set_stage_gizmo(
        &mut self,
        cx: &mut Cx,
        comp_dims: Option<(f32, f32)>,
        target: Option<GizmoTarget>,
    ) {
        self.comp_dims = comp_dims;
        // 選択が別レイヤーへ変わった/消えたら、進行中のドラッグは意味を失う
        // (transform-gizmo 側の drag 状態と食い違うより、素直に打ち切る方が安全)。
        if target.map(|t| t.layer) != self.gizmo_target.map(|t| t.layer) {
            self.gizmo_drag_active = false;
        }
        self.gizmo_target = target;
        self.refresh_gizmo(cx);
    }

    /// 直近の commit を1回だけ取り出す。main.rs が Document へ書いた後は空になる。
    pub fn take_gizmo_commit(&mut self) -> Option<GizmoCommit> {
        self.gizmo_pending_commit.take()
    }

    /// pan/zoom から成る正射影 view+proj(3D 数学の退化ケース、canon 冒頭)。
    /// Camera タブでは出力カメラを front が知らないので恒等を使う近似
    /// (EVIDENCE_GAP: Document のカメラに pan/zoom/roll があるとズレる)。
    ///
    /// 導出は新しい約束を作っていない — `comp_area` の「板いっぱいに comp が
    /// 映っている」前提のまま、`pan_by_pixels`/`zoom_by` が既に定義している
    /// 画面⇄comp-fraction の写像を clip 空間へ写しただけ。screen 側の実寸は
    /// `GizmoConfig::viewport` が別に持つので、ここは正規化した比だけを組む。
    fn gizmo_camera_matrices(&self, comp_width: f32, comp_height: f32) -> (gmath::DMat4, gmath::DMat4) {
        let cam = if self.is_user_view() {
            self.view_camera
        } else {
            StageViewCamera::default()
        };
        let cw = comp_width.max(1.0) as f64;
        let ch = comp_height.max(1.0) as f64;
        let zoom = cam.zoom.max(1e-6);
        let scale_x = 2.0 * zoom / cw;
        let scale_y = -2.0 * zoom / ch;
        let offset_x = -zoom * (1.0 + 2.0 * cam.pan_fraction[0]);
        let offset_y = zoom * (1.0 + 2.0 * cam.pan_fraction[1]);
        let projection = gmath::DMat4::from_cols(
            gmath::DVec4::new(scale_x, 0.0, 0.0, 0.0),
            gmath::DVec4::new(0.0, scale_y, 0.0, 0.0),
            gmath::DVec4::new(0.0, 0.0, 1.0, 0.0),
            gmath::DVec4::new(offset_x, offset_y, 0.0, 1.0),
        );
        (gmath::DMat4::IDENTITY, projection)
    }

    /// 画面画素 → comp 空間の点(pick 室の入力作り)。`gizmo_camera_matrices` と
    /// 対になる逆写像 — `zoom_by` が既に定義した `u`/`under_pointer` の関係をそのまま使う。
    fn screen_to_comp_point(
        &self,
        screen: [f64; 2],
        rect: Rect,
        comp_width: f32,
        comp_height: f32,
    ) -> [f32; 2] {
        let cam = if self.is_user_view() {
            self.view_camera
        } else {
            StageViewCamera::default()
        };
        let zoom = cam.zoom.max(1e-6);
        let u = [
            (screen[0] - rect.pos.x - rect.size.x * 0.5) / rect.size.x,
            (screen[1] - rect.pos.y - rect.size.y * 0.5) / rect.size.y,
        ];
        let fx = cam.pan_fraction[0] + u[0] / zoom;
        let fy = cam.pan_fraction[1] + u[1] / zoom;
        [
            ((fx + 0.5) * comp_width as f64) as f32,
            ((fy + 0.5) * comp_height as f64) as f32,
        ]
    }

    fn gizmo_config(&self, rect: Rect, comp_width: f32, comp_height: f32) -> GizmoConfig {
        let viewport = gmath::Rect::from_min_size(
            gmath::Pos2::new(rect.pos.x as f32, rect.pos.y as f32),
            gmath::Vec2::new(rect.size.x as f32, rect.size.y as f32),
        );
        let (view, projection) = self.gizmo_camera_matrices(comp_width, comp_height);
        GizmoConfig {
            view_matrix: view.into(),
            projection_matrix: projection.into(),
            viewport,
            // z=0 の退化ケース(canon 冒頭)。Z 軸に沿う/View 相対のモードは平坦な
            // 世界では意味を持たないので外す — TranslateXY と RotateZ が AE の
            // 「2D レイヤーを掴む」に対応する(view=identity で world Z がそのまま
            // カメラの前方軸になるので、RotateZ が画面内の回転になる)。
            modes: GizmoMode::TranslateX
                | GizmoMode::TranslateY
                | GizmoMode::TranslateXY
                | GizmoMode::RotateZ
                | GizmoMode::ScaleX
                | GizmoMode::ScaleY
                | GizmoMode::ScaleUniform,
            orientation: GizmoOrientation::Global,
            pivot_point: TransformPivotPoint::MedianPoint,
            ..Default::default()
        }
    }

    fn idle_interaction() -> GizmoInteraction {
        GizmoInteraction {
            cursor_pos: (0.0, 0.0),
            hovered: false,
            drag_started: false,
            dragging: false,
        }
    }

    fn gizmo_transform(target: GizmoTarget) -> gmath::Transform {
        gmath::Transform::from_scale_rotation_translation(
            gmath::DVec3::new(target.scale[0] as f64, target.scale[1] as f64, 1.0),
            gmath::DQuat::from_rotation_z((target.rotation_degrees as f64).to_radians()),
            gmath::DVec3::new(target.translation[0] as f64, target.translation[1] as f64, 0.0),
        )
    }

    /// `Gizmo::update` が返した絶対 Transform → `GizmoTarget` の形へ戻す。
    /// `gizmo_config` は RotateZ しか許可していないので、抽出した回転軸は
    /// 常にほぼ ±Z のはず — 符号だけ見て度数へ畳む。
    fn target_from_gizmo_transform(base: GizmoTarget, t: gmath::Transform) -> GizmoTarget {
        let translation = gmath::DVec3::from(t.translation);
        let scale = gmath::DVec3::from(t.scale);
        let rotation = gmath::DQuat::from(t.rotation);
        let (axis, angle) = rotation.to_axis_angle();
        let signed_angle = if axis.z < 0.0 { -angle } else { angle };
        GizmoTarget {
            layer: base.layer,
            translation: [translation.x as f32, translation.y as f32],
            rotation_degrees: signed_angle.to_degrees() as f32,
            scale: [scale.x as f32, scale.y as f32],
        }
    }

    /// `GizmoResult` の種類だけを見て、触った成分だけ運ぶ `GizmoCommit` を作る
    /// (canon: 「離した時に `Intent::SetTrack` 1発」)。
    fn gizmo_commit_for(layer: LayerId, live: GizmoTarget, result: GizmoResult) -> Option<GizmoCommit> {
        Some(match result {
            GizmoResult::Translation { .. } => GizmoCommit {
                layer,
                translation: Some(live.translation),
                rotation_degrees: None,
                scale: None,
            },
            GizmoResult::Rotation { .. } => GizmoCommit {
                layer,
                translation: None,
                rotation_degrees: Some(live.rotation_degrees),
                scale: None,
            },
            GizmoResult::Scale { .. } => GizmoCommit {
                layer,
                translation: None,
                rotation_degrees: None,
                scale: Some(live.scale),
            },
            // Arcball は `gizmo_config` の modes に入れていないので理論上出ない。
            // 出た場合は書かない(未対応の枝を黙って握りつぶすのではなく、
            // そもそも到達しない枝として明示する)。
            GizmoResult::Arcball { .. } => return None,
        })
    }

    fn set_overlay_triangles(&mut self, cx: &mut Cx, triangles: Vec<GizmoScreenTriangle>) {
        if let Some(mut overlay) = self
            .view
            .widget(cx, ids!(stage_void.comp.stage_gizmo))
            .borrow_mut::<StageGizmoOverlay>()
        {
            overlay.set_triangles(cx, triangles);
        }
    }

    /// `Gizmo::draw()` の頂点(viewport 座標、絶対 window 画素)を overlay の
    /// 三角形列へ均す。色は頂点0のものを面色として使う(`GizmoScreenTriangle` doc 参照)。
    fn push_gizmo_draw(&mut self, cx: &mut Cx) {
        let data = self.gizmo.draw();
        let mut triangles = Vec::with_capacity(data.indices.len() / 3);
        for face in data.indices.chunks_exact(3) {
            let (i0, i1, i2) = (face[0] as usize, face[1] as usize, face[2] as usize);
            let (Some(p0), Some(p1), Some(p2), Some(c0)) = (
                data.vertices.get(i0),
                data.vertices.get(i1),
                data.vertices.get(i2),
                data.colors.get(i0),
            ) else {
                continue;
            };
            let min_x = p0[0].min(p1[0]).min(p2[0]) - 1.0;
            let min_y = p0[1].min(p1[1]).min(p2[1]) - 1.0;
            let max_x = p0[0].max(p1[0]).max(p2[0]) + 1.0;
            let max_y = p0[1].max(p1[1]).max(p2[1]) + 1.0;
            let bounds = Rect {
                pos: dvec2(min_x as f64, min_y as f64),
                size: dvec2((max_x - min_x) as f64, (max_y - min_y) as f64),
            };
            triangles.push(GizmoScreenTriangle {
                bounds,
                v0: vec2(p0[0] - min_x, p0[1] - min_y),
                v1: vec2(p1[0] - min_x, p1[1] - min_y),
                v2: vec2(p2[0] - min_x, p2[1] - min_y),
                color: vec4(c0[0], c0[1], c0[2], c0[3]),
            });
        }
        self.set_overlay_triangles(cx, triangles);
    }

    /// ギズモの内部状態(位置・回転・拡大)を現在の target/camera へ合わせ、絵を
    /// 引き直す。**ドラッグ中は何もしない** — 位置の追随は `apply_gizmo_interaction`
    /// の専任で、ここが横から `update()` を呼ぶと transform-gizmo 内部の
    /// `active_subgizmo_id` が idle 扱いで消えてしまう(出典: crate 本体
    /// `Gizmo::update` — `interaction.dragging == false` の間 active subgizmo を
    /// 終える分岐がある)。
    fn refresh_gizmo(&mut self, cx: &mut Cx) {
        if self.gizmo_drag_active {
            return;
        }
        let (Some((comp_width, comp_height)), Some(target)) = (self.comp_dims, self.gizmo_target) else {
            self.set_overlay_triangles(cx, Vec::new());
            return;
        };
        let area = self.comp_area(cx);
        if area == Area::Empty {
            return;
        }
        let rect = area.rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        // `self.gizmo.update_config(self.gizmo_config(...))` は書けない —
        // `self.gizmo_config` は `&self` 全体を借りるので、`self.gizmo` への
        // `&mut` と同時に取れない(借用チェッカ)。先に値を作ってから渡す。
        let config = self.gizmo_config(rect, comp_width, comp_height);
        self.gizmo.update_config(config);
        let targets = [Self::gizmo_transform(target)];
        let _ = self.gizmo.update(Self::idle_interaction(), &targets);
        self.push_gizmo_draw(cx);
    }

    /// `update()` を1回呼び、結果を見た目だけ追随させる。**Document へは書かない**
    /// (canon: ドラッグ中はギズモ自身が追随して見せるだけ)。`releasing` の時だけ
    /// commit を積む — 毎フレーム SetTrack は禁止。
    fn apply_gizmo_interaction(
        &mut self,
        cx: &mut Cx,
        base: GizmoTarget,
        interaction: GizmoInteraction,
        releasing: bool,
    ) {
        let targets = [Self::gizmo_transform(base)];
        if let Some((result, updated)) = self.gizmo.update(interaction, &targets) {
            if let Some(t) = updated.first().copied() {
                let live = Self::target_from_gizmo_transform(base, t);
                if releasing {
                    self.gizmo_pending_commit = Self::gizmo_commit_for(base.layer, live, result);
                    if self.gizmo_pending_commit.is_some() {
                        cx.widget_action(self.uid, StageChromeAction::GizmoCommitted);
                    }
                }
            }
        }
        self.push_gizmo_draw(cx);
    }

    /// ギズモの掴み + 空きクリックの pick(TARGET5/6)。**戻り値 = このイベントを
    /// ギズモが消費したか** — true ならカメラの pan/zoom は同じイベントで動かさない
    /// (`Widget::handle_event` 側の優先順位)。pick 自体は戻り値に含めない — User
    /// View での空きクリックがパンの開始を兼ねるのは妨げない(選択が変わるのと
    /// 見回しが始まるのは独立な帰結なので、同じ指の1回のイベントが両方を運んでよい)。
    fn handle_gizmo_and_pick(&mut self, cx: &mut Cx, event: &Event) -> bool {
        let area = self.comp_area(cx);
        if area == Area::Empty {
            return false;
        }
        let rect = area.rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return false;
        }

        match event.hits(cx, area) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                let cursor = (fe.abs.x as f32, fe.abs.y as f32);
                let gizmo_ready = match (self.comp_dims, self.gizmo_target) {
                    (Some((cw, ch)), Some(_)) => {
                        // 借用の順序に注意(`refresh_gizmo` と同じ理由): 先に値を
                        // 作ってから `self.gizmo` へ渡す。
                        let config = self.gizmo_config(rect, cw, ch);
                        self.gizmo.update_config(config);
                        true
                    }
                    _ => false,
                };
                if gizmo_ready && self.gizmo.pick_preview(cursor) {
                    self.gizmo_drag_active = true;
                    let base = self.gizmo_target.expect("gizmo_ready implies Some");
                    self.apply_gizmo_interaction(
                        cx,
                        base,
                        GizmoInteraction {
                            cursor_pos: cursor,
                            hovered: true,
                            drag_started: true,
                            dragging: true,
                        },
                        false,
                    );
                    true
                } else {
                    // 空きクリック(ギズモの上ではない) — pick 室へ、値だけ運ぶ。
                    if let Some((comp_width, comp_height)) = self.comp_dims {
                        let comp_point = self.screen_to_comp_point(
                            [fe.abs.x, fe.abs.y],
                            rect,
                            comp_width,
                            comp_height,
                        );
                        cx.widget_action(
                            self.uid,
                            StageChromeAction::StagePicked {
                                comp_point,
                                additive: fe.modifiers.shift,
                            },
                        );
                    }
                    false
                }
            }
            Hit::FingerMove(fe) if self.gizmo_drag_active => {
                if let Some(base) = self.gizmo_target {
                    self.apply_gizmo_interaction(
                        cx,
                        base,
                        GizmoInteraction {
                            cursor_pos: (fe.abs.x as f32, fe.abs.y as f32),
                            hovered: true,
                            drag_started: false,
                            dragging: true,
                        },
                        false,
                    );
                }
                true
            }
            Hit::FingerUp(fe) if self.gizmo_drag_active => {
                self.gizmo_drag_active = false;
                if let Some(base) = self.gizmo_target {
                    self.apply_gizmo_interaction(
                        cx,
                        base,
                        GizmoInteraction {
                            cursor_pos: (fe.abs.x as f32, fe.abs.y as f32),
                            hovered: fe.is_over,
                            drag_started: false,
                            // release の1回は dragging: true で渡す — Gizmo::update は
                            // dragging: false の呼び出しでは active subgizmo を終えて
                            // None を返す(結果を計算する前に)。ここを false にすると
                            // 最後の commit だけが常に握りつぶされる。
                            dragging: true,
                        },
                        true,
                    );
                }
                true
            }
            _ => false,
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

        // ギズモの掴み + 空きクリックの pick(S20)。カメラより先に見る — ギズモが
        // 掴んでいる間は同じイベントでカメラの pan/zoom を動かさない(TARGET6)。
        let gizmo_claimed = self.handle_gizmo_and_pick(cx, event);

        // 見回し(User View のみ)。タブ・⌂ の後に置く — 同じイベントで両方が
        // 動くことは無いが、状態の持ち主が1つである順序を読める形にしておく
        if !gizmo_claimed && self.handle_view_gestures(cx, event) {
            view_changed = true;
            band_dirty = true;
            // カメラが動いたのでギズモの画面位置も引き直す(TARGET2: 追随)。
            self.refresh_gizmo(cx);
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
