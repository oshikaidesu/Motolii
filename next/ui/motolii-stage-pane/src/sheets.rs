//! Stage 方眼シート+セーフマージン(B22 第1切片、発注 2026-08-22)。
//!
//! 視覚正本 = `next/reference/mocks/stage-semantics.html` v5 の方眼シート3種
//! (方眼 `.grid9` / 三分割 `.thirds` / 黄金比 `.phi`)+ Safe areas
//! (`.safe.a` inset 5% / `.safe.t` inset 10%)。**モック優先で転写** — 値は
//! ここで発明しない(比率は全部 mock の CSS 実測、下記の各定数 doc に出典)。
//!
//! ## 意味(map B22)
//!
//! - 方眼 = map 1119「Show or hide grid」/ 1469「Show Grid」
//! - 三分割 = map 1122「Show or hide proportional grid」(構図の比例グリッド)
//! - 黄金比 = mock v5 宣言(map に単独行は無い — 三分割と同じ「構図ガイド」族。
//!   UI 寸法の黄金比棄却(裁定164)とは別物=**ユーザーの道具**、mock 注記どおり)
//! - Safe areas = map 1124「Show or hide safe zones」/ 1465「Safe Area」
//!
//! ガイド(ユーザー定義ガイド線)・定規・スナップ・ロックは**次切片**
//! (map 1119〜1474 のガイド系 — ドラッグ配置と磁石エンジンが要るため別束)。
//!
//! ## 家(結線は次波)
//!
//! 表示トグルの家は Viewer(状態帯 or View メニュー — 未決、mock「入口は帯の
//! アイコン(View 系)」)。この pane は**描画と [`SheetMessage`] まで** —
//! トグル状態([`SheetToggles`])は shell が Session 状態として持ち、
//! [`SheetOverlay`] へ毎フレーム渡す(`StageOverlay` と同じ「不変な投影を
//! canvas::Program に渡す」形)。
//!
//! ## 座標系(シートは**出力フレーム**の上に住む)
//!
//! セーフマージンは「書き出される絵」の 5%/10% であって観測視点の 5% ではない —
//! 方眼/三分割/黄金比も同じ(構図は出力に対して組む)。したがって:
//!
//! - カメラを通して見ている間(observation `None`): 表示画像=出力キャンバス
//!   そのものなので、写像は letterbox だけ(恒等カメラ)。
//! - User View(observation `Some`): 出力フレームが世界のどこに写っているかは
//!   [`crate::render_camera_frame_corners_on_screen`] と同じ合成
//!   (`observation_affine ∘ render_affine⁻¹`)— フレーム枠 overlay と
//!   シートが**同じ矩形に住む**ことを同じ式で保証する。
//!
//! **GZ FINDING(bounds の pane 原点ずれ)を踏まない**: letterbox は
//! [`crate::gizmo::letterbox_screen_from_comp`](gizmo 側の原点正規化と同じ処置)
//! を使う — `draw` の `Frame` はローカル座標なので、window 絶対の `bounds.x/y`
//! を混ぜない。
//!
//! ## 色(裁定179: グリッド線は装飾でなく道具)
//!
//! 全シート = ink 最弱段([`Ink::Muted`])+ hairline(`border_width`)。
//! 方眼だけさらに減量(mock の alpha 比 `.10/.22` を転写 — 方眼は面で効くので
//! 線1本は三分割より薄い)。新色ゼロ(裁定142)— alpha の減量だけで段を作る。
//!
//! ## テスト方針(lib.rs / gizmo.rs と同じ)
//!
//! 幾何は全部純関数([`grid_lines`]・[`fraction_lines`]・[`safe_rect`]・
//! [`sheet_screen_from_frame`])— `SheetOverlay` は委譲するだけの薄い翻訳層。

use glam::{Affine2, Vec2};
use iced::widget::canvas;
use iced::{mouse, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_tokens_rs::{Colors, Dimensions, Ink};

use crate::gizmo::{checked_inverse, letterbox_screen_from_comp};
use crate::observation_as_resolved;

// ---------------------------------------------------------------------------
// mock 実測の比率(stage-semantics.html v5 — ここで発明しない)
// ---------------------------------------------------------------------------

/// 方眼の分割数(縦)。mock `.grid9` = 45px セルを 405px 高の comp に敷く
/// (405/45 = 9)— セルは正方形(横も同じ 45px ピッチ)なので、セル寸は
/// 「comp 高さの 1/9」だけで決まる。
/// 三分割線の位置(両軸共通)。mock `.thirds` の 33.33% / 66.67%。
pub const THIRDS_FRACTIONS: [f32; 2] = [1.0 / 3.0, 2.0 / 3.0];

/// 黄金比線の位置(両軸共通)。mock `.phi` の 38.2% / 61.8%。
pub const PHI_FRACTIONS: [f32; 2] = [0.382, 0.618];

/// Action safe の内接率。mock `.safe.a{inset:5%}`(放送セーフの慣習値)。
/// Title safe の内接率。mock `.safe.t{inset:10%}`。
/// 方眼線の ink 減量率。mock の alpha 比(`.grid9` = .10、`.sheet i` = .22)を
/// 転写 — 新色ではなく既存 [`Ink::Muted`] の alpha を掛けるだけ(裁定142)。

/// セーフ枠の破線の1セグメント長(実線・空白とも)。mock は CSS `dashed`
/// (実装依存の既定ピッチ)なので専用寸は起こさず、既存段 `spacing_s` を借用
/// (独自の中間値を発明しない — `gizmo_rotate_offset` と同じ流儀)。
fn safe_dash_segment(dims: &Dimensions) -> f32 {
    dims.theme().space.s
}

// ---------------------------------------------------------------------------
// トグルの語彙と Message(結線は次波 — shell 側 write-set 不触のため
// `GizmoDrag` と同じ「独立 message 型」の形)
// ---------------------------------------------------------------------------

/// 方眼シート束の1枚。mock v5 のチェックボックス4つと1:1。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sheet {
    /// 方眼(map 1119/1469)。
    Grid,
    /// 三分割(map 1122)。
    Thirds,
    /// 黄金比(mock v5 宣言 — 構図ガイドとしての φ、裁定164 の魔法数棄却とは別物)。
    GoldenRatio,
    /// Safe areas(action 5% + title 10% を1トグルで対に出す — mock と同じ束)。
    SafeMargins,
}

/// 表示トグルの Message。**発火元は Viewer(状態帯 or View メニュー、結線は
/// 次波)** — [`crate::Message`] へ variant を足すと shell 側の exhaustive match
/// が壊れる(write-set 外)ため、`GizmoDrag` と同じ独立型。root 側で
/// `.map(...)` して畳む。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetMessage {
    Toggle(Sheet),
}

/// どのシートが出ているか。**Document ではない**(表示専用 — 市松トグルと
/// 同格の「視界状態」、κ台帳 a型)。shell が Session 状態として持つ。
/// 既定は全部 off(mock のチェックボックス初期値どおり — シートは道具であって
/// 常設 chrome ではない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SheetToggles {
    pub grid: bool,
    pub thirds: bool,
    pub golden_ratio: bool,
    pub safe_margins: bool,
}

impl SheetToggles {
    /// 1枚だけ反転した新しい値(純関数 — shell 側の `update` はこれを呼ぶだけ)。
    pub fn toggled(self, sheet: Sheet) -> Self {
        let mut next = self;
        match sheet {
            Sheet::Grid => next.grid = !next.grid,
            Sheet::Thirds => next.thirds = !next.thirds,
            Sheet::GoldenRatio => next.golden_ratio = !next.golden_ratio,
            Sheet::SafeMargins => next.safe_margins = !next.safe_margins,
        }
        next
    }

    /// [`SheetMessage`] の適用(結線用の読み口 — match を shell に書かせない)。
    pub fn apply(self, message: SheetMessage) -> Self {
        match message {
            SheetMessage::Toggle(sheet) => self.toggled(sheet),
        }
    }

    /// 1枚でも出ているか(全 off なら overlay は何も描かない — 死に chrome 回避)。
    pub fn any(self) -> bool {
        self.grid || self.thirds || self.golden_ratio || self.safe_margins
    }
}

// ---------------------------------------------------------------------------
// 幾何(出力フレーム空間 = comp px の純関数)
// ---------------------------------------------------------------------------

/// フレーム空間の線分1本(comp px)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetLine {
    pub from: [f32; 2],
    pub to: [f32; 2],
}

/// 方眼の内側の線(縁と重なる 0/width/height 上の線は描かない — comp 枠線と
/// 二重になるだけ)。セル = 高さの 1/9([`GRID_DIVISIONS`])の正方形。
/// comp が退化していれば空。
pub fn grid_lines(comp: CompSpec) -> Vec<SheetLine> {
    grid_lines_with_divisions(comp, Dimensions::default().components.stage.grid_divisions)
}

fn grid_lines_with_divisions(comp: CompSpec, divisions: u32) -> Vec<SheetLine> {
    let (w, h) = (comp.width as f32, comp.height as f32);
    if comp.width == 0 || comp.height == 0 {
        return Vec::new();
    }
    let cell = h / divisions.max(1) as f32;
    let mut lines = Vec::new();
    // 縦線: x = cell, 2*cell, … < w(右縁ちょうどは縁扱いで描かない)。
    let mut x = cell;
    while x < w - 0.5 {
        lines.push(SheetLine {
            from: [x, 0.0],
            to: [x, h],
        });
        x += cell;
    }
    // 横線: y = cell, … < h。
    let mut y = cell;
    while y < h - 0.5 {
        lines.push(SheetLine {
            from: [0.0, y],
            to: [w, y],
        });
        y += cell;
    }
    lines
}

/// 比率線2対(縦2+横2)。三分割([`THIRDS_FRACTIONS`])と黄金比
/// ([`PHI_FRACTIONS`])が共用する形(mock の `.thirds`/`.phi` が同じ
/// マークアップなのと同じ)。
pub fn fraction_lines(comp: CompSpec, fractions: [f32; 2]) -> Vec<SheetLine> {
    let (w, h) = (comp.width as f32, comp.height as f32);
    let mut lines = Vec::with_capacity(4);
    for fraction in fractions {
        lines.push(SheetLine {
            from: [w * fraction, 0.0],
            to: [w * fraction, h],
        });
    }
    for fraction in fractions {
        lines.push(SheetLine {
            from: [0.0, h * fraction],
            to: [w, h * fraction],
        });
    }
    lines
}

/// セーフ枠1つ(comp px)。`inset` = 各辺の内接率(mock の CSS `inset` と同義)。
pub fn safe_rect(comp: CompSpec, inset: f32) -> Rectangle {
    let (w, h) = (comp.width as f32, comp.height as f32);
    Rectangle {
        x: w * inset,
        y: h * inset,
        width: w * (1.0 - 2.0 * inset),
        height: h * (1.0 - 2.0 * inset),
    }
}

/// 出力フレーム空間(comp px) → screen(bounds **ローカル** — GZ FINDING の
/// 原点正規化済み)のアフィン。モジュール冒頭 doc「座標系」:
///
/// - observation `None`: letterbox だけ(表示画像=出力キャンバスそのもの)。
/// - observation `Some`: `letterbox ∘ observation_affine ∘ render_affine⁻¹` —
///   [`crate::render_camera_frame_corners_on_screen`] と同じ合成(フレーム枠
///   overlay と同じ矩形に住む)。render camera が退化していて逆行列が立たない
///   なら `None`(描かない)。
pub fn sheet_screen_from_frame(
    bounds: Rectangle,
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
) -> Option<Affine2> {
    let letterbox = letterbox_screen_from_comp(bounds, comp)?;
    let Some(observation) = observation else {
        return Some(letterbox);
    };
    let render_affine = camera_screen_from_world_z0(comp, render_camera);
    let observation_affine =
        camera_screen_from_world_z0(comp, observation_as_resolved(observation));
    // `checked_inverse`: det を先に見てから inverse を呼ぶ(退化行列への生
    // `.inverse()` は glam の自己アサートで panic する — `gizmo::checked_inverse`
    // doc 参照)。この製品での唯一の正解の形、新しい流儀を発明しない。
    let render_inverse = checked_inverse(render_affine)?;
    let composite = observation_affine * render_inverse;
    if !composite.is_finite() {
        return None;
    }
    Some(letterbox * composite)
}

// ---------------------------------------------------------------------------
// canvas::Program(翻訳だけ — StageOverlay/GizmoOverlay と同じ規律)
// ---------------------------------------------------------------------------

/// 方眼シート束の overlay。**描画のみ・入力ゼロ**(mock `.sheet{pointer-events:
/// none}` の転写 — event は一切 capture せず下の層(ギズモ/観測カメラ)へ
/// 素通し)。`Shell::view` が毎フレーム組み直し、`stack!` で `StageOverlay` の
/// 上・`GizmoOverlay` の**下**に重ねる(構図ガイドの上でレイヤーを掴めること —
/// 結線は supervisor)。
#[derive(Clone, Copy)]
pub struct SheetOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    toggles: SheetToggles,
    dims: Dimensions,
    colors: Colors,
}

impl SheetOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        toggles: SheetToggles,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            toggles,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> iced::Element<'a, SheetMessage> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

/// フレーム空間の点を screen の [`Point`] へ。
fn to_screen(affine: Affine2, point: [f32; 2]) -> Point {
    let p = affine.transform_point2(Vec2::new(point[0], point[1]));
    Point::new(p.x, p.y)
}

/// 矩形をアフィン越しに閉路 path で描く(render camera の roll 下でも
/// 正しい平行四辺形になる — `Path::rectangle` は軸平行しか描けない)。
fn transformed_rect_path(affine: Affine2, rect: Rectangle) -> canvas::Path {
    let corners = [
        [rect.x, rect.y],
        [rect.x + rect.width, rect.y],
        [rect.x + rect.width, rect.y + rect.height],
        [rect.x, rect.y + rect.height],
    ];
    canvas::Path::new(|builder| {
        builder.move_to(to_screen(affine, corners[0]));
        for corner in &corners[1..] {
            builder.line_to(to_screen(affine, *corner));
        }
        builder.close();
    })
}

impl canvas::Program<SheetMessage> for SheetOverlay {
    type State = ();

    /// 入力ゼロ(mock `pointer-events:none` の転写)。トグルの発火元は Viewer
    /// (状態帯 or View メニュー、結線は次波)であってシート自身ではない。
    fn update(
        &self,
        _state: &mut (),
        _event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<SheetMessage>> {
        None
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        if !self.toggles.any() {
            return Vec::new();
        }
        let Some(affine) =
            sheet_screen_from_frame(bounds, self.comp, self.render_camera, self.observation)
        else {
            return Vec::new();
        };

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let ink = Ink::Muted.resolve(&self.colors);
        let hairline = canvas::Stroke::default()
            .with_color(ink)
            .with_width(self.dims.theme().stroke.hairline);

        let stroke_lines = |frame: &mut canvas::Frame, lines: &[SheetLine], stroke: &canvas::Stroke| {
            for line in lines {
                let path =
                    canvas::Path::line(to_screen(affine, line.from), to_screen(affine, line.to));
                frame.stroke(&path, stroke.clone());
            }
        };

        // 方眼(最弱 — mock alpha 比の転写、モジュール冒頭 doc「色」)。
        if self.toggles.grid {
            let grid_stroke = canvas::Stroke::default()
                .with_color(iced::Color {
                    a: ink.a * self.dims.components.stage.grid_ink_factor,
                    ..ink
                })
                .with_width(self.dims.theme().stroke.hairline);
            stroke_lines(
                &mut frame,
                &grid_lines_with_divisions(
                    self.comp,
                    self.dims.components.stage.grid_divisions,
                ),
                &grid_stroke,
            );
        }

        // 三分割・黄金比(ink 最弱段そのまま)。
        if self.toggles.thirds {
            stroke_lines(&mut frame, &fraction_lines(self.comp, THIRDS_FRACTIONS), &hairline);
        }
        if self.toggles.golden_ratio {
            stroke_lines(&mut frame, &fraction_lines(self.comp, PHI_FRACTIONS), &hairline);
        }

        // Safe areas(action 5% + title 10% を対で・破線 — mock `.safe` の転写)。
        if self.toggles.safe_margins {
            let segment = safe_dash_segment(&self.dims);
            let segments = [segment, segment];
            let dashed = canvas::Stroke {
                line_dash: canvas::LineDash {
                    segments: &segments,
                    offset: 0,
                },
                ..hairline.clone()
            };
            for inset in [
                self.dims.components.stage.action_safe_inset,
                self.dims.components.stage.title_safe_inset,
            ] {
                let path = transformed_rect_path(affine, safe_rect(self.comp, inset));
                frame.stroke(&path, dashed.clone());
            }
        }

        vec![frame.into_geometry()]
    }

    /// 触れない物はカーソルでも「触れない」と言う(既定のまま — Q0 の逆方向:
    /// 触れそうな合図を出さない)。
    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    fn bounds() -> Rectangle {
        Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0))
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    // -----------------------------------------------------------------
    // 幾何(mock 転写の実測 — 未実行、波末一括)
    // -----------------------------------------------------------------

    /// 方眼: セル = 高さの 1/9(mock 405/45)の正方形。640x360 → セル40、
    /// 縦線15本(40..600)+横線8本(40..320)。縁(0/640/360)は描かない。
    #[test]
    fn grid_lines_form_square_cells_of_one_ninth_of_the_comp_height() {
        let lines = grid_lines(COMP);
        let verticals: Vec<_> = lines.iter().filter(|l| approx(l.from[0], l.to[0])).collect();
        let horizontals: Vec<_> = lines.iter().filter(|l| approx(l.from[1], l.to[1])).collect();
        assert_eq!(verticals.len(), 15, "640/40 = 16 列 → 内側の縦線は15本");
        assert_eq!(horizontals.len(), 8, "360/40 = 9 行 → 内側の横線は8本");
        assert!(approx(verticals[0].from[0], 40.0));
        assert!(approx(horizontals[0].from[1], 40.0));
        assert!(
            lines.iter().all(|l| l.from[0] > 0.5 || l.from[1] > 0.5),
            "縁の線は描かない"
        );
    }

    /// 退化した comp では方眼は空(除算も無限ループも起こさない)。
    #[test]
    fn grid_lines_are_empty_for_a_degenerate_comp() {
        assert!(grid_lines(CompSpec { width: 0, height: 0 }).is_empty());
    }

    /// 三分割と黄金比: 縦2+横2の比率線(mock `.thirds`/`.phi` の転写)。
    #[test]
    fn fraction_lines_sit_at_their_declared_fractions() {
        let thirds = fraction_lines(COMP, THIRDS_FRACTIONS);
        assert_eq!(thirds.len(), 4);
        assert!(approx(thirds[0].from[0], 640.0 / 3.0));
        assert!(approx(thirds[1].from[0], 640.0 * 2.0 / 3.0));
        assert!(approx(thirds[2].from[1], 120.0));
        assert!(approx(thirds[3].from[1], 240.0));

        let phi = fraction_lines(COMP, PHI_FRACTIONS);
        assert!(approx(phi[0].from[0], 640.0 * 0.382));
        assert!(approx(phi[3].from[1], 360.0 * 0.618));
    }

    /// Safe areas: action 5% / title 10% の内接矩形(mock `.safe` の転写)。
    #[test]
    fn safe_rects_inset_by_five_and_ten_percent() {
        let stage = Dimensions::default().components.stage;
        let action = safe_rect(COMP, stage.action_safe_inset);
        assert!(approx(action.x, 32.0) && approx(action.y, 18.0));
        assert!(approx(action.width, 576.0) && approx(action.height, 324.0));
        let title = safe_rect(COMP, stage.title_safe_inset);
        assert!(approx(title.x, 64.0) && approx(title.y, 36.0));
        assert!(approx(title.width, 512.0) && approx(title.height, 288.0));
    }

    // -----------------------------------------------------------------
    // screen 写像(GZ FINDING と観測カメラ — 未実行、波末一括)
    // -----------------------------------------------------------------

    /// カメラを通して見ている間は letterbox だけ(同アスペクトなら恒等)。
    #[test]
    fn through_camera_mapping_is_letterbox_only() {
        let affine = sheet_screen_from_frame(bounds(), COMP, ResolvedCamera::default(), None)
            .expect("letterbox が組めるはず");
        let p = affine.transform_point2(Vec2::new(40.0, 40.0));
        assert!(approx(p.x, 40.0) && approx(p.y, 40.0));
    }

    /// **GZ FINDING の柵**: pane が window 原点以外に居ても(bounds.x/y ≠ 0)、
    /// 写像はローカル座標で同一 — 原点ずれを踏まない。
    #[test]
    fn sheet_mapping_is_normalized_to_the_bounds_origin() {
        let at_origin = sheet_screen_from_frame(bounds(), COMP, ResolvedCamera::default(), None)
            .expect("letterbox が組めるはず");
        let offset_bounds = Rectangle::new(Point::new(100.0, 50.0), iced::Size::new(640.0, 360.0));
        let offset = sheet_screen_from_frame(offset_bounds, COMP, ResolvedCamera::default(), None)
            .expect("letterbox が組めるはず");
        let p0 = at_origin.transform_point2(Vec2::new(320.0, 180.0));
        let p1 = offset.transform_point2(Vec2::new(320.0, 180.0));
        assert!(
            approx(p0.x, p1.x) && approx(p0.y, p1.y),
            "bounds 原点ずれが写像に漏れている: {p0:?} vs {p1:?}"
        );
    }

    /// User View ではシートは**出力フレーム**についていく — comp の4隅の写りが
    /// フレーム枠 overlay(`render_camera_frame_corners_on_screen`)と一致する
    /// (同じ矩形に住むことの実測)。
    #[test]
    fn sheets_live_on_the_render_camera_frame_under_observation() {
        let observation = ObservationCamera {
            pan: [10.0, -20.0],
            zoom: 2.0,
        };
        let render_camera = ResolvedCamera {
            center: [5.0, 5.0],
            zoom: 1.2,
            roll_degrees: 0.0,
        };
        let affine =
            sheet_screen_from_frame(bounds(), COMP, render_camera, Some(observation))
                .expect("写像が組めるはず");
        let frame_corners = crate::render_camera_frame_corners_on_screen(
            bounds(),
            COMP,
            render_camera,
            observation,
        )
        .expect("フレーム枠が組めるはず");

        let comp_corners = [[0.0, 0.0], [640.0, 0.0], [640.0, 360.0], [0.0, 360.0]];
        for (corner, expected) in comp_corners.into_iter().zip(frame_corners) {
            let p = affine.transform_point2(Vec2::new(corner[0], corner[1]));
            assert!(
                approx(p.x, expected.x) && approx(p.y, expected.y),
                "シートとフレーム枠がずれている: {p:?} vs {expected:?}"
            );
        }
    }

    // -----------------------------------------------------------------
    // トグル(結線用の純関数 — 未実行、波末一括)
    // -----------------------------------------------------------------

    /// 既定は全 off(シートは道具であって常設 chrome ではない)。
    #[test]
    fn toggles_default_to_all_off() {
        assert!(!SheetToggles::default().any());
    }

    /// 1枚だけ反転し、他は動かさない。2回で元に戻る。
    #[test]
    fn toggling_flips_only_the_named_sheet_and_round_trips() {
        let toggles = SheetToggles::default().toggled(Sheet::Thirds);
        assert!(toggles.thirds && !toggles.grid && !toggles.golden_ratio && !toggles.safe_margins);
        assert_eq!(toggles.toggled(Sheet::Thirds), SheetToggles::default());
    }

    /// Message 経由([`SheetToggles::apply`])も同じ意味(結線の読み口)。
    #[test]
    fn apply_routes_the_toggle_message() {
        let toggles = SheetToggles::default()
            .apply(SheetMessage::Toggle(Sheet::Grid))
            .apply(SheetMessage::Toggle(Sheet::SafeMargins));
        assert!(toggles.grid && toggles.safe_margins);
        assert!(!toggles.thirds && !toggles.golden_ratio);
    }
}
