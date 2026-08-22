//! Stage ギズモ第1弾(裁定124: スクラッチ・意味の手本=AE・見た目=先例の慣習形) —
//! 選択レイヤーの bbox+ハンドル8点+回転ハンドル+anchor 表示と、
//! drag = move(本体)/scale(角・辺)/rotate(回転ハンドル)。
//!
//! ## 意味(全部実装済みの経路を呼ぶだけ — ここは「絵と手」)
//!
//! - 値の意味は Inspector 値編集と同じ property(`position`/`scale`/`rotation`)。
//!   このモジュールは **Document を一切書かない** — [`GizmoDrag`] message を publish
//!   するだけで、shell 側(結線は supervisor)が Inspector drag と同じ経路
//!   (`Document::set_transient` → 確定時に `Intent::SetTrack`、キー持ち track へは
//!   AE 作法の playhead upsert)へ写す。**1 drag = 1 commit**(1 gesture = 1 undo)。
//! - **scale/rotate の不動点は anchor**(AE と同じ: comp panel のハンドル drag は
//!   Scale/Rotation property だけを書き、position は動かさない — anchor が数学的な
//!   不動点になる)。move は position だけを書く。
//! - Shift = 比率固定(scale、map 680「Modify Scale constrained to aspect ratio」)/
//!   15° スナップ(rotate、map 679「Modify Rotation in 15° increments」)。
//!   Esc = キャンセル(transient を捨てて drag 前の値へ戻る)。
//!
//! ## 見た目(AE/Figma/Canva 系の慣習形 — 独自形を発明しない)
//!
//! - bbox hairline(accent =「選択の器」)+角4・辺4 の正方形ハンドル8点
//!   (Figma/AE の selection handles と同形)。
//! - 回転ハンドル = 上辺中点から外側へ stem で繋いだ小円(Canva/PowerPoint/
//!   Google Slides の慣習形 — Figma の「角の外側の不可視ゾーン」は Q0
//!   (触れそうで触れない/触れるのに見えない)に反するので採らない)。
//! - anchor = ⊕(円+十字、AE のアンカーポイントの慣習形)。第1切片は表示のみ
//!   だったが、**第2切片(B22 波、2026-08-22)で drag 対象**: anchor drag =
//!   anchor 変更+position 補償(AE の pan-behind と同じ「見た目不動」 —
//!   [`anchor_value`] doc の導出参照)。2 property を [`GizmoValue::Anchor`] が
//!   対で運ぶ(片方だけ書くと絵が跳ぶため、1 message に両方乗せる)。
//!
//! ## 座標系(このモジュール内は **bounds ローカル** で閉じる)
//!
//! `canvas::Program` の `bounds` は window 絶対座標で届くが、`draw` の `Frame` は
//! bounds 原点へ translate された**ローカル座標**、`cursor.position_in(bounds)` も
//! ローカル座標を返す(iced_widget-0.14 `canvas.rs::draw` の `with_translation` 実測)。
//! そのためここでは letterbox 矩形を **bounds 原点=0 に正規化してから** 組む
//! ([`letterbox_screen_from_comp`])— 入力(hit)と出力(描画)が同じローカル系に
//! 揃う。写像の合成は既存部品だけ(新しい投影数学は無い):
//!
//! ```text
//! screen_from_local = letterbox ∘ camera_screen_from_world_z0(comp, 観測 or レンダリングカメラ)
//!                     ∘ world_from_parent ∘ LayerPlacement::from_transform(局所値)
//! ```
//!
//! ## テスト方針(モジュール冒頭 lib.rs と同じ)
//!
//! 計算は全て純関数([`gizmo_layout`]・[`gizmo_hit_test`]・[`move_value`]・
//! [`scale_value`]・[`rotation_value`]・[`GizmoDragState`])— `canvas::Program`
//! (`GizmoOverlay`)はこれらへ委譲するだけの薄い翻訳層。試験は純関数を直接呼ぶ。

use glam::{Affine2, Vec2};
use iced::widget::canvas;
use iced::{keyboard, mouse, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, LayerPlacement, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_store::{property, LayerId, PropertyId, RationalTime, StoreView, Value};
use motolii_tokens_rs::{Colors, Dimensions};

use crate::{letterboxed_rect, observation_as_resolved};

/// scale 解の分母(`handle - anchor` の成分、レイヤーローカル px)がこれ未満なら
/// その軸は解けない(anchor がハンドルの線上にある)— 開始時の scale を保つ。
const SOLVE_EPS: f32 = 1e-3;

// ---------------------------------------------------------------------------
// pane ローカル Message(発注書「`GizmoDrag { target_property, phase }` 級」)。
// 既存の [`crate::Message`] へ variant を足すと shell 側の exhaustive match が
// 壊れる(shell はこのレーンの write-set 外)ため、**独立した message 型**にして
// ある — supervisor が root 側で `.map(...)` して畳む。
// ---------------------------------------------------------------------------

/// ギズモ drag の1イベント。`GizmoOverlay` の canvas が publish する。
///
/// **契約**: 1回の drag は必ず `Start` で始まり、0回以上の `Move` を挟んで、
/// **ちょうど1回の `Commit` か `Cancel`** で終わる。shell 側の想定結線
/// (Inspector の drag-to-scrub と同型):
///
/// - `Start`: transient 準備(何も書かなくてよい)
/// - `Move { value }`: `Document::set_transient(layer, property, value)`
/// - `Commit { value }`: transient を外し、`Intent::SetTrack` 1回
///   (キー持ち property へは AE 作法の playhead upsert — 1 drag = 1 commit)
/// - `Cancel`: transient を外すだけ(Esc、または動かさずに release した空クリック)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoDrag {
    pub layer: LayerId,
    pub phase: GizmoPhase,
}

/// drag の段階。`Move`/`Commit` の値は [`GizmoValue`] が property の区別ごと運ぶ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoPhase {
    Start { property: GizmoProperty },
    Move { value: GizmoValue },
    Commit { value: GizmoValue },
    Cancel,
}

/// drag が書く先の property(第2切片で Anchor が加入)。shell 側の宛先:
/// [`property::POSITION`] / [`property::SCALE`] / [`property::ROTATION`] /
/// [`property::ANCHOR`]。**`Anchor` だけは2 property を書く**
/// ([`GizmoValue::Anchor`] が anchor と補償済み position を対で運ぶ —
/// `property_name` は主となる anchor 側の名前を返す)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoProperty {
    Position,
    Scale,
    Rotation,
    Anchor,
}

impl GizmoProperty {
    /// store の property 名(shell 結線用の読み口 — 文字列を二重に持たない)。
    /// `Anchor` は主 property(anchor)の名前 — 補償の書き先は
    /// [`property::POSITION`]([`GizmoValue::Anchor`] doc 参照)。
    pub fn property_name(self) -> &'static str {
        match self {
            Self::Position => property::POSITION,
            Self::Scale => property::SCALE,
            Self::Rotation => property::ROTATION,
            Self::Anchor => property::ANCHOR,
        }
    }
}

/// drag が計算した新しい値(store の単位そのまま: position/anchor = comp/親空間
/// /ローカル px、scale = 1.0 が等倍、rotation = 度・時計回り)。shell 側は
/// `Value::Vec2`/`Value::F64` へ写すだけ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoValue {
    Position([f64; 2]),
    Scale([f64; 2]),
    Rotation(f64),
    /// anchor drag(AE pan-behind 型)。**shell は両方を同時に書く**
    /// (transient も commit も対で — 「見た目不動」の不変量は anchor と
    /// position を同時に書けて初めて成立する。1 drag = 1 commit の契約は
    /// 変わらない: 1 gesture で2 property に1回ずつの upsert を**1つの
    /// history 段**として畳むのは shell 結線側の責務)。
    Anchor {
        /// レイヤーローカル px([`property::ANCHOR`] へ)。
        anchor: [f64; 2],
        /// 補償済みの親空間 px([`property::POSITION`] へ)。
        position: [f64; 2],
    },
}

impl GizmoValue {
    pub fn property(self) -> GizmoProperty {
        match self {
            Self::Position(_) => GizmoProperty::Position,
            Self::Scale(_) => GizmoProperty::Scale,
            Self::Rotation(_) => GizmoProperty::Rotation,
            Self::Anchor { .. } => GizmoProperty::Anchor,
        }
    }
}

// ---------------------------------------------------------------------------
// 対象(shell が選択レイヤーから組んで渡す投影)
// ---------------------------------------------------------------------------

/// ギズモの対象 = 選択レイヤーの変形の**局所値**と、親までの合成。
///
/// 局所値(anchor/position/scale/rotation/skew)は [`LayerPlacement::from_transform`]
/// (裁定58 の正本)の引数そのもの — 行列だけ持つと scale と rotation を分離できず
/// drag の解が立たないため、値で持つ。`world_from_parent` は親鎖(裁定173 H1)の
/// world アフィン(親が無ければ恒等)— position が親空間の値なので、drag の解は
/// 全部この空間で立てる(親の回転/拡大の下でも正しい値になる)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoTarget {
    pub layer: LayerId,
    /// レイヤーローカルの内容矩形 `(0,0)..size`(px)。Document が寸法を知らない
    /// 素材(Media/Text 等、`LayerSource::declared_size` が `None`)は呼び出し側が
    /// 実寸(engine の texture 実寸など)を渡す。
    pub size: [f32; 2],
    pub anchor: [f32; 2],
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation_degrees: f32,
    pub skew_degrees: f32,
    pub skew_axis_degrees: f32,
    /// 親鎖の world(comp 空間)アフィン。親が無ければ `Affine2::IDENTITY`。
    pub world_from_parent: Affine2,
}

impl GizmoTarget {
    /// レイヤーローカル → comp(world)のアフィン。正本の組み方
    /// ([`LayerPlacement::from_transform`])をそのまま呼ぶ — ここで行列を
    /// 発明しない。
    pub fn world_from_local(&self) -> Affine2 {
        self.world_from_parent
            * LayerPlacement::from_transform(
                self.anchor,
                self.position,
                self.scale,
                self.rotation_degrees,
                self.skew_degrees,
                self.skew_axis_degrees,
            )
    }
}

/// `StoreView` から [`GizmoTarget`] を組む読み口(書き口ゼロ —
/// [`crate::observation_preview_source`] と同じ「明示引数の自由関数」の形)。
///
/// `None` の条件: comp が無い / 時刻を写せない / この時刻にレイヤーが居ない
/// (hidden・solo 外・配置の外 — `StoreView::resolve` の `None` と同じ意味)/
/// property の読みが壊れている(型不一致)。いずれも「ギズモを出さない」が安全側。
///
/// `size` は呼び出し側が渡す([`GizmoTarget::size`] の doc 参照)。
pub fn gizmo_target(
    view: &StoreView<'_>,
    layer: LayerId,
    playhead: i64,
    size: [f32; 2],
) -> Option<GizmoTarget> {
    let composition = view.composition().ok().flatten()?;
    let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
    // 「今この時刻に見えているか」の門は resolve に任せる(hidden/solo/配置の判定を
    // ここで再実装しない)。見えていない物のギズモは出さない(Q0: 触れない物を描かない)。
    view.resolve(layer, t).ok().flatten()?;

    let scalar = |name: &str, default: f32| -> Option<f32> {
        let property = PropertyId::new(name).ok()?;
        match view.value_at(layer, &property, t).ok()? {
            Some(Value::F64(v)) => Some(v as f32),
            Some(_) => None, // 型不一致は黙って既定値に落とさない(打った値が効かない偽装を避ける)
            None => Some(default),
        }
    };
    let vec2 = |name: &str, default: [f32; 2]| -> Option<[f32; 2]> {
        let property = PropertyId::new(name).ok()?;
        match view.value_at(layer, &property, t).ok()? {
            Some(Value::Vec2(v)) => Some([v[0] as f32, v[1] as f32]),
            Some(_) => None,
            None => Some(default),
        }
    };

    // position は `position`(Vec2)優先、無ければ split(`position.x`/`position.y`)
    // — `StoreView::resolve_position`(private)と同じ裁定61 の意味論。
    let position = {
        let p = PropertyId::new(property::POSITION).ok()?;
        match view.value_at(layer, &p, t).ok()? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            Some(_) => return None, // 型不一致(scalar と同じ扱い)
            None => {
                let x = scalar(property::POSITION_X, 0.0)?;
                let y = scalar(property::POSITION_Y, 0.0)?;
                [x, y]
            }
        }
    };

    // 親鎖の world 合成(裁定173 H1 の意味論: 親が居なければ恒等、壊れた参照/循環は
    // そこで打ち切ってローカルへ縮退)。`StoreView::world_affine` は private なので
    // 公開口(`attrs` + `local_transform`)から同じ合成を組む。
    let mut world_from_parent = Affine2::IDENTITY;
    let mut seen = std::collections::HashSet::new();
    let mut current = view.attrs(layer).ok()?.unwrap_or_default().parent;
    while let Some(parent) = current {
        if !seen.insert(parent) || !view.has_layer(parent) {
            break;
        }
        world_from_parent = view.local_transform(parent, t).ok()? * world_from_parent;
        current = view.attrs(parent).ok()?.unwrap_or_default().parent;
    }

    Some(GizmoTarget {
        layer,
        size,
        anchor: vec2(property::ANCHOR, [0.0, 0.0])?,
        position,
        scale: vec2(property::SCALE, [1.0, 1.0])?,
        rotation_degrees: scalar(property::ROTATION, 0.0)?,
        skew_degrees: scalar(property::SKEW, 0.0)?,
        skew_axis_degrees: scalar(property::SKEW_AXIS, 0.0)?,
        world_from_parent,
    })
}

// ---------------------------------------------------------------------------
// ハンドルの語彙と幾何
// ---------------------------------------------------------------------------

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
    let local_from_screen = layout.screen_from_local.inverse();
    if local_from_screen.is_finite() {
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

/// move: 親空間での cursor 変位をそのまま position へ足す(position は親空間の値)。
pub fn move_value(start: &GizmoTarget, start_cursor_parent: Vec2, cursor_parent: Vec2) -> [f64; 2] {
    let delta = cursor_parent - start_cursor_parent;
    [
        (start.position[0] + delta.x) as f64,
        (start.position[1] + delta.y) as f64,
    ]
}

/// scale: anchor 不動(position 不変)で、掴んだハンドルが cursor に来る scale を
/// 閉じた式で解く。
///
/// **導出**: 局所→親の写像は `M(S) = T(pos)·R·K·S·T(-anchor)`
/// ([`LayerPlacement::from_transform`] の適用順)。ハンドルのローカル点 `h` が
/// cursor(親空間)に来る条件は `S'·(h-a) = (R·K)⁻¹·(cursor - pos)` — 右辺 `q` を
/// 出せば各軸独立に `s_i = q_i / (h_i - a_i)`。`R·K` は
/// `from_transform(anchor=0, pos=0, scale=1, rotation, skew, skew_axis)` で正本から
/// そのまま組む(skew の式をここへ複製しない)。分母が 0(anchor がハンドルの
/// 線上)の軸と、辺ハンドルの動かさない軸は開始時の値を保つ。負の解は許す
/// (AE と同じ: ハンドルを反対側へ引き抜くと反転)。
///
/// Shift(比率固定、map 680): 変化率 `f_i = s_i' / s_i(開始)` の**変化が大きい方**
/// を両軸へ適用する(辺ハンドルは解けた1軸の f を両軸へ)。開始 scale が 0 の軸は
/// 率が立たないので相手側の f を使う。
pub fn scale_value(
    start: &GizmoTarget,
    handle: ScaleHandle,
    cursor_parent: Vec2,
    shift: bool,
) -> [f64; 2] {
    let rot_skew = LayerPlacement::from_transform(
        [0.0, 0.0],
        [0.0, 0.0],
        [1.0, 1.0],
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    // 回転+skew(shear)は det=1 で常に可逆。
    let q = rot_skew
        .inverse()
        .transform_vector2(cursor_parent - Vec2::new(start.position[0], start.position[1]));

    let h = handle.local_point(start.size);
    let denom = [h[0] - start.anchor[0], h[1] - start.anchor[1]];
    let (affects_x, affects_y) = handle.affects();

    let solve = |affects: bool, q_i: f32, denom_i: f32| -> Option<f32> {
        (affects && denom_i.abs() > SOLVE_EPS).then(|| q_i / denom_i)
    };
    let solved_x = solve(affects_x, q.x, denom[0]);
    let solved_y = solve(affects_y, q.y, denom[1]);

    if !shift {
        return [
            solved_x.unwrap_or(start.scale[0]) as f64,
            solved_y.unwrap_or(start.scale[1]) as f64,
        ];
    }

    let factor_of = |solved: Option<f32>, start_scale: f32| -> Option<f32> {
        let solved = solved?;
        (start_scale.abs() > SOLVE_EPS).then(|| solved / start_scale)
    };
    let fx = factor_of(solved_x, start.scale[0]);
    let fy = factor_of(solved_y, start.scale[1]);
    let factor = match (fx, fy) {
        (Some(fx), Some(fy)) => {
            if (fx - 1.0).abs() >= (fy - 1.0).abs() {
                fx
            } else {
                fy
            }
        }
        (Some(f), None) | (None, Some(f)) => f,
        (None, None) => 1.0,
    };
    [
        (start.scale[0] * factor) as f64,
        (start.scale[1] * factor) as f64,
    ]
}

/// rotate: anchor(親空間では position そのもの — `M(anchor) = position`)を
/// 中心に、開始 cursor と今の cursor の親空間での角度差を開始 rotation へ足す。
/// 親空間で測るので観測カメラのズーム/パン/レンダリングカメラの roll に依らない。
/// y-down 座標では `atan2` の増加方向=時計回り = store の rotation の符号と一致。
///
/// Shift(map 679): 結果の絶対角を最寄りの 15° 刻みへスナップ。
pub fn rotation_value(
    start: &GizmoTarget,
    start_cursor_parent: Vec2,
    cursor_parent: Vec2,
    shift: bool,
) -> f64 {
    let pivot = Vec2::new(start.position[0], start.position[1]);
    let v0 = start_cursor_parent - pivot;
    let v1 = cursor_parent - pivot;
    let mut degrees = start.rotation_degrees as f64;
    if v0.length_squared() > SOLVE_EPS * SOLVE_EPS && v1.length_squared() > SOLVE_EPS * SOLVE_EPS {
        let mut delta = (v1.y.atan2(v1.x) - v0.y.atan2(v0.x)).to_degrees() as f64;
        // atan2 の分枝跨ぎ((-360,360) に散る)を1回転内へ畳む。
        if delta > 180.0 {
            delta -= 360.0;
        } else if delta < -180.0 {
            delta += 360.0;
        }
        degrees += delta;
    }
    if shift {
        degrees = (degrees / 15.0).round() * 15.0;
    }
    degrees
}

/// anchor drag(第2切片、AE pan-behind 型): anchor を cursor の下へ移し、
/// **見た目は不動**になるよう position を補償する。
///
/// **導出**: 局所→親の写像は `M = T(pos)·R·K·S·T(-a)`
/// ([`LayerPlacement::from_transform`] の適用順)。任意の局所点 `p` の像
/// `M(p) = pos + RKS(p - a)` を全 `p` で不変に保ったまま `a0 → a1` へ変えると
/// `pos1 = pos0 + RKS·(a1 - a0)` が唯一解。新しい anchor は「cursor の真下の
/// 局所点」 `a1 = M0⁻¹(cursor_parent)` — このとき anchor の親空間の像は
/// `M1(a1) = pos1 = pos0 + RKS(a1 - a0) = M0(a1) = cursor_parent`、つまり
/// **⊕ が cursor に吸い付き、絵は1px も動かない**(AE の pan-behind と同じ
/// 不変量)。`RKS` は正本 [`LayerPlacement::from_transform`]
/// (anchor=0, pos=0)で組む — 行列をここへ複製しない。
///
/// `M0` が退化(scale 0)して逆行列が立たないなら開始時の値をそのまま返す
/// (解けない drag は値を動かさない — [`GizmoDragState::begin`] の
/// 「掴めない物は掴めないまま」と同じ判断)。修飾キーは第1弾では持たない
/// (AE の pan-behind にも Shift の標準挙動は無い)。
pub fn anchor_value(start: &GizmoTarget, cursor_parent: Vec2) -> ([f64; 2], [f64; 2]) {
    let unchanged = (
        [start.anchor[0] as f64, start.anchor[1] as f64],
        [start.position[0] as f64, start.position[1] as f64],
    );
    let placement = LayerPlacement::from_transform(
        start.anchor,
        start.position,
        start.scale,
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    let local_from_parent = placement.inverse();
    if !local_from_parent.is_finite() {
        return unchanged;
    }
    let new_anchor = local_from_parent.transform_point2(cursor_parent);
    let rks = LayerPlacement::from_transform(
        [0.0, 0.0],
        [0.0, 0.0],
        start.scale,
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    let compensation =
        rks.transform_vector2(new_anchor - Vec2::new(start.anchor[0], start.anchor[1]));
    (
        [new_anchor.x as f64, new_anchor.y as f64],
        [
            (start.position[0] + compensation.x) as f64,
            (start.position[1] + compensation.y) as f64,
        ],
    )
}

/// 進行中の drag。**Document でも Session でもない widget 内だけの一時状態**
/// ([`crate::Interaction`] と同格)。開始時点の対象・座標系を凍結して持つ —
/// drag 中に shell が transient で対象を動かしても解が発振しない
/// (Inspector drag の `start_value` 凍結と同じ判断)。
#[derive(Debug, Clone, Copy)]
pub struct GizmoDragState {
    handle: GizmoHandle,
    start: GizmoTarget,
    parent_from_screen: Affine2,
    start_cursor_parent: Vec2,
    last_cursor: Point,
    moved: bool,
    last_value: Option<GizmoValue>,
}

impl GizmoDragState {
    /// press で drag を開始する。screen→親の逆行列が立たない(親鎖の scale 0 等)
    /// なら `None` — 解けない drag を始めない(掴めない物は掴めないまま)。
    pub fn begin(
        target: GizmoTarget,
        layout: &GizmoLayout,
        handle: GizmoHandle,
        cursor: Point,
    ) -> Option<Self> {
        let parent_from_screen = layout.screen_from_parent.inverse();
        if !parent_from_screen.is_finite() {
            return None;
        }
        let start_cursor_parent = parent_from_screen.transform_point2(Vec2::new(cursor.x, cursor.y));
        Some(Self {
            handle,
            start: target,
            parent_from_screen,
            start_cursor_parent,
            last_cursor: cursor,
            moved: false,
            last_value: None,
        })
    }

    pub fn layer(&self) -> LayerId {
        self.start.layer
    }

    pub fn handle(&self) -> GizmoHandle {
        self.handle
    }

    pub fn property(&self) -> GizmoProperty {
        self.handle.property()
    }

    /// 1回でも `update` が走ったか。release 時の Commit/Cancel(空クリック)判定。
    pub fn moved(&self) -> bool {
        self.moved
    }

    /// 直近の計算値。release は cursor 位置を運ばない(`ButtonReleased` に位置が
    /// 無い)ので、Commit はこれをそのまま使う(Inspector drag の `last_value` と
    /// 同じ持ち回し)。
    pub fn last_value(&self) -> Option<GizmoValue> {
        self.last_value
    }

    /// cursor 移動で新しい値を解く。
    pub fn update(&mut self, cursor: Point, shift: bool) -> GizmoValue {
        self.last_cursor = cursor;
        self.moved = true;
        let cursor_parent = self
            .parent_from_screen
            .transform_point2(Vec2::new(cursor.x, cursor.y));
        let value = match self.handle {
            GizmoHandle::Body => GizmoValue::Position(move_value(
                &self.start,
                self.start_cursor_parent,
                cursor_parent,
            )),
            GizmoHandle::Scale(handle) => {
                GizmoValue::Scale(scale_value(&self.start, handle, cursor_parent, shift))
            }
            GizmoHandle::Rotate => GizmoValue::Rotation(rotation_value(
                &self.start,
                self.start_cursor_parent,
                cursor_parent,
                shift,
            )),
            GizmoHandle::Anchor => {
                let (anchor, position) = anchor_value(&self.start, cursor_parent);
                GizmoValue::Anchor { anchor, position }
            }
        };
        self.last_value = Some(value);
        value
    }

    /// 修飾キーの途中変更(Shift の押し離し)を、cursor が動かなくても即時に
    /// 反映する(AE/Figma と同じ生の手触り)。まだ1回も動いていなければ何もしない
    /// (空クリックを Move に化けさせない)。
    pub fn refresh(&mut self, shift: bool) -> Option<GizmoValue> {
        self.moved.then(|| self.update(self.last_cursor, shift))
    }
}

// ---------------------------------------------------------------------------
// canvas::Program(翻訳だけ — StageOverlay と同じ規律)
// ---------------------------------------------------------------------------

/// ギズモの canvas 一式。`Shell::view` が選択レイヤーの [`GizmoTarget`] から
/// 毎フレーム組み直し、`stack!` で `StageOverlay` の**上**に重ねる(結線は
/// supervisor)。掴んでいない場所のイベントは capture しない —
/// ホイール/中ボタン(観測カメラ)や空クリックは下の層へ素通しする。
#[derive(Clone, Copy)]
pub struct GizmoOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: GizmoTarget,
    dims: Dimensions,
    colors: Colors,
}

impl GizmoOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        target: GizmoTarget,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            target,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> iced::Element<'a, GizmoDrag> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn layout(&self, bounds: Rectangle) -> Option<GizmoLayout> {
        gizmo_layout(
            bounds,
            self.comp,
            self.render_camera,
            self.observation,
            &self.target,
            self.dims,
        )
    }

    fn message(&self, phase: GizmoPhase) -> GizmoDrag {
        GizmoDrag {
            layer: self.target.layer,
            phase,
        }
    }
}

/// canvas の一時状態(drag と Shift の追跡)。
#[derive(Default)]
pub struct GizmoInteraction {
    drag: Option<GizmoDragState>,
    shift: bool,
}

impl canvas::Program<GizmoDrag> for GizmoOverlay {
    type State = GizmoInteraction;

    fn update(
        &self,
        state: &mut GizmoInteraction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<GizmoDrag>> {
        match event {
            canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift = modifiers.shift();
                // drag 中なら Shift の途中切り替えを即時反映(capture はしない —
                // 修飾キーは他 widget も見る)。
                let drag = state.drag.as_mut()?;
                let value = drag.refresh(state.shift)?;
                Some(canvas::Action::publish(
                    self.message(GizmoPhase::Move { value }),
                ))
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                // Esc = キャンセル(drag 中だけ食う — それ以外は素通し)。
                state.drag.take()?;
                Some(canvas::Action::publish(self.message(GizmoPhase::Cancel)).and_capture())
            }
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let position = cursor.position_in(bounds)?;
                    let layout = self.layout(bounds)?;
                    let handle = gizmo_hit_test(&layout, position)?;
                    let drag = GizmoDragState::begin(self.target, &layout, handle, position)?;
                    state.drag = Some(drag);
                    Some(
                        canvas::Action::publish(self.message(GizmoPhase::Start {
                            property: handle.property(),
                        }))
                        .and_capture(),
                    )
                }
                mouse::Event::CursorMoved { .. } => {
                    let drag = state.drag.as_mut()?;
                    // drag 中は bounds の外へ出ても続く(絶対座標からローカルへ
                    // 自前で戻す — `position_in` は外に出た瞬間 `None` になる)。
                    let absolute = cursor.position()?;
                    let position = Point::new(absolute.x - bounds.x, absolute.y - bounds.y);
                    let value = drag.update(position, state.shift);
                    Some(
                        canvas::Action::publish(self.message(GizmoPhase::Move { value }))
                            .and_capture(),
                    )
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let drag = state.drag.take()?;
                    let phase = match drag.last_value() {
                        // 契約: Start は必ず Commit か Cancel で閉じる(空クリック=
                        // 動いていない= Cancel、値を書かない)。
                        Some(value) if drag.moved() => GizmoPhase::Commit { value },
                        _ => GizmoPhase::Cancel,
                    };
                    Some(canvas::Action::publish(self.message(phase)).and_capture())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// bbox(accent hairline)+8ハンドル+回転ハンドル(stem+円)+anchor ⊕。
    /// 全部 [`GizmoLayout`] の座標をそのまま描く — hit-test と同じ正本(Q0)。
    fn draw(
        &self,
        _state: &GizmoInteraction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let Some(layout) = self.layout(bounds) else {
            return Vec::new();
        };
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let accent = self.colors.action_active;
        let handle_fill = self.colors.surface_raised;
        let hairline = canvas::Stroke::default()
            .with_color(accent)
            .with_width(self.dims.border_width);

        // 選択の器(bbox)。
        let bbox = canvas::Path::new(|builder| {
            builder.move_to(layout.corners[0]);
            for corner in &layout.corners[1..] {
                builder.line_to(*corner);
            }
            builder.close();
        });
        frame.stroke(&bbox, hairline.clone());

        // 回転ハンドル(stem を先に描き、ハンドル群を上へ)。
        if let Some(rotate) = layout.rotate_handle {
            let stem = canvas::Path::line(layout.top_center, rotate);
            frame.stroke(&stem, hairline.clone());
            let knob = canvas::Path::circle(rotate, self.dims.gizmo_handle_size * 0.5);
            frame.fill(&knob, handle_fill);
            frame.stroke(&knob, hairline.clone());
        }

        // 8ハンドル(正方形、中心合わせ)。
        let handle_size = self.dims.gizmo_handle_size;
        for point in layout.scale_handles {
            let square = canvas::Path::rectangle(
                Point::new(point.x - handle_size * 0.5, point.y - handle_size * 0.5),
                iced::Size::new(handle_size, handle_size),
            );
            frame.fill(&square, handle_fill);
            frame.stroke(&square, hairline.clone());
        }

        // anchor ⊕(AE の慣習形。第2切片から drag 対象 — 命中は hit_radius、
        // 見た目の寸はトークンのまま: 変形の不動点はハンドルより一段軽い視覚重量)。
        let radius = self.dims.gizmo_anchor_radius;
        let ring = canvas::Path::circle(layout.anchor, radius);
        frame.stroke(&ring, hairline.clone());
        let cross_horizontal = canvas::Path::line(
            Point::new(layout.anchor.x - radius * 2.0, layout.anchor.y),
            Point::new(layout.anchor.x + radius * 2.0, layout.anchor.y),
        );
        let cross_vertical = canvas::Path::line(
            Point::new(layout.anchor.x, layout.anchor.y - radius * 2.0),
            Point::new(layout.anchor.x, layout.anchor.y + radius * 2.0),
        );
        frame.stroke(&cross_horizontal, hairline.clone());
        frame.stroke(&cross_vertical, hairline);

        vec![frame.into_geometry()]
    }

    /// hover/drag のカーソル形状(Q0「触れそう」の正直な合図)。ハンドルの
    /// リサイズ向きは回転済みの screen 位置から出す([`resize_interaction`])。
    fn mouse_interaction(
        &self,
        state: &GizmoInteraction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let interaction_for = |handle: GizmoHandle, layout: &GizmoLayout, dragging: bool| {
            match handle {
                GizmoHandle::Body => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Move
                    }
                }
                GizmoHandle::Rotate => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Grab
                    }
                }
                // ⊕ グリフと同形の Crosshair(hover)— 「精密に置く物」の合図。
                // 第1切片の「表示のみ」からの繰り上がりを、カーソルが最初に語る
                // (Q0: 触れる物は触れそうに見せる)。
                GizmoHandle::Anchor => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Crosshair
                    }
                }
                GizmoHandle::Scale(scale_handle) => {
                    let index = SCALE_HANDLES
                        .iter()
                        .position(|h| *h == scale_handle)
                        .unwrap_or(0);
                    let point = layout.scale_handles[index];
                    let center_x =
                        layout.corners.iter().map(|c| c.x).sum::<f32>() / layout.corners.len() as f32;
                    let center_y =
                        layout.corners.iter().map(|c| c.y).sum::<f32>() / layout.corners.len() as f32;
                    resize_interaction(Vec2::new(point.x - center_x, point.y - center_y))
                }
            }
        };

        if let Some(drag) = &state.drag {
            if let Some(layout) = self.layout(bounds) {
                return interaction_for(drag.handle(), &layout, true);
            }
            return mouse::Interaction::Grabbing;
        }
        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let Some(layout) = self.layout(bounds) else {
            return mouse::Interaction::default();
        };
        match gizmo_hit_test(&layout, position) {
            Some(handle) => interaction_for(handle, &layout, false),
            None => mouse::Interaction::default(),
        }
    }
}

/// bbox 中心→ハンドルの screen 方向から、リサイズカーソルの4向きを選ぶ
/// (45° 扇形で最寄りの軸へ丸める — 回転したレイヤーでも視覚的に正しい向き)。
/// `ResizingDiagonallyDown` = NW–SE 軸(↘)、`ResizingDiagonallyUp` = NE–SW 軸(↗)
/// (iced→winit の CursorIcon 対応どおり)。
pub fn resize_interaction(direction: Vec2) -> mouse::Interaction {
    if direction.length_squared() < SOLVE_EPS * SOLVE_EPS {
        return mouse::Interaction::default();
    }
    let mut angle = direction.y.atan2(direction.x).to_degrees();
    // 軸としては 180° 対称なので [0, 180) へ畳む。
    if angle < 0.0 {
        angle += 180.0;
    }
    if !(22.5..157.5).contains(&angle) {
        mouse::Interaction::ResizingHorizontally
    } else if angle < 67.5 {
        mouse::Interaction::ResizingDiagonallyDown
    } else if angle < 112.5 {
        mouse::Interaction::ResizingVertically
    } else {
        mouse::Interaction::ResizingDiagonallyUp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{
        Composition, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerMeta,
        LayerSource, LayerTiming,
    };

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    fn bounds() -> Rectangle {
        Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0))
    }

    /// 100x50 の層を comp 中央へ、anchor は内容中央。bbox は (270,155)-(370,205)。
    fn target() -> GizmoTarget {
        GizmoTarget {
            layer: LayerId(1),
            size: [100.0, 50.0],
            anchor: [50.0, 25.0],
            position: [320.0, 180.0],
            scale: [1.0, 1.0],
            rotation_degrees: 0.0,
            skew_degrees: 0.0,
            skew_axis_degrees: 0.0,
            world_from_parent: Affine2::IDENTITY,
        }
    }

    fn dims() -> Dimensions {
        Dimensions::default()
    }

    fn layout_with(observation: Option<ObservationCamera>, target: &GizmoTarget) -> GizmoLayout {
        gizmo_layout(
            bounds(),
            COMP,
            ResolvedCamera::default(),
            observation,
            target,
            dims(),
        )
        .expect("layout が組めるはず")
    }

    fn approx_point(actual: Point, expected: (f32, f32)) {
        assert!(
            (actual.x - expected.0).abs() < 1e-2 && (actual.y - expected.1).abs() < 1e-2,
            "{actual:?} != {expected:?}"
        );
    }

    /// 値の照合(f32 の逆行列を経由するので機械精度ではなく 1e-3 の帯で見る —
    /// screen px/scale/度のどの単位でも視認できない差)。
    fn approx_value(actual: GizmoValue, expected: GizmoValue) {
        let vec2_close =
            |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        let ok = match (actual, expected) {
            (GizmoValue::Position(a), GizmoValue::Position(b))
            | (GizmoValue::Scale(a), GizmoValue::Scale(b)) => vec2_close(a, b),
            (GizmoValue::Rotation(a), GizmoValue::Rotation(b)) => (a - b).abs() < 1e-3,
            (
                GizmoValue::Anchor {
                    anchor: a1,
                    position: p1,
                },
                GizmoValue::Anchor {
                    anchor: a2,
                    position: p2,
                },
            ) => vec2_close(a1, a2) && vec2_close(p1, p2),
            _ => false,
        };
        assert!(ok, "{actual:?} != {expected:?}");
    }

    // -----------------------------------------------------------------
    // layout(カメラ変換下の座標 — 発注 ORACLE)
    // -----------------------------------------------------------------

    /// 既定カメラ・恒等 letterbox なら bbox は層の comp 矩形そのもの。
    #[test]
    fn layout_puts_the_bbox_at_the_layers_comp_rect() {
        let layout = layout_with(None, &target());
        approx_point(layout.corners[0], (270.0, 155.0));
        approx_point(layout.corners[1], (370.0, 155.0));
        approx_point(layout.corners[2], (370.0, 205.0));
        approx_point(layout.corners[3], (270.0, 205.0));
        approx_point(layout.anchor, (320.0, 180.0));
        // 辺中点ハンドル(Top)は上辺の中央。
        let top_index = SCALE_HANDLES.iter().position(|h| *h == ScaleHandle::Top).unwrap();
        approx_point(layout.scale_handles[top_index], (320.0, 155.0));
    }

    /// **観測カメラ下の本命**: zoom 2(comp 中心アンカー)で bbox の角が
    /// `camera_screen_from_world_z0` の写像どおりに動く(座標変換は既存関数の
    /// 合成 — 独自の投影をしていないことの実測)。
    #[test]
    fn layout_follows_the_observation_camera() {
        let observation = ObservationCamera {
            pan: [10.0, -20.0],
            zoom: 2.0,
        };
        let layout = layout_with(Some(observation), &target());

        // 期待値は既存の正本写像で独立に計算する。
        let affine = camera_screen_from_world_z0(
            COMP,
            ResolvedCamera {
                center: observation.pan,
                zoom: observation.zoom,
                roll_degrees: 0.0,
            },
        );
        let expected = affine.transform_point2(Vec2::new(270.0, 155.0));
        approx_point(layout.corners[0], (expected.x, expected.y));
        let expected_anchor = affine.transform_point2(Vec2::new(320.0, 180.0));
        approx_point(layout.anchor, (expected_anchor.x, expected_anchor.y));
    }

    /// 回転ハンドルは上辺中点の外側 `gizmo_rotate_offset` px。
    #[test]
    fn layout_places_the_rotate_handle_outside_the_top_edge() {
        let layout = layout_with(None, &target());
        let offset = dims().gizmo_rotate_offset;
        approx_point(layout.top_center, (320.0, 155.0));
        approx_point(
            layout.rotate_handle.expect("回転ハンドルが出るはず"),
            (320.0, 155.0 - offset),
        );
    }

    /// 垂直反転(scale.y < 0)しても回転ハンドルは「上辺の外側」へついてくる
    /// (screen では下向きに出る)。
    #[test]
    fn layout_rotate_handle_follows_a_flipped_layer() {
        let mut flipped = target();
        flipped.scale = [1.0, -1.0];
        let layout = layout_with(None, &flipped);
        let top = layout.top_center;
        let rotate = layout.rotate_handle.expect("回転ハンドルが出るはず");
        // 反転で上辺中点は中心より下(y=205)に写る — ハンドルはさらに外(下)。
        approx_point(top, (320.0, 205.0));
        assert!(
            rotate.y > top.y,
            "反転時は外側=下へ出るはず: rotate={rotate:?} top={top:?}"
        );
    }

    // -----------------------------------------------------------------
    // hit-test(命中と優先順位 — 発注 ORACLE)
    // -----------------------------------------------------------------

    #[test]
    fn hit_test_finds_each_handle_kind() {
        let layout = layout_with(None, &target());
        // 角。
        assert_eq!(
            gizmo_hit_test(&layout, Point::new(270.0, 155.0)),
            Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
        );
        // 辺中点。
        assert_eq!(
            gizmo_hit_test(&layout, Point::new(370.0, 180.0)),
            Some(GizmoHandle::Scale(ScaleHandle::Right))
        );
        // 回転ハンドル。
        let rotate = layout.rotate_handle.unwrap();
        assert_eq!(gizmo_hit_test(&layout, rotate), Some(GizmoHandle::Rotate));
        // anchor ⊕(第2切片から drag 対象)。
        assert_eq!(
            gizmo_hit_test(&layout, layout.anchor),
            Some(GizmoHandle::Anchor)
        );
        // 内部 = body(anchor の hit_radius の外)。
        assert_eq!(
            gizmo_hit_test(&layout, Point::new(300.0, 180.0)),
            Some(GizmoHandle::Body)
        );
        // 全部の外 = None(下の層へ素通し)。
        assert_eq!(gizmo_hit_test(&layout, Point::new(100.0, 60.0)), None);
    }

    /// ハンドルは body に優先する(ハンドル位置は bbox 縁 = body でもあるが、
    /// 掴めるのは小さい方)。
    #[test]
    fn hit_test_prefers_handles_over_the_body() {
        let layout = layout_with(None, &target());
        // 角のすぐ内側(半径内): body ではなく角。
        assert_eq!(
            gizmo_hit_test(&layout, Point::new(272.0, 157.0)),
            Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
        );
    }

    /// レイヤーが小さく潰れてハンドル同士が重なったら、角が辺に勝つ
    /// (2軸動かせる方を残す)。
    #[test]
    fn hit_test_prefers_corners_over_edges_when_handles_overlap() {
        let mut tiny = target();
        // 6x6 の極小層: 角と辺中点の距離は 3px < hit_radius(8)。
        tiny.size = [6.0, 6.0];
        tiny.anchor = [3.0, 3.0];
        let layout = layout_with(None, &tiny);
        let top_left_index = SCALE_HANDLES
            .iter()
            .position(|h| *h == ScaleHandle::TopLeft)
            .unwrap();
        let at_corner = layout.scale_handles[top_left_index];
        let hit = gizmo_hit_test(&layout, at_corner);
        assert_eq!(hit, Some(GizmoHandle::Scale(ScaleHandle::TopLeft)));
    }

    /// 回転ハンドルは重なった scale ハンドルより優先(外側の小さい物が勝つ)。
    #[test]
    fn hit_test_prefers_the_rotate_handle_over_scale_handles() {
        let layout = layout_with(None, &target());
        let rotate = layout.rotate_handle.unwrap();
        // 回転ハンドルと Top ハンドルの中間(両方の半径内になりうる点)でも
        // 回転ハンドル半径内なら Rotate。
        let probe = Point::new(rotate.x, rotate.y + layout.hit_radius * 0.9);
        assert_eq!(gizmo_hit_test(&layout, probe), Some(GizmoHandle::Rotate));
    }

    // -----------------------------------------------------------------
    // move(カメラ/親変換下の値 — 発注 ORACLE)
    // -----------------------------------------------------------------

    #[test]
    fn body_drag_moves_position_by_the_screen_delta_under_default_cameras() {
        let target = target();
        let layout = layout_with(None, &target);
        let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(300.0, 170.0))
            .expect("drag が始まるはず");
        assert!(!drag.moved());
        let value = drag.update(Point::new(310.0, 175.0), false);
        approx_value(value, GizmoValue::Position([330.0, 185.0]));
        assert!(drag.moved());
        assert_eq!(drag.last_value(), Some(value));
    }

    /// 観測カメラ zoom 2 の下では screen 10px = world 5px(絵の上で掴んだ場所に
    /// カーソルがついてくる)。
    #[test]
    fn body_drag_compensates_for_observation_zoom() {
        let observation = ObservationCamera {
            pan: [0.0, 0.0],
            zoom: 2.0,
        };
        let target = target();
        let layout = layout_with(Some(observation), &target);
        let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(320.0, 180.0))
            .expect("drag が始まるはず");
        let value = drag.update(Point::new(330.0, 185.0), false);
        approx_value(value, GizmoValue::Position([325.0, 182.5]));
    }

    /// 親が2倍に拡大している時、world 10px の変位は親空間では 5px —
    /// position は親空間の値なので 5 だけ動く。
    #[test]
    fn body_drag_solves_in_parent_space() {
        let mut parented = target();
        parented.world_from_parent = Affine2::from_scale(Vec2::new(2.0, 2.0));
        // 親の2倍で screen 上の絵は position*2 に写るが、drag の解は親空間で立つ。
        let layout = layout_with(None, &parented);
        let start = layout.anchor; // anchor の screen 位置から掴む
        let mut drag = GizmoDragState::begin(parented, &layout, GizmoHandle::Body, start)
            .expect("drag が始まるはず");
        let value = drag.update(Point::new(start.x + 10.0, start.y + 5.0), false);
        approx_value(value, GizmoValue::Position([325.0, 182.5]));
    }

    // -----------------------------------------------------------------
    // scale(角=2軸・辺=1軸・Shift=比率固定 — map 680)
    // -----------------------------------------------------------------

    #[test]
    fn corner_drag_scales_both_axes_about_the_anchor() {
        let target = target();
        let layout = layout_with(None, &target);
        let corner = Point::new(370.0, 205.0); // BottomRight
        let mut drag =
            GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
                .expect("drag が始まるはず");
        // 1.5倍の位置へ: anchor(320,180) + (75, 37.5)。
        let value = drag.update(Point::new(395.0, 217.5), false);
        approx_value(value, GizmoValue::Scale([1.5, 1.5]));
        // 非等方: x だけ 1.5 倍の位置。
        let value = drag.update(Point::new(395.0, 205.0), false);
        approx_value(value, GizmoValue::Scale([1.5, 1.0]));
    }

    /// Shift = 比率固定: 変化の大きい軸の率を両軸へ(map 680)。
    #[test]
    fn corner_drag_with_shift_keeps_the_aspect_ratio() {
        let target = target();
        let layout = layout_with(None, &target);
        let corner = Point::new(370.0, 205.0);
        let mut drag =
            GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
                .expect("drag が始まるはず");
        let value = drag.update(Point::new(395.0, 205.0), true);
        approx_value(value, GizmoValue::Scale([1.5, 1.5]));
    }

    /// 辺ハンドルは1軸だけ。y の分母(handle-anchor)が 0 でも x が解ける。
    #[test]
    fn edge_drag_scales_only_its_axis() {
        let target = target();
        let layout = layout_with(None, &target);
        let right = Point::new(370.0, 180.0);
        let mut drag =
            GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::Right), right)
                .expect("drag が始まるはず");
        // カーソルを右+下へ: y 成分は無視され x だけ 1.5。
        let value = drag.update(Point::new(395.0, 190.0), false);
        approx_value(value, GizmoValue::Scale([1.5, 1.0]));
        // Shift: 解けた1軸の率を両軸へ。
        let value = drag.update(Point::new(395.0, 190.0), true);
        approx_value(value, GizmoValue::Scale([1.5, 1.5]));
    }

    /// ハンドルを anchor の反対側へ引き抜くと負 scale(反転、AE と同じ)。
    #[test]
    fn dragging_through_the_anchor_flips_the_scale() {
        let target = target();
        let layout = layout_with(None, &target);
        let right = Point::new(370.0, 180.0);
        let mut drag =
            GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::Right), right)
                .expect("drag が始まるはず");
        let value = drag.update(Point::new(270.0, 180.0), false);
        approx_value(value, GizmoValue::Scale([-1.0, 1.0]));
    }

    /// 回転している層でも、掴んだハンドルがカーソルへ来る scale が解ける
    /// (解は (R·K)⁻¹ 経由 — screen 位置は回転込みで一致する)。
    #[test]
    fn corner_drag_solves_under_layer_rotation() {
        let mut rotated = target();
        rotated.rotation_degrees = 90.0;
        let layout = layout_with(None, &rotated);
        let index = SCALE_HANDLES
            .iter()
            .position(|h| *h == ScaleHandle::BottomRight)
            .unwrap();
        let corner = layout.scale_handles[index];
        let mut drag =
            GizmoDragState::begin(rotated, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
                .expect("drag が始まるはず");
        // 90°回転で local (50,25)(anchor→BR)は親空間 (-25,50) に写る。2倍の位置へ。
        let value = drag.update(Point::new(320.0 - 50.0, 180.0 + 100.0), false);
        approx_value(value, GizmoValue::Scale([2.0, 2.0]));
    }

    // -----------------------------------------------------------------
    // rotate(anchor 中心・Shift=15°スナップ — map 679)
    // -----------------------------------------------------------------

    /// 上(12時)から右(3時)へ回すと +90°(y-down で時計回り = store の符号)。
    #[test]
    fn rotate_drag_measures_the_clockwise_angle_about_the_anchor() {
        let target = target();
        let layout = layout_with(None, &target);
        let rotate = layout.rotate_handle.unwrap();
        let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Rotate, rotate)
            .expect("drag が始まるはず");
        let value = drag.update(Point::new(320.0 + 41.0, 180.0), false);
        let GizmoValue::Rotation(degrees) = value else {
            panic!("Rotation が出るはず: {value:?}");
        };
        assert!((degrees - 90.0).abs() < 1e-3, "degrees={degrees}");
    }

    /// Shift = 最寄りの 15° 刻みへスナップ(map 679)。40° 相当 → 45°。
    #[test]
    fn rotate_drag_with_shift_snaps_to_15_degree_steps() {
        let target = target();
        let pivot = Vec2::new(320.0, 180.0);
        let v0 = Vec2::new(0.0, -41.0); // 真上
        let angle = (-90.0f32 + 40.0).to_radians();
        let v1 = Vec2::new(angle.cos(), angle.sin()) * 41.0;
        let degrees = rotation_value(&target, pivot + v0, pivot + v1, true);
        assert!((degrees - 45.0).abs() < 1e-3, "degrees={degrees}");
    }

    /// 観測カメラ(ズーム/パン)の下でも回転の値は変わらない — 角度は親空間で
    /// 測る(発注 ORACLE「カメラ変換下の座標」)。
    #[test]
    fn rotate_drag_is_independent_of_the_observation_camera() {
        let observation = ObservationCamera {
            pan: [37.0, -12.0],
            zoom: 1.7,
        };
        let target = target();
        let plain = layout_with(None, &target);
        let observed = layout_with(Some(observation), &target);

        // 同じ親空間の2点を、それぞれの screen 系へ写してから drag に食わせる。
        let parent_start = Vec2::new(320.0, 139.0);
        let parent_end = Vec2::new(361.0, 180.0);
        let run = |layout: &GizmoLayout| -> f64 {
            let to_screen = |p: Vec2| {
                let s = layout.screen_from_parent.transform_point2(p);
                Point::new(s.x, s.y)
            };
            let mut drag =
                GizmoDragState::begin(target, layout, GizmoHandle::Rotate, to_screen(parent_start))
                    .expect("drag が始まるはず");
            let GizmoValue::Rotation(degrees) = drag.update(to_screen(parent_end), false) else {
                panic!("Rotation が出るはず");
            };
            degrees
        };
        let a = run(&plain);
        let b = run(&observed);
        assert!((a - b).abs() < 1e-3, "カメラで回転値が変わっている: {a} vs {b}");
    }

    // -----------------------------------------------------------------
    // drag 状態機械(Start/Move/Commit/Cancel の材料)
    // -----------------------------------------------------------------

    /// 空クリック(動かさず release)は Commit の材料を持たない — canvas 側は
    /// Cancel を出す(契約: Start は必ず Commit/Cancel で閉じる)。
    #[test]
    fn a_click_without_movement_has_no_value_to_commit() {
        let target = target();
        let layout = layout_with(None, &target);
        let drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(300.0, 180.0))
            .expect("drag が始まるはず");
        assert!(!drag.moved());
        assert_eq!(drag.last_value(), None);
    }

    /// Shift の途中切り替え(refresh)は動いた後だけ効く。
    #[test]
    fn refresh_reapplies_modifiers_only_after_movement() {
        let target = target();
        let layout = layout_with(None, &target);
        let corner = Point::new(370.0, 205.0);
        let mut drag =
            GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
                .expect("drag が始まるはず");
        assert_eq!(drag.refresh(true), None, "動く前の refresh は no-op のはず");
        let _ = drag.update(Point::new(395.0, 205.0), false);
        let refreshed = drag.refresh(true).expect("Shift on で比率固定へ再計算されるはず");
        approx_value(refreshed, GizmoValue::Scale([1.5, 1.5]));
    }

    /// handle → property の対応(shell 結線の宛先)。
    #[test]
    fn handles_map_to_their_properties() {
        assert_eq!(GizmoHandle::Body.property(), GizmoProperty::Position);
        assert_eq!(
            GizmoHandle::Scale(ScaleHandle::Top).property(),
            GizmoProperty::Scale
        );
        assert_eq!(GizmoHandle::Rotate.property(), GizmoProperty::Rotation);
        assert_eq!(GizmoHandle::Anchor.property(), GizmoProperty::Anchor);
        assert_eq!(GizmoProperty::Position.property_name(), property::POSITION);
        assert_eq!(GizmoProperty::Scale.property_name(), property::SCALE);
        assert_eq!(GizmoProperty::Rotation.property_name(), property::ROTATION);
        assert_eq!(GizmoProperty::Anchor.property_name(), property::ANCHOR);
        assert_eq!(
            GizmoValue::Anchor {
                anchor: [0.0, 0.0],
                position: [0.0, 0.0]
            }
            .property(),
            GizmoProperty::Anchor
        );
    }

    // -----------------------------------------------------------------
    // anchor drag(第2切片 — AE pan-behind 型。未実行、波末一括)
    // -----------------------------------------------------------------

    /// **本命**: ⊕ が cursor の真下へ来て、position が補償される
    /// (恒等変形なら anchor の移動量=position の移動量)。
    #[test]
    fn anchor_drag_lands_the_anchor_under_the_cursor_and_compensates_position() {
        let target = target();
        let (anchor, position) = anchor_value(&target, Vec2::new(330.0, 190.0));
        // M0⁻¹(330,190) = (330,190) - (270,155) = (60,35)。
        assert!((anchor[0] - 60.0).abs() < 1e-3 && (anchor[1] - 35.0).abs() < 1e-3);
        // RKS = 恒等なので補償は anchor の差分そのまま: (320,180)+(10,10)。
        assert!((position[0] - 330.0).abs() < 1e-3 && (position[1] - 190.0).abs() < 1e-3);
    }

    /// **不変量**: 回転+非等方 scale の下でも、anchor drag の前後で任意の
    /// 局所点の親空間の像が動かない(見た目不動 — AE pan-behind の本体)。
    #[test]
    fn anchor_drag_keeps_the_image_stationary_under_rotation_and_scale() {
        let mut twisted = target();
        twisted.rotation_degrees = 30.0;
        twisted.scale = [2.0, 0.5];
        let cursor_parent = Vec2::new(300.0, 200.0);
        let (anchor, position) = anchor_value(&twisted, cursor_parent);

        let before = LayerPlacement::from_transform(
            twisted.anchor,
            twisted.position,
            twisted.scale,
            twisted.rotation_degrees,
            twisted.skew_degrees,
            twisted.skew_axis_degrees,
        );
        let after = LayerPlacement::from_transform(
            [anchor[0] as f32, anchor[1] as f32],
            [position[0] as f32, position[1] as f32],
            twisted.scale,
            twisted.rotation_degrees,
            twisted.skew_degrees,
            twisted.skew_axis_degrees,
        );
        for probe in [Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0), Vec2::new(37.0, 11.0)] {
            let b = before.transform_point2(probe);
            let a = after.transform_point2(probe);
            assert!(
                (a.x - b.x).abs() < 1e-2 && (a.y - b.y).abs() < 1e-2,
                "絵が動いた: probe={probe:?} before={b:?} after={a:?}"
            );
        }
        // ⊕ 自身は cursor の真下(M1(a1) = cursor_parent)。
        let landed = after.transform_point2(Vec2::new(anchor[0] as f32, anchor[1] as f32));
        assert!(
            (landed.x - cursor_parent.x).abs() < 1e-2
                && (landed.y - cursor_parent.y).abs() < 1e-2,
            "⊕ が cursor に吸い付いていない: {landed:?}"
        );
    }

    /// 観測カメラ zoom 2 の下でも drag 経路(screen → 親空間)ごと正しく解ける
    /// (screen 10px = 親空間 5px)。
    #[test]
    fn anchor_drag_solves_under_the_observation_camera() {
        let observation = ObservationCamera {
            pan: [0.0, 0.0],
            zoom: 2.0,
        };
        let target = target();
        let layout = layout_with(Some(observation), &target);
        let start = layout.anchor;
        let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Anchor, start)
            .expect("drag が始まるはず");
        let value = drag.update(Point::new(start.x + 10.0, start.y + 5.0), false);
        approx_value(
            value,
            GizmoValue::Anchor {
                anchor: [55.0, 27.5],
                position: [325.0, 182.5],
            },
        );
    }

    /// scale 0 で写像が潰れている層は解かない — 開始時の値を返すだけ
    /// (`begin` の「掴めない物は掴めないまま」と同じ判断)。
    #[test]
    fn anchor_drag_with_a_degenerate_scale_keeps_the_start_values() {
        let mut flat = target();
        flat.scale = [0.0, 1.0];
        let (anchor, position) = anchor_value(&flat, Vec2::new(400.0, 300.0));
        assert_eq!(anchor, [50.0, 25.0]);
        assert_eq!(position, [320.0, 180.0]);
    }

    /// 優先順位: 極小レイヤーで anchor が scale ハンドルに飲まれても角が勝つ
    /// (2軸動かせる方を残す — 既存の角>辺と同じ原則)。逆に body とは
    /// anchor が勝つ(小さい物が大きい物に勝つ)。
    #[test]
    fn hit_test_ranks_the_anchor_between_scale_handles_and_the_body() {
        // 通常寸: anchor(中央)は body の海の中 — anchor が勝つ。
        let layout = layout_with(None, &target());
        assert_eq!(
            gizmo_hit_test(&layout, Point::new(layout.anchor.x + 3.0, layout.anchor.y)),
            Some(GizmoHandle::Anchor)
        );

        // 極小 6x6: 角ハンドルと anchor が半径内で重なる — 角が勝つ。
        let mut tiny = target();
        tiny.size = [6.0, 6.0];
        tiny.anchor = [3.0, 3.0];
        let layout = layout_with(None, &tiny);
        let top_left_index = SCALE_HANDLES
            .iter()
            .position(|h| *h == ScaleHandle::TopLeft)
            .unwrap();
        assert_eq!(
            gizmo_hit_test(&layout, layout.scale_handles[top_left_index]),
            Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
        );
    }

    // -----------------------------------------------------------------
    // カーソル形状(Q0 の合図)
    // -----------------------------------------------------------------

    #[test]
    fn resize_interaction_picks_the_axis_by_direction() {
        use mouse::Interaction;
        assert_eq!(resize_interaction(Vec2::new(1.0, 0.0)), Interaction::ResizingHorizontally);
        assert_eq!(resize_interaction(Vec2::new(-1.0, 0.0)), Interaction::ResizingHorizontally);
        assert_eq!(resize_interaction(Vec2::new(0.0, 1.0)), Interaction::ResizingVertically);
        // ↘(右下)= NW–SE 軸。
        assert_eq!(
            resize_interaction(Vec2::new(1.0, 1.0)),
            Interaction::ResizingDiagonallyDown
        );
        // ↗(右上)= NE–SW 軸。
        assert_eq!(
            resize_interaction(Vec2::new(1.0, -1.0)),
            Interaction::ResizingDiagonallyUp
        );
    }

    // -----------------------------------------------------------------
    // gizmo_target(StoreView からの読み口)
    // -----------------------------------------------------------------

    fn still(value: Value) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value,
            interp: Interp::Hold,
            spatial: None,
        });
        track
    }

    fn doc_with_layer() -> (Document, LayerId) {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: motolii_core::Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [255, 0, 0, 255],
                        width: 100,
                        height: 50,
                    },
                    order: 0,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
        doc.apply(Intent::SetTrack {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            track: still(Value::Vec2([320.0, 180.0])),
        })
        .unwrap();
        doc.apply(Intent::SetTrack {
            layer,
            property: PropertyId::new(property::ANCHOR).unwrap(),
            track: still(Value::Vec2([50.0, 25.0])),
        })
        .unwrap();
        doc.apply(Intent::SetTrack {
            layer,
            property: PropertyId::new(property::ROTATION).unwrap(),
            track: still(Value::F64(30.0)),
        })
        .unwrap();
        (doc, layer)
    }

    /// 実 Document から局所値が読める(Inspector と同じ値の経路)。
    #[test]
    fn gizmo_target_reads_the_layers_transform_values() {
        let (doc, layer) = doc_with_layer();
        let view = doc.view();
        let target = gizmo_target(&view, layer, 0, [100.0, 50.0]).expect("target が組めるはず");
        assert_eq!(target.position, [320.0, 180.0]);
        assert_eq!(target.anchor, [50.0, 25.0]);
        assert_eq!(target.rotation_degrees, 30.0);
        assert_eq!(target.scale, [1.0, 1.0], "track が無い property は既定値");
        assert_eq!(target.world_from_parent, Affine2::IDENTITY, "親なし = 恒等");
    }

    /// 親を持つ層は親の local 変換が world_from_parent に入る(裁定173 H1 —
    /// 親が動くと子のギズモも動く)。
    #[test]
    fn gizmo_target_composes_the_parent_chain() {
        let (mut doc, layer) = doc_with_layer();
        let parent = LayerId(2);
        doc.apply_all([
            Intent::AddLayer(parent),
            Intent::SetMeta {
                layer: parent,
                meta: LayerMeta {
                    source: LayerSource::Null,
                    order: 1,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
        doc.apply(Intent::SetTrack {
            layer: parent,
            property: PropertyId::new(property::POSITION).unwrap(),
            track: still(Value::Vec2([40.0, 10.0])),
        })
        .unwrap();
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                parent: Some(Some(parent)),
                ..Default::default()
            },
        })
        .unwrap();

        let view = doc.view();
        let target = gizmo_target(&view, layer, 0, [100.0, 50.0]).expect("target が組めるはず");
        let moved = target
            .world_from_parent
            .transform_point2(Vec2::new(0.0, 0.0));
        assert!(
            (moved.x - 40.0).abs() < 1e-4 && (moved.y - 10.0).abs() < 1e-4,
            "親の平行移動が合成されていない: {moved:?}"
        );
    }

    /// hidden な層はギズモを出さない(Q0: 触れない物を描かない)。
    #[test]
    fn gizmo_target_is_none_for_a_hidden_layer() {
        let (mut doc, layer) = doc_with_layer();
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                hidden: Some(true),
                ..Default::default()
            },
        })
        .unwrap();
        let view = doc.view();
        assert!(gizmo_target(&view, layer, 0, [100.0, 50.0]).is_none());
    }
}
