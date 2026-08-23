use glam::{Affine2, Vec2};
use iced::{Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_tokens_rs::Dimensions;

use crate::{letterboxed_rect, observation_as_resolved};

use super::*;

/// bbox の8ハンドル(角4+辺4)。並びは [`SCALE_HANDLES`] が固定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleHandle {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// 8ハンドルの固定順(layout の `scale_handles` と index で対応する)。
pub const SCALE_HANDLES: [ScaleHandle; 8] = [
    ScaleHandle::TopLeft,
    ScaleHandle::Top,
    ScaleHandle::TopRight,
    ScaleHandle::Left,
    ScaleHandle::Right,
    ScaleHandle::BottomLeft,
    ScaleHandle::Bottom,
    ScaleHandle::BottomRight,
];

impl ScaleHandle {
    /// レイヤーローカルの座標(内容矩形 `(0,0)..size` の角と辺中点)。
    pub fn local_point(self, size: [f32; 2]) -> [f32; 2] {
        let [w, h] = size;
        match self {
            Self::TopLeft => [0.0, 0.0],
            Self::Top => [w * 0.5, 0.0],
            Self::TopRight => [w, 0.0],
            Self::Left => [0.0, h * 0.5],
            Self::Right => [w, h * 0.5],
            Self::BottomLeft => [0.0, h],
            Self::Bottom => [w * 0.5, h],
            Self::BottomRight => [w, h],
        }
    }

    pub fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    /// この handle が動かす軸 `(x, y)`(AE と同じ: 辺は1軸、角は2軸)。
    pub fn affects(self) -> (bool, bool) {
        match self {
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight => (true, true),
            Self::Left | Self::Right => (true, false),
            Self::Top | Self::Bottom => (false, true),
        }
    }
}

/// hit-test の結果(= drag の種別)。優先順位は [`gizmo_hit_test`] doc 参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandle {
    /// bbox 内部 = move(position)。
    Body,
    /// 8ハンドル = scale。
    Scale(ScaleHandle),
    /// 回転ハンドル = rotate(rotation)。
    Rotate,
    /// anchor ⊕ = anchor drag(第2切片 — anchor 変更+position 補償)。
    Anchor,
}

impl GizmoHandle {
    pub fn property(self) -> GizmoProperty {
        match self {
            Self::Body => GizmoProperty::Position,
            Self::Scale(_) => GizmoProperty::Scale,
            Self::Rotate => GizmoProperty::Rotation,
            Self::Anchor => GizmoProperty::Anchor,
        }
    }
}

/// letterbox 込みの comp-pixel → screen(bounds ローカル)のアフィン。
/// [`letterboxed_rect`] と同じ矩形を**原点0へ正規化した bounds** で組む
/// (モジュール冒頭 doc「座標系」— `draw` のローカル Frame・`position_in` の
/// ローカル cursor と同じ系に揃える)。退化(comp/bounds が 0)なら `None`。
/// `pub(crate)`: 方眼シート overlay([`crate::sheets`])が同じ原点正規化を
/// 共有する(GZ FINDING を2箇所目で再発させない — 計算を複製しない)。
pub(crate) fn letterbox_screen_from_comp(bounds: Rectangle, comp: CompSpec) -> Option<Affine2> {
    let local_bounds = Rectangle::new(Point::ORIGIN, bounds.size());
    let rect = letterboxed_rect(local_bounds, comp)?;
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    Some(
        Affine2::from_translation(Vec2::new(rect.x, rect.y))
            * Affine2::from_scale(Vec2::new(
                rect.width / comp.width as f32,
                rect.height / comp.height as f32,
            )),
    )
}

/// 今 Stage に映っている視点のカメラ(観測中は観測カメラ、そうでなければ
/// レンダリングカメラ — `Shell::compute_display_source` が絵を出すのと同じ分岐)。
fn display_camera(
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
) -> ResolvedCamera {
    observation.map(observation_as_resolved).unwrap_or(render_camera)
}

/// 1フレームぶんのギズモの絵と当たり判定の正本。`draw`/`mouse_interaction`/
/// `update`(press)が全員これを読む — 「見えている位置」と「触れる位置」が
/// 構造的に一致する(Q0)。
#[derive(Debug, Clone, Copy)]
pub struct GizmoLayout {
    /// レイヤーローカル → screen(bounds ローカル)。
    pub screen_from_local: Affine2,
    /// 親空間 → screen。drag の解([`GizmoDragState`])が逆行列を使う。
    pub screen_from_parent: Affine2,
    /// 対象の内容矩形(レイヤーローカル px)。body hit-test 用。
    pub size: [f32; 2],
    /// bbox の4隅(TL, TR, BR, BL の順、screen)。
    pub corners: [Point; 4],
    /// 8ハンドルの screen 位置([`SCALE_HANDLES`] と index 対応)。
    pub scale_handles: [Point; 8],
    /// 回転ハンドルの screen 位置。bbox が線に潰れている(上辺中点と中心が
    /// 一致する)時は `None`(出せない物は描かない)。
    pub rotate_handle: Option<Point>,
    /// 上辺中点(回転ハンドルの stem の根本)。
    pub top_center: Point,
    /// anchor の screen 位置(表示のみ)。
    pub anchor: Point,
    /// ハンドル命中の半径(screen px、`dims.gizmo_hit_radius`)。
    pub hit_radius: f32,
}

/// ギズモの幾何を組む(純関数)。観測カメラのズーム/パン下でも正しい位置に
/// 出る(モジュール冒頭の合成 — 座標変換は既存関数の合成だけ)。
pub fn gizmo_layout(
    bounds: Rectangle,
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: &GizmoTarget,
    dims: Dimensions,
) -> Option<GizmoLayout> {
    let letterbox = letterbox_screen_from_comp(bounds, comp)?;
    let camera = camera_screen_from_world_z0(comp, display_camera(render_camera, observation));
    let screen_from_world = letterbox * camera;
    let screen_from_parent = screen_from_world * target.world_from_parent;
    let screen_from_local = screen_from_world * target.world_from_local();
    if !screen_from_local.is_finite() || !screen_from_parent.is_finite() {
        return None;
    }

    let [w, h] = target.size;
    let to_point = |local: [f32; 2]| -> Point {
        let p = screen_from_local.transform_point2(Vec2::new(local[0], local[1]));
        Point::new(p.x, p.y)
    };

    let corners = [
        to_point([0.0, 0.0]),
        to_point([w, 0.0]),
        to_point([w, h]),
        to_point([0.0, h]),
    ];
    let mut scale_handles = [Point::ORIGIN; 8];
    for (index, handle) in SCALE_HANDLES.into_iter().enumerate() {
        scale_handles[index] = to_point(handle.local_point(target.size));
    }

    // 回転ハンドル: 上辺中点を bbox 中心から外側へ延長した先(screen 空間で
    // 固定 px オフセット — ズームしてもハンドルの実寸は変わらない、Figma/Canva の
    // ハンドルが画面固定寸なのと同じ)。反転(負 scale)でも「上辺の外側」が
    // 向きごと反転してついてくる。
    let center = to_point([w * 0.5, h * 0.5]);
    let top_center = to_point([w * 0.5, 0.0]);
    let stem = Vec2::new(top_center.x - center.x, top_center.y - center.y);
    let rotate_handle = if stem.length_squared() > SOLVE_EPS * SOLVE_EPS {
        let direction = stem.normalize();
        Some(Point::new(
            top_center.x + direction.x * dims.gizmo_rotate_offset,
            top_center.y + direction.y * dims.gizmo_rotate_offset,
        ))
    } else {
        None
    };

    Some(GizmoLayout {
        screen_from_local,
        screen_from_parent,
        size: target.size,
        corners,
        scale_handles,
        rotate_handle,
        top_center,
        anchor: to_point(target.anchor),
        hit_radius: dims.gizmo_hit_radius,
    })
}

/// 命中判定(純関数)。優先順位(重なった時に「小さくて外側の物」が勝つ —
/// AE/Figma と同じ、body に飲み込まれてハンドルが触れなくなる事故を防ぐ):
///
/// 1. 回転ハンドル(半径 `hit_radius` 内)
/// 2. 8ハンドルのうち半径内の最寄り。**角が辺に優先**(同距離なら角 —
///    レイヤーが小さく潰れてハンドル同士が重なった時、2軸動かせる角を残す)
/// 3. bbox 内部 = body(move)
/// 4. どれでもない = `None`(イベントは下の層 — 観測カメラ操作等 — へ素通し)
pub fn gizmo_hit_test(layout: &GizmoLayout, cursor: Point) -> Option<GizmoHandle> {
    let radius_squared = layout.hit_radius * layout.hit_radius;
    let distance_squared = |p: Point| -> f32 {
        let dx = cursor.x - p.x;
        let dy = cursor.y - p.y;
        dx * dx + dy * dy
    };

    if let Some(rotate) = layout.rotate_handle {
        if distance_squared(rotate) <= radius_squared {
            return Some(GizmoHandle::Rotate);
        }
    }

    let mut best: Option<(bool, f32, ScaleHandle)> = None;
    for (index, handle) in SCALE_HANDLES.into_iter().enumerate() {
        let d = distance_squared(layout.scale_handles[index]);
        if d > radius_squared {
            continue;
        }
        let candidate = (handle.is_corner(), d, handle);
        best = match best {
            None => Some(candidate),
            Some(current) => {
                // 角(true)が辺(false)より優先、同格なら近い方。
                let better = (candidate.0 && !current.0)
                    || (candidate.0 == current.0 && candidate.1 < current.1);
                Some(if better { candidate } else { current })
            }
        };
    }
    if let Some((_, _, handle)) = best {
        return Some(GizmoHandle::Scale(handle));
    }

    // anchor ⊕(第2切片で drag 対象)。scale ハンドルの後・body の前 —
    // 極小レイヤーでハンドル群と重なった時は2軸動かせる角/辺を残し、body に
    // 飲み込まれて掴めなくなる事故は防ぐ(優先順位の原則は doc 冒頭と同じ
    // 「小さい物が大きい物に勝つ」)。命中半径は他ハンドルと同じ
    // `hit_radius` — 見た目(`gizmo_anchor_radius` = 4)より広い判定で
    // Q0「見えている物は必ず触れる」を保つ。
    if distance_squared(layout.anchor) <= radius_squared {
        return Some(GizmoHandle::Anchor);
    }

    // body: レイヤーローカルへ戻して内容矩形の中か(回転/skew/カメラ込みで正しい
    // 判定になる)。ローカルへ戻れない(scale 0 で潰れている)なら body は無い。
    if let Some(local_from_screen) = checked_inverse(layout.screen_from_local) {
        let local = local_from_screen.transform_point2(Vec2::new(cursor.x, cursor.y));
        if local.x >= 0.0 && local.x <= layout.size[0] && local.y >= 0.0 && local.y <= layout.size[1]
        {
            return Some(GizmoHandle::Body);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// drag の解(純関数)と状態機械
// ---------------------------------------------------------------------------

