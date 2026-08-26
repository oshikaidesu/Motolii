//! Stage 矩形選択(マーキー)— B31 キャンバス側第1切片(発注 2026-08-22)。
//! クリック選択・矩形(マーキー)選択・Shift 追加/トグル・空クリック解除。
//!
//! ## map 消化(`next/reference/normal-map.tsv` bundle B31、freq 降順)
//!
//! - map 444「Clear Selection」・452「Deselect all layers」(共に freq 1・採用予定)
//!   → **空クリック/空マーキー = 選択解除**([`SelectLayers`] の `ids=[]` +
//!   `additive=false`)。メニュー側の 433「Deselect All」/436「Select All」
//!   (freq 3)は MB-0 で採用済 — ここは直接操作側の対。
//! - map 966「Toggle selection of layer by number」(freq 1・採用予定)のトグル
//!   意味論 → **Shift = 追加/トグル**([`apply_selection`] の `additive`)。
//!   番号入力そのものはキーボード束なので本切片外。
//! - クリック選択・矩形選択のジェスチャ自体は map に行が無い(map は
//!   shortcut/menu 表)— bundle 定義行(`intent-bundles.tsv` B31
//!   「select(scope, criterion) キャンバス/Timeline共通」)のキャンバス scope を
//!   ここで実装する。
//!
//! ## 意味(Document を一切書かない — gizmo.rs と同じ規律)
//!
//! 出力は pane ローカル message [`SelectLayers`] のみ。shell 側(結線は
//! supervisor)が `Session::selected_layers` へ写す — 現在選択とヒット集合の
//! 合成規則は [`apply_selection`](純関数)がこの crate 側で固定する
//! (`additive=false` = 置換、`additive=true` = ヒット各 id をトグル:
//! 未選択なら追加・選択済みなら解除。AE/Figma の Shift クリックと同じ意味)。
//!
//! ## 座標系(**GZ FINDING「bounds の pane 原点ずれ」を踏まない**)
//!
//! レイヤー bbox の screen 位置は自前で投影しない — [`gizmo_layout`] を呼んで
//! その `corners` をそのまま使う([`selection_quad`])。gizmo.rs は
//! `letterbox_screen_from_comp` で letterbox 矩形を **bounds 原点=0 に正規化**
//! してから組む(gizmo.rs 冒頭 doc「座標系」)ので、ここも自動的に同じ
//! ローカル系(`draw` の `Frame`・`cursor.position_in(bounds)` と同系)に揃う。
//! カメラ変換(観測カメラ/レンダリングカメラ・回転/skew/親鎖)下でも正しい。
//!
//! ## 優先順位(gizmo が勝つ — 純関数 [`press_starts_marquee`] で固定)
//!
//! gizmo のハンドル/回転ハンドル/body の上の press はマーキーを開始しない。
//! 判定は [`gizmo_hit_test`] の**再利用**(2つ目の hit 実装を作らない — 見えて
//! いる gizmo と触れる gizmo が同じ正本(Q0)のまま、こちらはその補集合)。
//! Shift+クリックで「選択済みレイヤーを外す」は gizmo が body を掴むため
//! ここへは届かない — 外す操作は Shift+マーキー(トグル)が受け持つ。
//!
//! ## テスト方針(lib.rs / gizmo.rs と同じ)
//!
//! 計算は全て純関数([`marquee_rect`]・[`rect_intersects_quad`]・
//! [`quad_contains_point`]・[`selection_quad`]・[`marquee_hit_ids`]・
//! [`click_hit_id`]・[`apply_selection`]・[`release_message`]・
//! [`press_starts_marquee`]・[`MarqueeDragState`])— `canvas::Program`
//! ([`MarqueeOverlay`])はこれらへ委譲するだけの薄い翻訳層。試験は純関数を
//! 直接呼ぶ(KNOWN「iced_test は canvas 不可視」)。

use glam::Vec2;
use iced::widget::canvas;
use iced::{keyboard, mouse, Point, Rectangle, Size};

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_store::LayerId;
use motolii_tokens_rs::{Colors, Dimensions};

use crate::gizmo::{gizmo_hit_test, gizmo_layout, GizmoLayout, GizmoTarget};

/// クリックとマーキーの分水嶺(screen px)。press からこの距離を1軸でも超えたら
/// マーキー(超えた後に戻ってもマーキーのまま — 判定が途中で裏返らない)。
/// 値は Windows の既定 drag 閾値(4px)級の中庸値(自由なデザイン定数 —
/// `ZOOM_STEP_FACTOR` と同じ扱い、裁定に既定値の指定は無い)。
const CLICK_DRAG_THRESHOLD: f32 = 4.0;

/// マーキー面の不透明度(選択矩形の中身)。縁は accent の hairline、面は同色の
/// 12% — `inspector_pane` の `action_active.scale_alpha(..)` と同じ既存流儀
/// (新しい意味色ロールは足さない、裁定142「新色ゼロ」と同型)。
const MARQUEE_FILL_ALPHA: f32 = 0.12;

/// quad の縁上判定・退化判定の許容(screen px の交差積なので px² 級)。
const QUAD_EPS: f32 = 1e-6;

// ---------------------------------------------------------------------------
// pane ローカル Message(発注書「`SelectLayers { ids, additive }` 級」)。
// gizmo.rs の `GizmoDrag` と同じ理由で既存 [`crate::Message`] へ variant を
// 足さない(shell の exhaustive match は write-set 外)— supervisor が root 側で
// `.map(...)` して畳む。
// ---------------------------------------------------------------------------

/// キャンバス選択の1確定。release 時に1回だけ publish される(drag 途中の
/// プレビュー選択は出さない — 選択は release で確定、AE/Figma と同じ)。
///
/// shell 側の想定結線(結線は supervisor):
/// `session.selected_layers = apply_selection(&session.selected_layers, &ids, additive)`。
/// `ids` はヒット集合(現在選択との合成前) — 合成規則は [`apply_selection`] が
/// この crate 側で固定する。`ids=[]` + `additive=false` = 選択解除
/// (map 444/452)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectLayers {
    /// ヒットしたレイヤー(クリック=0または1件、マーキー=交差した全件。
    /// 並びは候補列の順=下→上の描画順のまま)。
    pub ids: Vec<LayerId>,
    /// Shift 押下(追加/トグル)。`false` = 置換。
    pub additive: bool,
}

/// ヒット集合を現在選択へ合成する純関数(意味の正本 — shell はこれを呼ぶだけ)。
///
/// - `additive=false`: 置換(`hits` がそのまま新しい選択。空なら選択解除)。
/// - `additive=true`: `hits` の各 id をトグル — 未選択なら末尾へ追加、選択済み
///   なら外す(map 966 のトグル意味論・AE/Figma の Shift クリックと同じ)。
///   既存選択の順序は保つ(選択順に意味を足さないが、無用な並び替えもしない)。
pub fn apply_selection(current: &[LayerId], hits: &[LayerId], additive: bool) -> Vec<LayerId> {
    if !additive {
        return hits.to_vec();
    }
    let mut out = current.to_vec();
    for id in hits {
        if let Some(index) = out.iter().position(|c| c == id) {
            out.remove(index);
        } else {
            out.push(*id);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 幾何(純関数)
// ---------------------------------------------------------------------------

/// drag の2点から正規化した選択矩形を組む(どの向きへ drag しても
/// width/height ≥ 0)。
pub fn marquee_rect(origin: Point, current: Point) -> Rectangle {
    let x = origin.x.min(current.x);
    let y = origin.y.min(current.y);
    Rectangle {
        x,
        y,
        width: (origin.x - current.x).abs(),
        height: (origin.y - current.y).abs(),
    }
}

/// レイヤーの screen quad(bounds ローカル、TL→TR→BR→BL)。**[`gizmo_layout`]
/// の `corners` をそのまま返す**(bbox 算出と座標系を二重化しない — GZ FINDING
/// の原点正規化も gizmo と同じ経路で満たす)。退化(letterbox が組めない・
/// 非有限)なら `None`。
pub fn selection_quad(
    bounds: Rectangle,
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: &GizmoTarget,
    dims: Dimensions,
) -> Option<[Point; 4]> {
    gizmo_layout(bounds, comp, render_camera, observation, target, dims).map(|layout| layout.corners)
}

/// 区間 `[min, max]` への射影(SAT 用)。
fn project(points: &[Point], axis: Vec2) -> (f32, f32) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for p in points {
        let d = axis.dot(Vec2::new(p.x, p.y));
        min = min.min(d);
        max = max.max(d);
    }
    (min, max)
}

/// 軸平行の選択矩形と(回転/skew され得る)凸 quad の交差判定(SAT)。
/// 分離軸 = 矩形の2軸 + quad の各辺の法線。触れているだけ(境界一致)も交差と
/// みなす(掠めた物を落とさない — 触れそうな物は触れる、Q0 と同じ方向)。
/// quad が線/点に潰れていても正しい(潰れた層も矩形が跨げば選べる)。
pub fn rect_intersects_quad(rect: Rectangle, quad: [Point; 4]) -> bool {
    if rect.width < 0.0 || rect.height < 0.0 {
        return false;
    }
    let rect_points = [
        Point::new(rect.x, rect.y),
        Point::new(rect.x + rect.width, rect.y),
        Point::new(rect.x + rect.width, rect.y + rect.height),
        Point::new(rect.x, rect.y + rect.height),
    ];

    let mut axes: Vec<Vec2> = vec![Vec2::X, Vec2::Y];
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let edge = Vec2::new(b.x - a.x, b.y - a.y);
        if edge.length_squared() > QUAD_EPS {
            axes.push(Vec2::new(-edge.y, edge.x));
        }
    }

    for axis in axes {
        let (rect_min, rect_max) = project(&rect_points, axis);
        let (quad_min, quad_max) = project(&quad, axis);
        if rect_max < quad_min || quad_max < rect_min {
            return false;
        }
    }
    true
}

/// 点が凸 quad の中か(縁上も含む)。交差積の符号が一貫するかで見る —
/// 負 scale で巻き向きが反転していても正しい。完全退化(全点同一線上)は
/// `false`(面の無い物はクリックでは掴めない — マーキーなら跨げば選べる)。
pub fn quad_contains_point(quad: [Point; 4], p: Point) -> bool {
    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        if cross.abs() <= QUAD_EPS {
            continue; // 縁上(または退化辺)はどちら側でもない — 他の辺で決める。
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    sign != 0.0
}

/// マーキー矩形に交差した全レイヤー(候補列の順のまま = 下→上の描画順)。
pub fn marquee_hit_ids(rect: Rectangle, quads: &[(LayerId, [Point; 4])]) -> Vec<LayerId> {
    quads
        .iter()
        .filter(|(_, quad)| rect_intersects_quad(rect, *quad))
        .map(|(id, _)| *id)
        .collect()
}

/// クリック位置の最前面レイヤー(候補列は下→上なので**後ろから**最初の命中)。
pub fn click_hit_id(cursor: Point, quads: &[(LayerId, [Point; 4])]) -> Option<LayerId> {
    quads
        .iter()
        .rev()
        .find(|(_, quad)| quad_contains_point(*quad, cursor))
        .map(|(id, _)| *id)
}

/// press がマーキー/クリック選択を始めてよいか(優先順位の正本)。
/// 表示中の gizmo のどれか([`gizmo_hit_test`] の再利用 — ハンドル・回転
/// ハンドル・body 全部)に命中していたら始めない(gizmo が勝つ)。
pub fn press_starts_marquee(gizmo_layouts: &[GizmoLayout], cursor: Point) -> bool {
    gizmo_layouts
        .iter()
        .all(|layout| gizmo_hit_test(layout, cursor).is_none())
}

// ---------------------------------------------------------------------------
// drag 状態(純関数の状態機械 — Document でも Session でもない widget 内だけの
// 一時状態、`GizmoDragState` と同格)
// ---------------------------------------------------------------------------

/// 進行中の press〜release。閾値([`CLICK_DRAG_THRESHOLD`])を1度でも超えたら
/// マーキーに確定し、戻っても裏返らない(sticky — release 直前の1pxで
/// クリック扱いへ化けない)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarqueeDragState {
    origin: Point,
    current: Point,
    became_marquee: bool,
}

impl MarqueeDragState {
    pub fn begin(origin: Point) -> Self {
        Self {
            origin,
            current: origin,
            became_marquee: false,
        }
    }

    pub fn update(&mut self, cursor: Point) {
        self.current = cursor;
        if (cursor.x - self.origin.x).abs() > CLICK_DRAG_THRESHOLD
            || (cursor.y - self.origin.y).abs() > CLICK_DRAG_THRESHOLD
        {
            self.became_marquee = true;
        }
    }

    /// マーキーとして扱うか(false = クリック)。
    pub fn is_marquee(&self) -> bool {
        self.became_marquee
    }

    /// 今の選択矩形(正規化済み)。
    pub fn rect(&self) -> Rectangle {
        marquee_rect(self.origin, self.current)
    }

    /// press した点(クリック判定に使う — release イベントは位置を運ばないので
    /// origin を使う: 閾値内しかクリックにならないため十分)。
    pub fn origin(&self) -> Point {
        self.origin
    }
}

/// release 時の message を決める純関数(publish 規則の正本):
///
/// - マーキー: 交差集合。クリック: 最前面1件(0件 = 空クリック)。
/// - `additive=false` はヒット0件でも publish する(= 選択解除、map 444/452)。
/// - `additive=true` でヒット0件は publish しない(トグル対象が無い —
///   Shift+空クリックで選択を失わない、AE と同じ)。
pub fn release_message(
    drag: &MarqueeDragState,
    quads: &[(LayerId, [Point; 4])],
    additive: bool,
) -> Option<SelectLayers> {
    let ids = if drag.is_marquee() {
        marquee_hit_ids(drag.rect(), quads)
    } else {
        click_hit_id(drag.origin(), quads).into_iter().collect()
    };
    if additive && ids.is_empty() {
        return None;
    }
    Some(SelectLayers { ids, additive })
}

// ---------------------------------------------------------------------------
// canvas::Program(翻訳だけ — StageOverlay / GizmoOverlay と同じ規律)
// ---------------------------------------------------------------------------

/// マーキーの canvas 一式。`Shell::view` が毎フレーム組み直し、`stack!` で
/// `StageOverlay` の上・`GizmoOverlay` の**下**に重ねる想定(結線は supervisor —
/// 上に置いても [`press_starts_marquee`] が gizmo 命中 press を素通しするので
/// どちらでも壊れないが、下に置けば gizmo の capture が先に走る)。
/// ホイール/中ボタン(観測カメラ)は一切食わない — 左ボタンと drag 中の
/// Esc だけ。
pub struct MarqueeOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    /// 選択候補(id + gizmo と同じ局所値の投影)。**下→上の描画順**で渡す —
    /// クリックは最前面優先([`click_hit_id`])がこの順に依る。shell 側は
    /// 可視レイヤーごとに [`crate::gizmo_target`] で組む(結線は supervisor)。
    candidates: Vec<GizmoTarget>,
    /// 今 gizmo を表示しているレイヤー(= 選択中)。press の優先順位判定
    /// ([`press_starts_marquee`])にだけ使う。
    gizmo_layers: Vec<LayerId>,
    dims: Dimensions,
    colors: Colors,
}

impl MarqueeOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        candidates: Vec<GizmoTarget>,
        gizmo_layers: Vec<LayerId>,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            candidates,
            gizmo_layers,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> iced::Element<'a, SelectLayers> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// 候補全員の screen quad(組めた物だけ — 退化した候補は黙って外す:
    /// 見えない物は選べないのが安全側)。
    fn quads(&self, bounds: Rectangle) -> Vec<(LayerId, [Point; 4])> {
        self.candidates
            .iter()
            .filter_map(|target| {
                selection_quad(
                    bounds,
                    self.comp,
                    self.render_camera,
                    self.observation,
                    target,
                    self.dims,
                )
                .map(|quad| (target.layer, quad))
            })
            .collect()
    }

    /// 表示中 gizmo の layout(press 優先順位判定用)。
    fn gizmo_layouts(&self, bounds: Rectangle) -> Vec<GizmoLayout> {
        self.candidates
            .iter()
            .filter(|target| self.gizmo_layers.contains(&target.layer))
            .filter_map(|target| {
                gizmo_layout(
                    bounds,
                    self.comp,
                    self.render_camera,
                    self.observation,
                    target,
                    self.dims,
                )
            })
            .collect()
    }
}

/// canvas の一時状態(drag と Shift の追跡 — `GizmoInteraction` と同型)。
#[derive(Default)]
pub struct MarqueeInteraction {
    drag: Option<MarqueeDragState>,
    shift: bool,
}

impl canvas::Program<SelectLayers> for MarqueeOverlay {
    type State = MarqueeInteraction;

    fn update(
        &self,
        state: &mut MarqueeInteraction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<SelectLayers>> {
        match event {
            canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                // 修飾キーは他 widget も見る — 記録だけして capture しない。
                state.shift = modifiers.shift();
                None
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                // Esc = キャンセル(drag 中だけ食う — message は出さない:
                // まだ何も選んでいないので取り消す物は矩形だけ)。
                state.drag.take()?;
                Some(canvas::Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let position = cursor.position_in(bounds)?;
                    if !press_starts_marquee(&self.gizmo_layouts(bounds), position) {
                        return None; // gizmo が勝つ(優先順位の正本は純関数)。
                    }
                    state.drag = Some(MarqueeDragState::begin(position));
                    Some(canvas::Action::capture())
                }
                mouse::Event::CursorMoved { .. } => {
                    let drag = state.drag.as_mut()?;
                    // drag 中は bounds の外へ出ても続く(GizmoOverlay と同じ:
                    // `position_in` は外に出た瞬間 `None` になるため絶対座標から
                    // ローカルへ自前で戻す)。
                    let absolute = cursor.position()?;
                    drag.update(Point::new(absolute.x - bounds.x, absolute.y - bounds.y));
                    Some(canvas::Action::request_redraw().and_capture())
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let drag = state.drag.take()?;
                    let message = release_message(&drag, &self.quads(bounds), state.shift);
                    let action = match message {
                        Some(message) => canvas::Action::publish(message),
                        // Shift+空振り: 選択は変えないが矩形は消す。
                        None => canvas::Action::request_redraw(),
                    };
                    Some(action.and_capture())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// マーキー中だけ選択矩形を描く(accent の hairline 縁 + 同色 12% の面 —
    /// 既存流儀、モジュール冒頭 doc)。クリック段階(閾値内)では描かない —
    /// 4px 角の点滅を出さない。
    fn draw(
        &self,
        state: &MarqueeInteraction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let Some(drag) = &state.drag else {
            return Vec::new();
        };
        if !drag.is_marquee() {
            return Vec::new();
        }
        let rect = drag.rect();
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let accent = self.colors.action_active;
        let path = canvas::Path::rectangle(
            Point::new(rect.x, rect.y),
            Size::new(rect.width, rect.height),
        );
        frame.fill(&path, accent.scale_alpha(MARQUEE_FILL_ALPHA));
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(accent)
                .with_width(self.dims.theme().stroke.hairline),
        );
        vec![frame.into_geometry()]
    }

    /// マーキー drag 中は Crosshair(Timeline 空白面の scrub と同じ「面を引く」
    /// 合図 — 文法§5.5)。それ以外は既定(hover のカーソル形状は上に重なる
    /// GizmoOverlay が受け持つ — ここで常時 Crosshair を出すと gizmo の
    /// hover 形状と喧嘩する)。
    fn mouse_interaction(
        &self,
        state: &MarqueeInteraction,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match &state.drag {
            Some(drag) if drag.is_marquee() => mouse::Interaction::Crosshair,
            _ => mouse::Interaction::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Affine2;

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    fn bounds() -> Rectangle {
        Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0))
    }

    fn dims() -> Dimensions {
        Dimensions::default()
    }

    /// 100x50 の層を comp 中央へ(gizmo.rs の fixture と同じ値 —
    /// bbox は (270,155)-(370,205))。
    fn target(id: u64) -> GizmoTarget {
        GizmoTarget {
            layer: LayerId(id),
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

    fn quad_of(target: &GizmoTarget) -> [Point; 4] {
        selection_quad(bounds(), COMP, ResolvedCamera::default(), None, target, dims())
            .expect("quad が組めるはず")
    }

    // -----------------------------------------------------------------
    // 幾何
    // -----------------------------------------------------------------

    /// どの向きへ drag しても矩形は正規化される(width/height ≥ 0)。
    #[test]
    fn marquee_rect_normalizes_reversed_drags() {
        let rect = marquee_rect(Point::new(100.0, 80.0), Point::new(40.0, 20.0));
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (40.0, 20.0, 60.0, 60.0)
        );
    }

    /// 軸平行どうしの基本: 重なれば true、離れていれば false。
    #[test]
    fn rect_intersects_quad_on_axis_aligned_overlap_only() {
        let quad = quad_of(&target(1)); // (270,155)-(370,205)
        assert!(rect_intersects_quad(
            Rectangle::new(Point::new(350.0, 190.0), Size::new(100.0, 100.0)),
            quad
        ));
        assert!(!rect_intersects_quad(
            Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0)),
            quad
        ));
    }

    /// **SAT の本命**: 45° 回転した quad の「AABB は重なるが実体は重ならない」
    /// 角の空白で false になる(素朴な AABB 判定ではないことの実測)。
    #[test]
    fn rect_intersects_quad_rejects_the_empty_corner_of_a_rotated_quad() {
        let mut rotated = target(1);
        rotated.rotation_degrees = 45.0;
        let quad = quad_of(&rotated);
        // 回転後の AABB の角(bbox 中心 (320,180) から対角方向の外れ)には
        // 実体が無い。AABB は跨ぐが SAT は false になる小矩形を置く。
        let min_x = quad.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let min_y = quad.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let corner_probe = Rectangle::new(Point::new(min_x + 1.0, min_y + 1.0), Size::new(4.0, 4.0));
        assert!(
            !rect_intersects_quad(corner_probe, quad),
            "回転 quad の AABB 角の空白で交差扱いになっている(AABB 判定に退化)"
        );
        // 実体(中心)を跨げば true のまま。
        let center_probe = Rectangle::new(Point::new(318.0, 178.0), Size::new(4.0, 4.0));
        assert!(rect_intersects_quad(center_probe, quad));
    }

    /// 線に潰れた quad(scale.y=0 相当)でも矩形が跨げば選べる。
    #[test]
    fn rect_intersects_quad_still_hits_a_degenerate_quad() {
        let line = [
            Point::new(270.0, 180.0),
            Point::new(370.0, 180.0),
            Point::new(370.0, 180.0),
            Point::new(270.0, 180.0),
        ];
        assert!(rect_intersects_quad(
            Rectangle::new(Point::new(300.0, 170.0), Size::new(20.0, 20.0)),
            line
        ));
        assert!(!rect_intersects_quad(
            Rectangle::new(Point::new(300.0, 100.0), Size::new(20.0, 20.0)),
            line
        ));
    }

    /// 点 in 凸 quad: 内側 true・外側 false。負 scale(巻き向き反転)でも同じ。
    #[test]
    fn quad_contains_point_handles_both_windings() {
        let quad = quad_of(&target(1));
        assert!(quad_contains_point(quad, Point::new(320.0, 180.0)));
        assert!(!quad_contains_point(quad, Point::new(100.0, 60.0)));

        let mut flipped = target(1);
        flipped.scale = [1.0, -1.0]; // 巻き向きが反転する。
        let flipped_quad = quad_of(&flipped);
        assert!(quad_contains_point(flipped_quad, Point::new(320.0, 180.0)));
        assert!(!quad_contains_point(flipped_quad, Point::new(100.0, 60.0)));
    }

    // -----------------------------------------------------------------
    // 座標系(GZ FINDING を踏まない — 発注 ORACLE)
    // -----------------------------------------------------------------

    /// **原点正規化の本命**: bounds の原点が (0,0) でなくても(pane が画面の
    /// 途中に居ても)quad は同じローカル座標に出る — gizmo と同じ
    /// `letterbox_screen_from_comp` 経路を通っていることの実測。
    #[test]
    fn selection_quad_is_bounds_local_regardless_of_pane_origin() {
        let target = target(1);
        let at_origin = quad_of(&target);
        let offset_bounds = Rectangle::new(Point::new(100.0, 50.0), iced::Size::new(640.0, 360.0));
        let offset = selection_quad(
            offset_bounds,
            COMP,
            ResolvedCamera::default(),
            None,
            &target,
            dims(),
        )
        .expect("quad が組めるはず");
        for (a, b) in at_origin.iter().zip(offset.iter()) {
            assert!(
                (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3,
                "pane 原点ずれ: {a:?} != {b:?}"
            );
        }
    }

    /// 観測カメラ(zoom 2)下でも quad は gizmo_layout の corners と一致する
    /// (bbox の座標系を二重化していないことの実測)。
    #[test]
    fn selection_quad_matches_gizmo_layout_under_the_observation_camera() {
        let target = target(1);
        let observation = Some(ObservationCamera {
            pan: [10.0, -20.0],
            zoom: 2.0,
        });
        let quad = selection_quad(
            bounds(),
            COMP,
            ResolvedCamera::default(),
            observation,
            &target,
            dims(),
        )
        .expect("quad が組めるはず");
        let layout = gizmo_layout(
            bounds(),
            COMP,
            ResolvedCamera::default(),
            observation,
            &target,
            dims(),
        )
        .expect("layout が組めるはず");
        assert_eq!(quad, layout.corners);
    }

    // -----------------------------------------------------------------
    // ヒットと選択合成
    // -----------------------------------------------------------------

    /// クリックは最前面(候補列の後ろ)が勝つ。外れは None。
    #[test]
    fn click_hit_id_picks_the_topmost_layer() {
        let bottom = target(1);
        let top = target(2); // 同じ位置に重なる2層。
        let quads = vec![(bottom.layer, quad_of(&bottom)), (top.layer, quad_of(&top))];
        assert_eq!(click_hit_id(Point::new(320.0, 180.0), &quads), Some(LayerId(2)));
        assert_eq!(click_hit_id(Point::new(10.0, 10.0), &quads), None);
    }

    /// マーキーは交差した全件を候補列の順(下→上)で返す。
    #[test]
    fn marquee_hit_ids_collects_every_intersecting_layer_in_order() {
        let mut left = target(1);
        left.position = [100.0, 180.0];
        let right = target(2);
        let quads = vec![(left.layer, quad_of(&left)), (right.layer, quad_of(&right))];
        // 両方を跨ぐ横長の矩形。
        let rect = Rectangle::new(Point::new(40.0, 150.0), Size::new(320.0, 60.0));
        assert_eq!(marquee_hit_ids(rect, &quads), vec![LayerId(1), LayerId(2)]);
        // 右の層だけ跨ぐ矩形。
        let rect = Rectangle::new(Point::new(300.0, 150.0), Size::new(40.0, 60.0));
        assert_eq!(marquee_hit_ids(rect, &quads), vec![LayerId(2)]);
    }

    /// 置換(additive=false)はヒットがそのまま新しい選択。空なら選択解除
    /// (map 444/452)。
    #[test]
    fn apply_selection_replaces_when_not_additive() {
        let current = vec![LayerId(1), LayerId(2)];
        assert_eq!(
            apply_selection(&current, &[LayerId(3)], false),
            vec![LayerId(3)]
        );
        assert_eq!(apply_selection(&current, &[], false), Vec::<LayerId>::new());
    }

    /// Shift(additive=true)はトグル: 未選択は追加・選択済みは外す(map 966 の
    /// トグル意味論)。既存の並びは保つ。
    #[test]
    fn apply_selection_toggles_when_additive() {
        let current = vec![LayerId(1), LayerId(2)];
        assert_eq!(
            apply_selection(&current, &[LayerId(3)], true),
            vec![LayerId(1), LayerId(2), LayerId(3)]
        );
        assert_eq!(
            apply_selection(&current, &[LayerId(1)], true),
            vec![LayerId(2)]
        );
        // 混在(1を外し3を足す)。
        assert_eq!(
            apply_selection(&current, &[LayerId(1), LayerId(3)], true),
            vec![LayerId(2), LayerId(3)]
        );
    }

    // -----------------------------------------------------------------
    // drag 状態と release 規則
    // -----------------------------------------------------------------

    /// 閾値内はクリックのまま、超えたらマーキー、戻ってもマーキーのまま(sticky)。
    #[test]
    fn drag_state_becomes_marquee_only_past_the_threshold_and_stays() {
        let origin = Point::new(100.0, 100.0);
        let mut drag = MarqueeDragState::begin(origin);
        assert!(!drag.is_marquee());
        drag.update(Point::new(102.0, 101.0)); // 閾値(4px)内。
        assert!(!drag.is_marquee());
        drag.update(Point::new(120.0, 100.0)); // 超えた。
        assert!(drag.is_marquee());
        drag.update(origin); // 戻っても裏返らない。
        assert!(drag.is_marquee());
    }

    /// クリック(閾値内 release)は最前面1件を置換で publish。
    #[test]
    fn release_message_click_selects_the_topmost_layer() {
        let t = target(7);
        let quads = vec![(t.layer, quad_of(&t))];
        let drag = MarqueeDragState::begin(Point::new(320.0, 180.0));
        assert_eq!(
            release_message(&drag, &quads, false),
            Some(SelectLayers {
                ids: vec![LayerId(7)],
                additive: false
            })
        );
    }

    /// 空クリック(additive=false)は `ids=[]` を publish = 選択解除
    /// (map 444/452)。Shift+空クリックは publish しない(選択を失わない)。
    #[test]
    fn release_message_empty_click_clears_only_without_shift() {
        let t = target(7);
        let quads = vec![(t.layer, quad_of(&t))];
        let drag = MarqueeDragState::begin(Point::new(10.0, 10.0)); // 層の外。
        assert_eq!(
            release_message(&drag, &quads, false),
            Some(SelectLayers {
                ids: vec![],
                additive: false
            })
        );
        assert_eq!(release_message(&drag, &quads, true), None);
    }

    /// マーキー release は交差全件を publish(Shift なら additive=true で運ぶ)。
    #[test]
    fn release_message_marquee_carries_every_hit_and_the_shift_flag() {
        let mut left = target(1);
        left.position = [100.0, 180.0];
        let right = target(2);
        let quads = vec![(left.layer, quad_of(&left)), (right.layer, quad_of(&right))];
        let mut drag = MarqueeDragState::begin(Point::new(40.0, 150.0));
        drag.update(Point::new(360.0, 210.0));
        assert!(drag.is_marquee());
        assert_eq!(
            release_message(&drag, &quads, true),
            Some(SelectLayers {
                ids: vec![LayerId(1), LayerId(2)],
                additive: true
            })
        );
    }

    // -----------------------------------------------------------------
    // 優先順位(gizmo が勝つ — 発注 EXACT TARGET 4)
    // -----------------------------------------------------------------

    /// gizmo のハンドル/body の上の press はマーキーを始めない。gizmo の外なら
    /// 始める。gizmo が1つも無ければどこでも始める。
    #[test]
    fn press_starts_marquee_only_off_the_gizmo() {
        let t = target(1);
        let layout = gizmo_layout(bounds(), COMP, ResolvedCamera::default(), None, &t, dims())
            .expect("layout が組めるはず");
        let layouts = vec![layout];
        // 角ハンドル(bbox TL)= gizmo が勝つ。
        assert!(!press_starts_marquee(&layouts, Point::new(270.0, 155.0)));
        // body 内部 = gizmo(move)が勝つ。
        assert!(!press_starts_marquee(&layouts, Point::new(320.0, 180.0)));
        // 回転ハンドルも gizmo。
        let rotate = layout.rotate_handle.expect("回転ハンドルが出るはず");
        assert!(!press_starts_marquee(&layouts, rotate));
        // gizmo の外 = マーキー開始。
        assert!(press_starts_marquee(&layouts, Point::new(100.0, 60.0)));
        // gizmo 無し = どこでも開始。
        assert!(press_starts_marquee(&[], Point::new(320.0, 180.0)));
    }
}
