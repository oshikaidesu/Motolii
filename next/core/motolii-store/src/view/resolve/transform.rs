//! `world_affine` の再帰合成(裁定173 H1)と、その部品である local transform。
//! `view.rs`(`resolve.rs` 経由)から移送(裁定220 SP-3、中身は変えていない)。

use std::collections::{HashMap, HashSet};

use motolii_core::RationalTime;
use motolii_eval::Value;

use crate::{property, LayerId, LayerPlacement, PropertyId, StoreError};

use super::super::StoreView;

impl<'a> StoreView<'a> {
    /// **この layer 自身**の property track だけから local `Affine2` を組む(裁定58
    /// の正本 `LayerPlacement::from_transform` をそのまま呼ぶ)。祖先を一切見ない —
    /// [`Self::world_affine`] がこれを再帰の各段の部品として使う。
    pub(super) fn local_placement_transform(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine2, StoreError> {
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        let vec2 = |name: &str, default: [f32; 2]| -> Result<[f32; 2], StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に2成分でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        Ok(LayerPlacement::from_transform(
            vec2(property::ANCHOR, [0.0, 0.0])?,
            self.resolve_position(layer, t)?,
            vec2(property::SCALE, [1.0, 1.0])?,
            scalar(property::ROTATION, 0.0)?,
            scalar(property::SKEW, 0.0)?,
            scalar(property::SKEW_AXIS, 0.0)?,
        ))
    }

    /// [`Self::local_placement_transform`] の外部公開口(裁定174 G1)。
    ///
    /// Ungroup が「Group の変換を子ローカルへ焼き込む」際に使う — 計算ロジックは
    /// ここで複製せず、H1 の正本をそのまま呼ぶだけ(`Document::ungroup_layers` が
    /// この口経由で group/子それぞれの local `Affine2` を読む、単一源)。
    pub fn local_transform(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        self.local_placement_transform(layer, t)
    }

    /// この layer の**world(comp 空間)アフィン** = 親の world アフィン × 自分の
    /// local アフィン(裁定173 H1、キーフレームは各ノードローカルのまま・**合成だけが
    /// 再帰**という利用者仮説の実装形)。旧世界 `crates/motolii-doc/src/
    /// spatial_resolve.rs::ensure_resolve_affine` の概念移植 — メモ化 `HashMap` +
    /// `visiting` の cycle ガードで「同じフレームで親を二度解決しない」を保証する。
    ///
    /// - **parent が無い**: local のみ(既存 Document は今まで通りの値)
    /// - **parent が tombstone(present ではない)/存在しない**: `present` でのフィルタで
    ///   `None` 扱いに縮退する — `next/ui/motolii-timeline-pane/src/projection.rs::rows`
    ///   (裁定173 H2)の `attrs.parent.filter(|p| present.contains(p))` と**同じ意味論**
    ///   (壊れた参照で resolve を落とさない、H2 到達性判定の踏襲)
    /// - **parent 鎖に循環がある**(書き込み時ガード `document::validate_no_parent_cycle`
    ///   を抜けた壊れた Document を読んだ場合の第二の柵、H-survey §2.2): 再訪を
    ///   `visiting` で検知し、その枝は local のみへ縮退する。**memo には書かない** —
    ///   循環中に観測される値は「本当の」world ではないので、後で(外側の呼び出しが)
    ///   書く値を上書きしない
    pub(super) fn world_affine(
        &self,
        layer: LayerId,
        t: RationalTime,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<glam::Affine2, StoreError> {
        if let Some(world) = memo.get(&layer) {
            return Ok(*world);
        }
        // メモ化の呼び出し回数証明(裁定173 H1 oracle)が数える「本当にこの layer を
        // 解決した回数」— memo hit を素通りした、ここから先の1回だけを数える。
        #[cfg(test)]
        record_world_affine_compute();

        let local = self.local_placement_transform(layer, t)?;

        let parent = self
            .attrs(layer)?
            .unwrap_or_default()
            .parent
            .filter(|p| present.contains(p));
        let Some(parent) = parent else {
            memo.insert(layer, local);
            return Ok(local);
        };

        if !visiting.insert(layer) {
            // 防御的セカンドガード(H-survey §2.2)。壊れた Document でも無限再帰
            // しない — memo には書かず、ローカルのみへ縮退した値をこの枝だけに返す。
            return Ok(local);
        }
        let parent_world = self.world_affine(parent, t, present, memo, visiting)?;
        visiting.remove(&layer);

        let world = parent_world * local;
        memo.insert(layer, world);
        Ok(world)
    }

    /// position の値。**`position`(Vec2 単一 track)を優先し、無ければ split(x/y 別
    /// track)を試す**(裁定61)。どちらも無ければ既定 `[0,0]`。
    ///
    /// split は「x か y のどちらかだけキーを打つ」も許す — 片方が無い場合はその成分だけ
    /// 0.0(AE で「そちらの軸は動かしていない」と同じ扱い)。
    pub(super) fn resolve_position(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let position = PropertyId::new(property::POSITION)?;
        match self.value_at(layer, &position, t)? {
            Some(Value::Vec2(v)) => return Ok([v[0] as f32, v[1] as f32]),
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::POSITION
                )))
            }
            None => {}
        }

        let x = self.split_position_component(layer, property::POSITION_X, t)?;
        let y = self.split_position_component(layer, property::POSITION_Y, t)?;
        Ok([x.unwrap_or(0.0), y.unwrap_or(0.0)])
    }

    pub(super) fn split_position_component(
        &self,
        layer: LayerId,
        name: &str,
        t: RationalTime,
    ) -> Result<Option<f32>, StoreError> {
        let property = PropertyId::new(name)?;
        match self.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(Some(v as f32)),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(None),
        }
    }
}

/// `world_affine` の呼び出し回数計測(裁定173 H1 oracle「メモ化の呼び出し回数証明」)。
/// `#[cfg(test)]` なのでテスト以外のビルドには存在しない — `StoreView` は `&self` の
/// 純粋な読み口という設計(モジュール doc)を壊さずに、白箱ユニットテスト
/// (このファイル末尾の `mod tests`)だけがこのスレッドローカルを覗く。
#[cfg(test)]
thread_local! {
    static WORLD_AFFINE_COMPUTE_COUNT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn record_world_affine_compute() {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.set(c.get() + 1));
}

#[cfg(test)]
pub(super) fn reset_world_affine_compute_count() {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.set(0));
}

#[cfg(test)]
pub(super) fn world_affine_compute_count() -> u32 {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.get())
}
