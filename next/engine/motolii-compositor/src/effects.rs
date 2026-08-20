//! layer 単位オフスクリーンパスの枠(裁定153 S2、2026-08-21)。
//!
//! [`EffectPass`] は compositor ローカルの**closed enum**(裁定13: trait はまだ
//! 作らない — [`crate::BlendMode`] と同じ形)。今は [`EffectPass::Identity`] だけを
//! 持つ: 絵を変えない pass で、枠(オフスクリーンへ描き・pass 列を適用し・結果を
//! 通常合成へ戻す往復)そのものの正しさを固定するためだけに存在する。実 effect
//! (Glow 等)は S4 がこの enum へ変種を足す(裁定153 の切片割り、
//! `docs/reviews/2026-08-21-effect-seam-survey.md` 4節)。
//!
//! [`EffectScratch`] は中間 texture の再利用プール。**pipeline はまだ無い** —
//! Identity は `wgpu::CommandEncoder::copy_texture_to_texture` だけで画素単位に
//! 表現できるので、shader/bind group を建てる理由が無い(S2 の要件「pipeline
//! 未定でも通る形」をそのまま満たす、shader pipeline の設計は S4 の Glow が持ち込む)。
//! texture は `(幅, 高さ, フォーマット)` をキーに使い回し、**フレームをまたいで
//! 毎回作り直さない**(M5 proof `GlowFixture` の Host 所有パターンと同じ動機 —
//! `spikes/m5-known-implementation/M5-R0/src/glow.rs` `new()` が texture/pipeline を
//! 一括生成して `render()` では再生成しないのと同型)。

use std::collections::HashMap;

/// layer に適用する GPU pass の記述。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectPass {
    /// 恒等 pass。入力と出力が画素単位で一致する。
    Identity,
}

/// オフスクリーン texture のプール。**サイズ+フォーマットが同じ物は使い回す** —
/// 新規生成は「そのサイズ/フォーマットの空き texture が無い時」だけ。
#[derive(Default)]
pub(crate) struct EffectScratch {
    free: HashMap<(u32, u32, wgpu::TextureFormat), Vec<wgpu::Texture>>,
    /// **新規生成した回数**(再利用ではなく実際に `device.create_texture` を呼んだ回数)。
    /// 「pass 無し layer はオフスクリーンを一切作らない」を試験が数値で縛れるように、
    /// ここを隠さない(`RenderTiming` が時間の内訳を隠さないのと同じ規律)。
    created: u64,
}

impl EffectScratch {
    pub(crate) fn acquire(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let key = (width, height, format);
        if let Some(pool) = self.free.get_mut(&key) {
            if let Some(texture) = pool.pop() {
                return texture;
            }
        }
        self.created += 1;
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-compositor-effect-scratch"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // TEXTURE_BINDING: 通常合成へ渡す時に texture_manager_2d が sample する。
            // COPY_DST: Identity の copy 先。COPY_SRC: 将来 pass を連鎖させる時に
            // 前段の出力を次段の入力へ持ち回るための余地(S2 では未使用)。
            // RENDER_ATTACHMENT: S4(Glow 等)が実 shader pass をここへ描く時の余地
            // (S2 では未使用 — Identity は copy だけで完結する)。
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// 使い終わった texture をプールへ返す。**GPU が読み終わってから**(呼び手が
    /// `device.poll` 済みであることを保証する)呼ぶこと — でなければ次の
    /// `acquire` がまだ使用中の texture を上書きしてしまう。
    pub(crate) fn release(
        &mut self,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        texture: wgpu::Texture,
    ) {
        self.free
            .entry((width, height, format))
            .or_default()
            .push(texture);
    }

    pub(crate) fn created_count(&self) -> u64 {
        self.created
    }
}
