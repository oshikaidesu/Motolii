//! Stage の観測カメラ(裁定157) — 「カメラを通して見る」/「自由に見る」の
//! pan/zoom 入力と、観測中だけ出すレンダリングカメラのフレーム枠 overlay。
//!
//! **engine 側は着地済み**(`motolii_engine::ObservationCamera`・
//! `Engine::render_frame_with_view_camera`、裁定157)。ここは Shell 側の
//! S1(状態・トグル/操作)・S3(overlay)の実装 — export 経路
//! (`Engine::render_frame`)には一切触れない([`crate::Shell::refresh_frame`]
//! が export 真値と Stage 表示用の複製を別々に持つ、そちらの doc comment 参照)。
//!
//! ## 設計: なぜ `canvas::Program` に入力とoverlay描画を両方持たせるか
//!
//! `canvas::Program::update`/`draw` は毎回 `bounds: Rectangle` を直接受け取れる
//! ([`timeline/canvas.rs`]・[`timeline/input.rs`] と同じ形) — `mouse_area` の
//! `on_scroll`/`on_move` はコールバックに widget の実サイズを運ばない
//! (`iced_widget-0.14` の `MouseArea::update` 実測、位置は運ぶがサイズは運ばない)
//! ため、letterbox(D8: `image` の既定 `ContentFit::Contain`)を踏まえた
//! カーソルアンカーズームには使えない。canvas 1枚に両方まとめるのが
//! Timeline と同じ既存動線 — 新しい入力経路を発明しない。
//!
//! ## テスト方針(KNOWN「iced_test は canvas 不可視」)
//!
//! ここの計算は全て `bounds`/`cursor`/`comp`/カメラ値だけを取る**純関数**
//! ([`letterboxed_rect`]・[`zoom_at_screen_point`]・[`pan_by_screen_delta`]・
//! [`render_camera_frame_corners_on_screen`])にしてある —
//! `canvas::Program::update`/`draw` はこれらへ委譲するだけの薄い翻訳層
//! (`timeline/input.rs` と同じ構造)。試験は実 widget を経由せず、これらの
//! 純関数を直接呼ぶ(`timeline_key_gesture_drive.rs` 等、既存の drive 系試験と
//! 同じ形)。

use iced::widget::canvas;
use iced::{mouse, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;

use crate::tokens::{Colors, Dimensions};
use crate::Message;

/// zoom の下限/上限。UI 上の作業視点であって Document の値ではないので、
/// `motolii_core::camera::camera_projection` が持つ `zoom.max(1e-3)`
/// (画角が発散しない床)より内側の、実用的に意味のある範囲へ絞る。
const OBSERVATION_ZOOM_MIN: f32 = 0.1;
const OBSERVATION_ZOOM_MAX: f32 = 20.0;

/// ホイール1ノッチ(`ScrollDelta::Lines` の `y` が1.0)あたりの倍率。
/// Blender/Unity 系 viewport の「1ノッチで軽く拡大」の手触りに合わせた中庸値
/// (自由なデザイン定数 — 裁定に既定値の指定は無い)。
const ZOOM_STEP_FACTOR: f32 = 1.1;

/// トラックパッド等の `ScrollDelta::Pixels` を疑似「ノッチ数」へ正規化する係数。
/// OS のホイール実装は1ノッチ≒何 px か揃っていないので厳密な換算ではないが、
/// 「大きく動かすほど大きくズームする」の方向だけ合わせれば十分(自由な値)。
const PIXELS_PER_NOTCH: f32 = 40.0;

/// `ObservationCamera`(pan/zoom のみ・roll 無し)を `ResolvedCamera` へ写す。
/// `motolii_engine::ObservationCamera::as_resolved_camera` と同じ変換だが、
/// あちらは engine crate 内部専用(`pub` ではない)— フィールドは両方とも
/// `pub` な平坦構造体なので、ここで同じ3行を再構成する方が「engine の private
/// API を割る」よりましと判断した(engine 側は裁定157 で着地済み・この lane の
/// write-set 外)。roll は常に0度(`ObservationCamera` は roll を持たない設計、
/// module doc 参照)。
fn observation_as_resolved(observation: ObservationCamera) -> ResolvedCamera {
    ResolvedCamera {
        center: observation.pan,
        zoom: observation.zoom,
        roll_degrees: 0.0,
    }
}

/// comp を `bounds` へ letterbox で収めた実際の矩形(D8: letterbox は neutral)。
/// `screenshot.rs::blit_letterboxed` と同じ計算 — 実描画(`image` widget の
/// 既定 `ContentFit::Contain`)と同じ挙動を Rust 側で再現する(2箇所目の
/// letterbox 実装だが、`screenshot.rs` 側は `image` crate の `RgbaImage` を
/// 描く headless instrument で、こちらは iced の実 `Rectangle`/`Point` を扱う
/// 実描画側 — 型が違うので1箇所へ統合すると instrument 側が iced 依存を
/// 増やすことになり、`screenshot.rs` 冒頭 doc の設計方針に反する)。
pub fn letterboxed_rect(bounds: Rectangle, comp: CompSpec) -> Option<Rectangle> {
    if bounds.width <= 0.0 || bounds.height <= 0.0 || comp.width == 0 || comp.height == 0 {
        return None;
    }
    let comp_aspect = comp.width as f32 / comp.height as f32;
    let bounds_aspect = bounds.width / bounds.height;
    let (width, height) = if comp_aspect > bounds_aspect {
        (bounds.width, bounds.width / comp_aspect)
    } else {
        (bounds.height * comp_aspect, bounds.height)
    };
    Some(Rectangle {
        x: bounds.x + (bounds.width - width) * 0.5,
        y: bounds.y + (bounds.height - height) * 0.5,
        width,
        height,
    })
}

/// Stage 画面座標(`bounds` ローカル) → 今見えている絵の comp-pixel 座標
/// (= `Engine::render_frame_with_view_camera` が返す RGBA と同じ座標系)。
/// letterbox の外(レターボックス帯・黒帯)でも**外挿する**(`None` を返さない)
/// — letterbox 帯の上でホイールしても無反応にしない(M13。dead zone を作らない)。
/// `letterboxed_rect` が `None`(comp/bounds が退化)の時だけ `None`。
fn screen_to_rendered_comp_pixel(bounds: Rectangle, comp: CompSpec, screen: Point) -> Option<[f32; 2]> {
    let rect = letterboxed_rect(bounds, comp)?;
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    Some([
        (screen.x - rect.x) / rect.width * comp.width as f32,
        (screen.y - rect.y) / rect.height * comp.height as f32,
    ])
}

/// カーソル位置を不動点にしてズームする(裁定157・EXACT TARGET 1「ホイール=
/// Stage ズーム(カーソルアンカー)」)。
///
/// **導出**: 観測カメラ(roll 無し)の z=0 写像は
/// `screen = comp_center + zoom * (world - comp_center - pan)`
/// (`motolii_core::camera::tests::zoom_scales_the_z0_plane_isotropically_around_comp_center`/
/// `panning_the_camera_shifts_the_z0_plane_the_opposite_way` が実測で縛る形と一致)。
/// カーソル位置 `S`(comp-pixel 空間)を固定して zoom だけ `z0→z1` へ変えるとき、
/// 対応する `pan1` は
/// `pan1 = pan0 + (S - comp_center) * (1/z0 - 1/z1)`
/// で閉じた式になる(`glam::Affine2` の逆行列を数値的に取らなくてよい —
/// [`render_camera_frame_corners_on_screen`] が使う一般形はロール付きの
/// レンダリングカメラ用に別途 `camera_screen_from_world_z0` を再利用する)。
///
/// `delta_y` が 0 か、結果の zoom が実質変わらない(床/天井で飽和済み)なら
/// `None`(呼び出し側は Message を出さない = 無駄な `Some` 遷移をしない)。
pub fn zoom_at_screen_point(
    current: ObservationCamera,
    bounds: Rectangle,
    comp: CompSpec,
    cursor: Point,
    delta_y: f32,
) -> Option<ObservationCamera> {
    if delta_y == 0.0 {
        return None;
    }
    let comp_pixel = screen_to_rendered_comp_pixel(bounds, comp, cursor)?;
    let comp_center = [comp.width as f32 * 0.5, comp.height as f32 * 0.5];

    let factor = ZOOM_STEP_FACTOR.powf(delta_y.clamp(-6.0, 6.0));
    let new_zoom = (current.zoom * factor).clamp(OBSERVATION_ZOOM_MIN, OBSERVATION_ZOOM_MAX);
    if (new_zoom - current.zoom).abs() < f32::EPSILON {
        return None;
    }

    let inv_diff = 1.0 / current.zoom - 1.0 / new_zoom;
    let new_pan = [
        current.pan[0] + (comp_pixel[0] - comp_center[0]) * inv_diff,
        current.pan[1] + (comp_pixel[1] - comp_center[1]) * inv_diff,
    ];
    Some(ObservationCamera {
        pan: new_pan,
        zoom: new_zoom,
    })
}

/// `ScrollDelta` を「ノッチ数」(`zoom_at_screen_point` の `delta_y`)へ正規化する。
/// `Lines` はそのまま、`Pixels` は [`PIXELS_PER_NOTCH`] で割る。
pub fn scroll_delta_notches(delta: mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => y,
        mouse::ScrollDelta::Pixels { y, .. } => y / PIXELS_PER_NOTCH,
    }
}

/// 中ボタンドラッグの1フレーム分の画面移動量(screen px、`bounds` ローカル)を
/// pan(world px)へ変換する。「掴んで動かす」手触り(Blender の Shift+MMB と同型):
/// 画面上でカーソルが右へ動けば、絵も右へついてくる。
///
/// **導出**: [`zoom_at_screen_point`] の doc と同じ写像 `screen = comp_center +
/// zoom*(world-comp_center-pan)` を `pan` で偏微分すると `d(screen)/d(pan) = -zoom`
/// (`camera.rs::panning_the_camera_shifts_the_z0_plane_the_opposite_way` が
/// 「pan を +X へ動かすと絵は -X 側へ寄る」で示す符号と一致)。screen を固定した
/// まま world を動かしたいので符号を反転し、letterbox の縮小率で screen px を
/// comp-pixel px へ戻してから zoom で割る。
pub fn pan_by_screen_delta(
    current: ObservationCamera,
    bounds: Rectangle,
    comp: CompSpec,
    screen_delta: [f32; 2],
) -> ObservationCamera {
    let Some(rect) = letterboxed_rect(bounds, comp) else {
        return current;
    };
    if rect.width <= 0.0 || rect.height <= 0.0 || current.zoom.abs() < f32::EPSILON {
        return current;
    }
    let comp_delta = [
        screen_delta[0] / rect.width * comp.width as f32,
        screen_delta[1] / rect.height * comp.height as f32,
    ];
    ObservationCamera {
        pan: [
            current.pan[0] - comp_delta[0] / current.zoom,
            current.pan[1] - comp_delta[1] / current.zoom,
        ],
        zoom: current.zoom,
    }
}

/// レンダリングカメラの写る範囲(comp 全域の4隅)を、観測カメラ視点で見た時の
/// comp-pixel 空間(letterbox 前、`Engine::render_frame_with_view_camera` の
/// 出力と同じ座標系)での位置。**新しい投影数学は無い** — `camera_projection`/
/// `camera_screen_from_world_z0`(`motolii_core::camera`)の合成だけ
/// (縫い目調査 `docs/reviews/2026-08-21-camera-seam-survey.md` §4):
///
/// 1. `render_affine = camera_screen_from_world_z0(comp, render_camera)` の
///    **逆行列**を出力キャンバスの4隅へ掛け、レンダリングカメラが実際に
///    フレーミングしている world 上の矩形(z=0)を求める。
/// 2. その world 座標を `observation_affine = camera_screen_from_world_z0(comp,
///    observation)` で観測カメラ越しの comp-pixel 座標へ写す。
///
/// レンダリングカメラが既定(pan無し・zoom1・roll無し)なら `render_affine` は
/// 恒等写像なので、1. は「出力キャンバスの4隅=world の4隅」に潰れる
/// (`camera.rs::default_camera_matches_identity_pixel_mapping_at_z0` と同じ土台)。
fn render_camera_frame_corners(
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: ObservationCamera,
) -> [[f32; 2]; 4] {
    let render_affine = camera_screen_from_world_z0(comp, render_camera);
    let observation_affine = camera_screen_from_world_z0(comp, observation_as_resolved(observation));
    let render_inverse = render_affine.inverse();

    let canvas_corners = [
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(comp.width as f32, 0.0),
        glam::Vec2::new(comp.width as f32, comp.height as f32),
        glam::Vec2::new(0.0, comp.height as f32),
    ];

    let mut out = [[0.0f32; 2]; 4];
    for (index, corner) in canvas_corners.into_iter().enumerate() {
        let world = render_inverse.transform_point2(corner);
        let observed = observation_affine.transform_point2(world);
        out[index] = [observed.x, observed.y];
    }
    out
}

/// [`render_camera_frame_corners`] を実際の Stage screen 座標(letterbox 込み)
/// へさらに写す。overlay の描画(`StageOverlay::draw`)と `screenshot.rs` 器具が
/// 両方ここを呼ぶ(同じ絵になることを構造で保証する — 2箇所目の letterbox
/// 実装を作らない)。letterbox が組めない(comp/bounds が退化)なら `None`。
pub fn render_camera_frame_corners_on_screen(
    bounds: Rectangle,
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: ObservationCamera,
) -> Option<[Point; 4]> {
    let rect = letterboxed_rect(bounds, comp)?;
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    let corners = render_camera_frame_corners(comp, render_camera, observation);
    Some(corners.map(|c| {
        Point::new(
            rect.x + c[0] / comp.width as f32 * rect.width,
            rect.y + c[1] / comp.height as f32 * rect.height,
        )
    }))
}

/// canvas の drag 状態。**Document でも Session でもない、widget 内だけの
/// 一時状態**(`timeline::input::Interaction` と同格)。
#[derive(Default)]
pub struct Interaction {
    /// 中ボタンドラッグ中の直前フレームのカーソル位置(`bounds` ローカル)。
    pan_origin: Option<Point>,
}

/// Stage の観測カメラ入力(ホイール=ズーム・中ボタンドラッグ=パン)と
/// フレーム枠 overlay(裁定157・S1〜S3)。`Shell::view` が毎フレーム
/// (`self.observation`/comp/レンダリングカメラの今の値から)組み直す —
/// `timeline_pane::TimelinePane` と同じ「不変な投影を canvas::Program に渡す」形。
#[derive(Clone, Copy)]
pub struct StageOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    dims: Dimensions,
    colors: Colors,
}

impl StageOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> Element<'a> {
        iced::widget::canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// [`render_camera_frame_corners_on_screen`] をこの overlay 自身が持つ
    /// comp/レンダリングカメラ/観測カメラで呼ぶだけの薄い口。**`draw` と
    /// `screenshot.rs` の両方がこれを呼ぶ**(2箇所目の計算を作らない — 実 canvas
    /// (iced_wgpu の実描画)と headless instrument(`screenshot.rs`)が同じ絵に
    /// なることを構造で保証する)。観測カメラが無効(`None`)なら `None`
    /// (`draw` の「自明のため描かない」と同じ判断)。
    pub fn frame_corners_on_screen(&self, bounds: Rectangle) -> Option<[Point; 4]> {
        let observation = self.observation?;
        render_camera_frame_corners_on_screen(bounds, self.comp, self.render_camera, observation)
    }
}

/// `iced::Element` の別名(`stage::StageOverlay::view` の戻り値専用) —
/// 呼び出し側(`lib.rs::stage_pane`)が `iced::Element<'_, Message>` を直接
/// 書かずに済む程度の軽い別名で、新しい抽象を増やす意図は無い。
type Element<'a> = iced::Element<'a, Message>;

impl canvas::Program<Message> for StageOverlay {
    type State = Interaction;

    /// **翻訳だけ**(`timeline/input.rs` 冒頭の規律どおり)。ズーム/パンの
    /// 計算は全てモジュール冒頭の純関数(`zoom_at_screen_point`/
    /// `pan_by_screen_delta`)へ委譲する — ここは event を仕分けて渡すだけ。
    ///
    /// **観測に入る操作自体が None→Some 遷移**(裁定157 EXACT TARGET 1):
    /// `self.observation.unwrap_or_default()` を基準値として使うので、
    /// `None` の間にホイール/中ボタンドラッグを始めた瞬間 `Message::StageObserve`
    /// が飛び、`Shell::update` が `self.observation = Some(..)` にする。
    fn update(
        &self,
        state: &mut Interaction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Mouse(mouse_event) = event else {
            return None;
        };
        let current = self.observation.unwrap_or_default();
        match mouse_event {
            mouse::Event::WheelScrolled { delta } => {
                let position = cursor.position_in(bounds)?;
                let delta_notches = scroll_delta_notches(*delta);
                let next = zoom_at_screen_point(current, bounds, self.comp, position, delta_notches)?;
                Some(canvas::Action::publish(Message::StageObserve(next)).and_capture())
            }
            mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                let position = cursor.position_in(bounds)?;
                state.pan_origin = Some(position);
                Some(canvas::Action::capture())
            }
            mouse::Event::CursorMoved { .. } => {
                let origin = state.pan_origin?;
                let position = cursor.position_in(bounds)?;
                state.pan_origin = Some(position);
                let delta = [position.x - origin.x, position.y - origin.y];
                if delta[0] == 0.0 && delta[1] == 0.0 {
                    return Some(canvas::Action::capture());
                }
                let next = pan_by_screen_delta(current, bounds, self.comp, delta);
                Some(canvas::Action::publish(Message::StageObserve(next)).and_capture())
            }
            mouse::Event::ButtonReleased(mouse::Button::Middle) => {
                if state.pan_origin.take().is_some() {
                    Some(canvas::Action::capture())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// **観測中(`Some`)だけ**レンダリングカメラのフレーム枠を hairline で描く
    /// (裁定157・EXACT TARGET 3、M13「なぜ書き出しと違う絵なのか」を黙らせる) —
    /// `None`(カメラを通して見ている間)は何も描かない(自明のため、死に
    /// chrome 回避)。
    fn draw(
        &self,
        _state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let Some(corners) = self.frame_corners_on_screen(bounds) else {
            return Vec::new();
        };

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let path = canvas::Path::new(|builder| {
            builder.move_to(corners[0]);
            for corner in &corners[1..] {
                builder.line_to(*corner);
            }
            builder.close();
        });
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(self.colors.action_active)
                .with_width(self.dims.border_width * 1.5),
        );
        vec![frame.into_geometry()]
    }

    /// 中ボタンドラッグ中は掴んでいる手触り(`timeline/input.rs::mouse_interaction`
    /// と同じ `Grabbing` の使い方)。それ以外は既定(observation の有無に関わらず
    /// ホイール/中ボタンは常に効くので、常時 `Grab` を出すと「常設 chrome の
    /// hover 色」のような誤読を招く — カーソル形状は「今まさに掴んでいる」時
    /// だけ変える)。
    fn mouse_interaction(
        &self,
        state: &Interaction,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.pan_origin.is_some() {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    /// 16:9 の comp を同アスペクトの bounds へ収めると letterbox は要らない
    /// (bounds 全域が rect になる)。
    #[test]
    fn letterboxed_rect_fills_bounds_when_aspect_matches() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(1280.0, 720.0));
        let rect = letterboxed_rect(bounds, COMP).expect("有効な bounds/comp");
        assert!((rect.width - 1280.0).abs() < 1e-3);
        assert!((rect.height - 720.0).abs() < 1e-3);
        assert!((rect.x).abs() < 1e-3);
        assert!((rect.y).abs() < 1e-3);
    }

    /// 正方形の bounds へ 16:9 comp を収めると上下に letterbox 帯ができ、中央寄せになる。
    #[test]
    fn letterboxed_rect_centers_with_vertical_bars_on_a_square_container() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(360.0, 360.0));
        let rect = letterboxed_rect(bounds, COMP).expect("有効な bounds/comp");
        // 640x360 を幅360に収めると高さは 360*360/640 = 202.5。
        assert!((rect.width - 360.0).abs() < 1e-3);
        assert!((rect.height - 202.5).abs() < 1e-1);
        assert!(rect.y > 0.0, "上下に letterbox 帯が要るはず");
    }

    /// **カーソルアンカーの本命**: comp 中心以外の点でズームしても、
    /// その点の comp-pixel 位置は zoom 前後で screen 座標が変わらない。
    #[test]
    fn zoom_at_screen_point_keeps_the_cursor_anchored() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0));
        let current = ObservationCamera::default();
        // comp 中心から離れた点(中心にすると常に不動点になり試験にならない)。
        let cursor = Point::new(160.0, 90.0);
        let before = screen_to_rendered_comp_pixel(bounds, COMP, cursor).unwrap();

        let next = zoom_at_screen_point(current, bounds, COMP, cursor, 3.0)
            .expect("ズームが起きるはず");
        assert!(next.zoom > current.zoom, "delta_y>0 はズームインのはず");

        // 新しい観測カメラでの affine を組み、同じ comp-pixel(before)が同じ
        // screen 座標(cursor)へ戻ることを確認する。
        let affine = camera_screen_from_world_z0(COMP, observation_as_resolved(next));
        let mapped = affine.transform_point2(glam::Vec2::new(before[0], before[1]));
        assert!(
            (mapped.x - cursor.x).abs() < 1e-2 && (mapped.y - cursor.y).abs() < 1e-2,
            "カーソル位置が不動点になっていない: mapped={mapped:?} cursor={cursor:?}"
        );
    }

    /// delta が0ならズームしない(無駄な None→Some 遷移をしない)。
    #[test]
    fn zoom_at_screen_point_is_none_when_delta_is_zero() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0));
        let result = zoom_at_screen_point(
            ObservationCamera::default(),
            bounds,
            COMP,
            Point::new(320.0, 180.0),
            0.0,
        );
        assert!(result.is_none());
    }

    /// 中ボタンドラッグ: 画面上でカーソルが右へ動けば、絵も右へついてくる
    /// (= comp 中心が screen 上でもカーソルと同じ向きへ動く)。
    #[test]
    fn pan_by_screen_delta_follows_the_cursor_direction() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0));
        let current = ObservationCamera::default();
        let next = pan_by_screen_delta(current, bounds, COMP, [50.0, 0.0]);

        let before_affine = camera_screen_from_world_z0(COMP, observation_as_resolved(current));
        let after_affine = camera_screen_from_world_z0(COMP, observation_as_resolved(next));
        let comp_center = glam::Vec2::new(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);
        let before_screen = before_affine.transform_point2(comp_center);
        let after_screen = after_affine.transform_point2(comp_center);

        assert!(
            after_screen.x > before_screen.x,
            "右へドラッグしたら絵も右へついてくるはず: before={before_screen:?} after={after_screen:?}"
        );
    }

    /// 既定のレンダリングカメラ(pan無し・zoom1・roll無し)・既定の観測カメラ
    /// (=カメラを通して見る相当の値)なら、フレーム枠は comp の4隅そのもの
    /// (letterbox 込みの screen 座標)に一致する。
    #[test]
    fn frame_corners_match_the_comp_rect_when_both_cameras_are_default() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0));
        let corners = render_camera_frame_corners_on_screen(
            bounds,
            COMP,
            ResolvedCamera::default(),
            ObservationCamera::default(),
        )
        .expect("letterbox が組めるはず");

        assert!((corners[0].x - 0.0).abs() < 1e-1 && (corners[0].y - 0.0).abs() < 1e-1);
        assert!((corners[2].x - 640.0).abs() < 1e-1 && (corners[2].y - 360.0).abs() < 1e-1);
    }

    /// 観測カメラをズームインすると、フレーム枠(レンダリングカメラの範囲)は
    /// screen 上でより大きく写る(観測が近づいたぶん枠も拡大して見える)。
    #[test]
    fn frame_corners_grow_when_observation_zooms_in() {
        let bounds = Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0));
        let zoomed_in = ObservationCamera {
            pan: [0.0, 0.0],
            zoom: 2.0,
        };
        let corners = render_camera_frame_corners_on_screen(
            bounds,
            COMP,
            ResolvedCamera::default(),
            zoomed_in,
        )
        .expect("letterbox が組めるはず");

        let width = corners[2].x - corners[0].x;
        assert!(width > 640.0 * 1.5, "枠が拡大していない: width={width}");
    }
}
