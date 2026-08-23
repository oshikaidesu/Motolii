use glam::Affine2;

use motolii_core::LayerPlacement;
use motolii_store::{property, LayerId, PropertyId, RationalTime, StoreView, Value};

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

