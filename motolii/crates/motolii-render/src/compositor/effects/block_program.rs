//! 箱のブロック(`"STAGE": "block"`): 物の箱の並びと、全員の今のずれ(state)を GPU の storage buffer に置き、
//! ブロックが state を少しずつ書き換える計算シェーダー(提案 2026-09-15、利用者「わたしの推奨は gpu」「この天井を作るべきでない」)。
//!
//! 作者の file は `fn block(k: u32) -> Offset` と、欄 1 つにつき module 階の `override` 1 本(`@label` `@range` `@options` `@reach`)、
//! 札の名は `fn block` か file の頭の `@id` `@label` `@description`(`@rounds` `@scope` `@physics` も)。頭の JSON(`/*{ "STAGE": "block" }*/`、
//! `fn block(k: u32, p: BlockParams)`)も移行中は通る。`k` は物の番号、返すのは物の component へ足す分:
//! 位置(足す)・回転(足す)・大きさ(掛ける)・色と不透明 `tint`(掛ける、rgb が色の倍率、a が不透明の倍率)。
//! 描く側はこれを物ごとの motion として読む(頂点で位置・回転・大きさ、画素で色と不透明)。時間のずれは別の口(書類の Stagger)。
//! 読める物: `objects[k]`(箱・住む箱・間合い・重み・組)、`host`(時刻・物の数・何回目)、`now_lo(k)` / `now_hi(k)`(今のずれ込みの箱)、
//! `neighbor_count(k)` / `neighbor(k, i)`(同じ組で近くに居る物。全員を回らずに済む — 物の数に天井を作らない)。
//! 欄の struct と、掛かった物(`members`)全員に掛ける外枠、ずれを state へ足す所はここが manifest から組む。
//! `"ROUNDS": n` なら n 回続けて解く(毎回、前の回の state を読む)。

use super::isf::IsfManifest;

/// 欄の slot の数(uniform の `array<vec4f, 6>`)。
pub(crate) const PARAM_SLOTS: usize = 24;

/// 1 つの物の箱(comp の座標、描く時と同じ置き方)。`room_*` は住む箱(並べる Group の箱)。
/// `group` は同じ住む箱に居る物の印(押し合いは同じ組の中だけ)、`margin` は間合い、`weight` は譲る比。
/// `parent_slot` / `anchor_slot` は名指しの相手の物の番号(親の層 / Position Anchor の層、無ければ `NO_OBJECT`)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockItem {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub room_lo: [f32; 2],
    pub room_size: [f32; 2],
    pub radius: f32,
    pub group: u32,
    pub margin: f32,
    pub weight: f32,
    pub parent_slot: u32,
    pub anchor_slot: u32,
}

/// 相手が居ない印(WGSL の `NO_OBJECT`)。
pub const NO_OBJECT: u32 = u32::MAX;

impl Default for BlockItem {
    fn default() -> Self {
        Self { lo: [0.0; 2], hi: [0.0; 2], room_lo: [0.0; 2], room_size: [0.0; 2], radius: 0.0, group: 0, margin: 0.0, weight: 0.0, parent_slot: NO_OBJECT, anchor_slot: NO_OBJECT }
    }
}

/// 物ごとのずれ(位置の差・回転の差(度)・大きさの倍率・色と不透明の倍率 `tint` = rgb + a)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockOffset {
    pub translate: [f32; 2],
    pub rotate: f32,
    pub scale: f32,
    pub tint: [f32; 4],
}

impl Default for BlockOffset {
    fn default() -> Self {
        Self { translate: [0.0; 2], rotate: 0.0, scale: 1.0, tint: [1.0; 4] }
    }
}

/// 1 つの物の byte(WGSL の `Item` と同じ並び)。
pub(crate) const ITEM_BYTES: u64 = 56;
pub(crate) const OFFSET_BYTES: u64 = 32;

pub(crate) fn item_bytes(items: &[BlockItem]) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * ITEM_BYTES as usize);
    for it in items {
        for v in [it.lo[0], it.lo[1], it.hi[0], it.hi[1], it.room_lo[0], it.room_lo[1], it.room_size[0], it.room_size[1], it.radius] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&it.group.to_le_bytes());
        out.extend_from_slice(&it.margin.to_le_bytes());
        out.extend_from_slice(&it.weight.to_le_bytes());
        out.extend_from_slice(&it.parent_slot.to_le_bytes());
        out.extend_from_slice(&it.anchor_slot.to_le_bytes());
    }
    out
}

pub(crate) fn offset_bytes(offsets: &[BlockOffset]) -> Vec<u8> {
    offsets.iter().flat_map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale, o.tint[0], o.tint[1], o.tint[2], o.tint[3]]).flat_map(f32::to_le_bytes).collect()
}

pub(crate) fn offsets_from_bytes(bytes: &[u8]) -> Vec<BlockOffset> {
    bytes.chunks_exact(OFFSET_BYTES as usize).map(|c| {
        let f = |i: usize| f32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]]);
        BlockOffset { translate: [f(0), f(4)], rotate: f(8), scale: f(12), tint: [f(16), f(20), f(24), f(28)] }
    }).collect()
}

/// 近くに居る物の一覧(CSR: `starts[k]..starts[k + 1]` が物 k の相手)。同じ組の物を、組の一番大きい箱(+ 両側の届く距離 `reach`)の 2 倍の升目に振り、
/// 周り 3×3 の升目の物を相手にする。物の数に比例する(全組を回らない)。押し合いで升目より遠くへ動く物は相手を取りこぼしうる。
pub(crate) fn neighbors(items: &[BlockItem], reach: f32) -> (Vec<u32>, Vec<u32>) {
    use std::collections::HashMap;
    let mut extent: HashMap<u32, f32> = HashMap::new();
    for it in items {
        let e = (it.hi[0] - it.lo[0]).abs().max((it.hi[1] - it.lo[1]).abs()) + 2.0 * it.margin.max(reach);
        let slot = extent.entry(it.group).or_insert(1.0);
        *slot = slot.max(e);
    }
    let cell_of = |it: &BlockItem| {
        let size = extent[&it.group] * 2.0;
        let c = [(it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5];
        ((c[0] / size).floor() as i64, (c[1] / size).floor() as i64)
    };
    let mut cells: HashMap<(u32, i64, i64), Vec<u32>> = HashMap::new();
    for (k, it) in items.iter().enumerate() {
        let (x, y) = cell_of(it);
        cells.entry((it.group, x, y)).or_default().push(k as u32);
    }
    let mut starts = Vec::with_capacity(items.len() + 1);
    let mut list = Vec::new();
    for (k, it) in items.iter().enumerate() {
        starts.push(list.len() as u32);
        let (x, y) = cell_of(it);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(bucket) = cells.get(&(it.group, x + dx, y + dy)) {
                    list.extend(bucket.iter().copied().filter(|j| *j != k as u32));
                }
            }
        }
    }
    starts.push(list.len() as u32);
    (starts, list)
}

mod passes;
mod wgsl;

pub(crate) use passes::{read_state, BlockProgram, BlockWorld, FollowPass, RopePass, WorldPass};
pub(crate) use wgsl::{module_source, validate, wgsl_manifest};


/// test の入口: 棚に載った札(disk の構成では vism/ の今の file、焼き込みでは埋めた物 — 描く時と同じ道)。
#[cfg(test)]
pub(crate) fn catalog_definition(name: &str) -> super::VismDefinition {
    super::catalog::catalog_snapshot().definitions.iter().find(|d| d.source.name == name).unwrap_or_else(|| panic!("{name} is not on the shelf")).clone()
}

#[cfg(test)]
pub(crate) fn program_for(device: &wgpu::Device, name: &str) -> BlockProgram {
    let definition = catalog_definition(name);
    assert_eq!(definition.manifest.stage, super::isf::IsfStage::Block);
    BlockProgram::new(device, "block-test", &definition.vertex_text, definition.manifest.rounds)
}

#[cfg(test)]
mod tests;
