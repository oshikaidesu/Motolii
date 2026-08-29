//! layer 単位オフスクリーンパスの枠(裁定153 S2、2026-08-21)。S4(2026-08-21)で
//! 内蔵 vism 第1号 [`EffectPass::Glow`] を追加(`glow` サブモジュール参照)。
//! 2026-08-29、第2号 [`EffectPass::Isf`] を追加(`isf` サブモジュール参照)——
//! `docs/vism-package-concept.md` §11 の stop-line 条件8(複数の実 plugin で
//! 公開境界をコード実証する)の evidence probe。**Glow と同じ形の Rust enum
//! variant(専用 struct・専用 param 名)を2つ並べても「境界が一般化するか」の
//! 証拠にならない**という supervisor 裁定を受け、`Isf` は Glow を模倣せず
//! ISF(Interactive Shader Format)の JSON manifest + GLSL を実際に読む——
//! 詳細・GLSL→WGSL 変換で崩れた/崩れなかった所は `isf` モジュール doc。
//!
//! [`EffectPass`] は compositor ローカルの**closed enum**(裁定13: trait はまだ
//! 作らない — [`crate::BlendMode`] と同じ形)。[`EffectPass::Identity`] は
//! 絵を変えない pass で、枠(オフスクリーンへ描き・pass 列を適用し・結果を
//! 通常合成へ戻す往復)そのものの正しさを固定するためだけに存在する。
//! [`EffectPass::Glow`] が最初の実 effect(裁定153 の切片割り、
//! `docs/reviews/2026-08-21-effect-seam-survey.md` 4節)。
//!
//! [`EffectScratch`] は中間 texture の再利用プール。**pipeline は `glow`/`isf`
//! サブモジュールが持つ**(S2 時点では Identity しか無く pipeline 不要だった、
//! S4 で Glow が最初の shader pass を持ち込んだ)。texture は
//! `(幅, 高さ, フォーマット)` をキーに使い回し、**フレームをまたいで毎回
//! 作り直さない**(M5 proof `GlowFixture` の Host 所有パターンと
//! 同じ動機 — `spikes/m5-known-implementation/M5-R0/src/glow.rs` `new()` が
//! texture/pipeline を一括生成して `render()` では再生成しないのと同型)。

use std::collections::HashMap;

mod glow;
pub(crate) mod isf;
mod wgsl_fragment;

pub(crate) use glow::{GlowPipelines, GLOW_INTERMEDIATE_FORMAT};
pub use isf::{IsfInput, IsfInputType, IsfManifest};
pub(crate) use isf::{IsfProgram, BLOOM_SOURCE, ISF_TARGET_FORMAT};
pub(crate) use wgsl_fragment::{WgslFragmentProgram, GRADIENT_SOURCE, GRADIENT_TARGET_FORMAT};

/// layer に適用する GPU pass の記述。**f32 param を持つので `Eq` は導出できない**
/// (`PartialEq` のみ — 既存の呼び手は `assert_eq!`/`PartialEq` 比較しか使っていない、
/// `HashMap`/`HashSet` のキーには使われていない)。`Isf` が `Vec<(String, f32)>` を
/// 運ぶため **`Copy` も導出できない**(`Glow`/`Identity` だけなら Copy だったが、
/// enum 全体の性質は一番大きい variant に揃う——呼び手はどこも `Copy` に依存して
/// いなかった、`render_effects.rs`/`presentable.rs` はどれも `&EffectPass` 越しの
/// 参照か `Clone` で足りている)。
#[derive(Clone, Debug, PartialEq)]
pub enum EffectPass {
    /// 恒等 pass。入力と出力が画素単位で一致する。
    Identity,
    /// 内蔵 vism 第1号(裁定153 S4)。bright-pass→水平blur→垂直blur→加算合成の5パス
    /// (`glow` サブモジュール参照、移植元は
    /// `spikes/m5-known-implementation/M5-R0/src/glow.rs`)。
    Glow {
        /// 輝度がこの値を超えた分だけ bright-pass が抜き出す(proof 既定 1.0)。
        threshold: f32,
        /// 合成時に blur 結果へ掛ける係数(proof 既定 0.75、`intensity: 0.0` で
        /// 実質 pass-through)。
        intensity: f32,
        /// 5-tap blur のタップ間隔(texel)。`1.0` が proof の固定オフセット
        /// (1texel・2texel)と厳密に一致する。
        radius: f32,
    },
    /// **内蔵 vism 第2号**——`Glow` と違い、この variant 自身は param を1つも
    /// 名指さない(`isf` サブモジュール doc 参照、「境界の名は ISF」)。実体は
    /// `effects::isf::IsfProgram`(1本だけ、`bloom.fs` — 実在する ISF
    /// ファイルの JSON `INPUTS` を汎用に読んで pipeline を組む)。`params` は
    /// `(name, value)` の対の列で、`IsfProgram::record` が manifest の名前と
    /// 突き合わせる——ここにも「threshold」という文字列は現れない。
    Isf {
        params: Vec<(String, f32)>,
    },
    /// 内蔵 vism 第3号。uniform も texture も読まない WGSL フラグメント1本
    /// (`wgsl_fragment` サブモジュール参照)。param を1つも持たない。
    Gradient,
}

impl EffectPass {
    /// この pass が layer 自身のテクスチャ境界の外へ出力を広げたい量(texel、
    /// 上下左右均等)。**既知の穴の根治**(`next/reference/KNOWN.md`)—
    /// `Compositor::render_with_effects` はこの値ぶんだけ scratch を layer 実寸より
    /// 大きく確保し、source を中央へ置いてから pass を回す。
    ///
    /// [`Self::Identity`] は画素単位の copy なので 0。[`Self::Glow`] は
    /// `blur_at`(`glow` サブモジュールの WGSL、`d2 = direction * step * 2`)が
    /// 1回の blur pass で届く最大距離が `step*2`(`step = max(round(radius), 1)`)——
    /// 水平 blur・垂直 blur は別々の pass で、片方が広げた軸をもう片方が再び
    /// 広げることはない(水平 blur は x のみ、垂直 blur は y のみ動かす)ので、
    /// x/y とも同じ `step*2` が上下左右の必要量になる(シェーダの `blur_at` と
    /// 厳密に一致させる—5-tap の重み・間隔は変えない)。
    pub fn padding(&self) -> u32 {
        match self {
            EffectPass::Identity => 0,
            EffectPass::Glow { radius, .. } => {
                // シェーダの `max(i32(round(params.radius)), 1)` と同じ丸め規約
                // (先に 1.0 で下限を掛けてから丸めても結果は同じだが、ここは
                // shader 側の「丸めてから下限」の順序をそのまま踏襲する)。
                let step = radius.round().max(1.0) as u32;
                step * 2
            }
            // **0 は近似、`Glow` と違って正しい値を計算できない**——`bloom.fs`
            // (`effects::isf` モジュール doc)は Glow と同じ理由(blur の到達距離)で
            // 本当は非 0 の padding が要るが、その距離は manifest の `radius` という
            // *named* param の値に依存する。`EffectPass::Glow` は `radius` を
            // 型付き field として直接持つので `padding()` がそれを読めるが、
            // `EffectPass::Isf` は `params: Vec<(String, f32)>` しか持たない
            // (どの param が空間的な reach を左右するかは manifest ごとに違う、
            // `isf` モジュール doc 冒頭の「ここにも名前を書かない」規律そのもの)。
            // ここで `radius` という名前を特別扱いすると、まさに避けたかった
            // 「Glow の語彙をこの enum に埋め込む」ことになる——padding=0 は
            // 「bloom は layer 自身の矩形の外へは滲まない(縁で clip される)」という
            // 正直な機能制限として選んだ(sampler の `ClampToEdge` が範囲外読みを
            // 安全にはするので、絵が壊れるわけではない——広がらないだけ)。
            // ISF 採否の判断材料: 汎用 pass が空間的 reach を正しく申告するには
            // (a) 「この param 名が reach を決める」という manifest 外の知識を
            // どこかへ持つ(結局また特化に戻る)か、(b) manifest の `MAX` から
            // 最悪ケースを保守的に確保するかのどちらかが要る——今回はどちらも
            // 実装していない。
            EffectPass::Isf { .. } => 0,
            EffectPass::Gradient => 0,
        }
    }

    /// この pass が scratch texture へ求める format。`None` なら layer 自身の
    /// 元 format をそのまま使う(`Identity` — 画素単位の copy なので変換不要)。
    /// `Some` は「この pass は自分専用の固定 format へ描く」——
    /// `effects::glow::GLOW_INTERMEDIATE_FORMAT`(HDR ヘッドルームが要る)、
    /// `effects::isf::ISF_TARGET_FORMAT`(layer の実際の format に揃えた固定値、
    /// `isf` モジュール doc 参照)がそれぞれ自分の値を持つ——**`render_effects.rs`
    /// 側は「Glow かどうか」を名指さない**(旧 `has_glow` の一般化。2本目の
    /// 実 effect を足して初めて、この一般化が要ることが分かった——1本しか無い
    /// うちは `if is_glow` で足りてしまい、それが「境界」なのか「Glow の都合」
    /// なのか区別がつかなかった)。
    pub(crate) fn intermediate_format(&self) -> Option<wgpu::TextureFormat> {
        match self {
            EffectPass::Identity => None,
            EffectPass::Glow { .. } => Some(GLOW_INTERMEDIATE_FORMAT),
            EffectPass::Isf { .. } => Some(ISF_TARGET_FORMAT),
            EffectPass::Gradient => Some(GRADIENT_TARGET_FORMAT),
        }
    }
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
