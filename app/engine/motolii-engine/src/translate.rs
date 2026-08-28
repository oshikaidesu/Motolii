//! 語彙の変換 — `motolii_store` の語彙(BlendMode/MatteMode/ResolvedEffect)を
//! `motolii_compositor` の語彙へ写す純関数群。`next/engine/motolii-engine/src/lib.rs`
//! から移送(SP-7、2026-08-23、中身は変えていない——移送のみ)。呼び手は
//! `crate::render`(`render_with_camera_override`/`layers_from_resolved`/`apply_matte`)。

use crate::EngineError;

/// `Composition::background`([f32;4]・0.0〜1.0)を素材アップロードが取る 8bit RGBA
/// へ写す。`round` で丸める(`as u8` の単純切り捨てだと 1.0 が 254 に落ちて
/// 「不透明のつもりが微妙に透ける」事故になる)。
pub(crate) fn to_u8_rgba(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[1] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[2] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[3] * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

/// `motolii_store::BlendMode`(Document の17値、裁定67 + BL2 の `Add`)を
/// `motolii_compositor::BlendMode`(合成器が表現できる分だけ、`motolii-compositor`
/// のモジュール doc 参照)へ写す。
///
/// **BL4(2026-08-22)で17値全部が `Ok` になった**——非分離4種(Hue/Saturation/
/// Color/Luminosity)も `motolii-compositor` 側に対応する variant が揃った
/// (`motolii_compositor::nonseparable_mode_index` 参照)ので、BL3 の分離可能11値と
/// 同じ1対1マッピングへ合流させた。[`EngineError::UnsupportedBlendMode`] 自体は
/// 削らない(型としては残す——`motolii_compositor::BlendMode` に将来 variant が
/// 増えた時にまたここが使う枠)が、**この関数からは現状もう構築されない**。
///
/// **`_` を使わない**(全 variant を列挙)——`motolii_store::BlendMode` に variant が
/// 増えた時、ここを更新し忘れるとコンパイルが落ちる。
pub(crate) fn translate_blend_mode(
    mode: motolii_store::BlendMode,
) -> Result<motolii_compositor::BlendMode, EngineError> {
    use motolii_compositor::BlendMode as Dst;
    use motolii_store::BlendMode as Src;
    match mode {
        Src::Normal => Ok(Dst::Normal),
        Src::Add => Ok(Dst::Add),
        Src::Multiply => Ok(Dst::Multiply),
        Src::Screen => Ok(Dst::Screen),
        Src::Overlay => Ok(Dst::Overlay),
        Src::Darken => Ok(Dst::Darken),
        Src::Lighten => Ok(Dst::Lighten),
        Src::ColorDodge => Ok(Dst::ColorDodge),
        Src::ColorBurn => Ok(Dst::ColorBurn),
        Src::HardLight => Ok(Dst::HardLight),
        Src::SoftLight => Ok(Dst::SoftLight),
        Src::Difference => Ok(Dst::Difference),
        Src::Exclusion => Ok(Dst::Exclusion),
        // 非分離4種(BL4、`motolii_compositor::nonseparable_mode_index` が扱う分)。
        Src::Hue => Ok(Dst::Hue),
        Src::Saturation => Ok(Dst::Saturation),
        Src::Color => Ok(Dst::Color),
        Src::Luminosity => Ok(Dst::Luminosity),
    }
}

/// `motolii_store::MatteMode`(AE/Lottie の4値)を `motolii_compositor::MatteMode`
/// (`motolii-compositor` の `matte` モジュール doc 参照)へ写す。**4値とも `Ok`**
/// (`translate_blend_mode` と違い、matte mode 自体に対応外は無い——BL4 で
/// `motolii-compositor` 側が4モード全部を実装した、`matte` モジュール doc 参照)。
///
/// **`_` を使わない**(全 variant を列挙、`translate_blend_mode` と同じ fail-closed
/// の形)。
pub(crate) fn translate_matte_mode(
    mode: motolii_store::MatteMode,
) -> motolii_compositor::MatteMode {
    use motolii_compositor::MatteMode as Dst;
    use motolii_store::MatteMode as Src;
    match mode {
        Src::Alpha => Dst::Alpha,
        Src::InvertedAlpha => Dst::InvertedAlpha,
        Src::Luma => Dst::Luma,
        Src::InvertedLuma => Dst::InvertedLuma,
    }
}

#[cfg(test)]
mod translate_blend_mode_tests {
    use super::translate_blend_mode;

    /// **BL2**: `Add` は `motolii-compositor` が無改造で出せる(モジュール doc
    /// 参照)ので `Ok` — `translate_effect_passes_tests` と同型の、private 関数への
    /// crate 内 unit test(`tests/` からは呼べないので colocate する)。
    #[test]
    fn add_is_accepted() {
        // `EngineError` は `PartialEq` を derive していない(`CompositorError`/
        // `MediaError` 由来の `#[from]` があるため)ので `Result` ごとの
        // `assert_eq!` はできない — `Ok` の中身だけを比較する。
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Add).unwrap(),
            motolii_compositor::BlendMode::Add
        );
    }

    /// **BL3**: 分離可能 blend の11値も同じ1対1で `Ok`(代表して Multiply/SoftLight
    /// の2つを固定 — 全11の網羅は `tests/blend_separable.rs` の数値検証が担う)。
    #[test]
    fn separable_modes_are_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Multiply).unwrap(),
            motolii_compositor::BlendMode::Multiply
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::SoftLight).unwrap(),
            motolii_compositor::BlendMode::SoftLight
        );
    }

    /// **BL4**: 非分離4値(Hue/Saturation/Color/Luminosity)も同じ1対1で `Ok`
    /// (`motolii-compositor` 側が実装した——数値の正しさは
    /// `tests/blend_nonseparable.rs` の独立オラクルが縛る)。
    #[test]
    fn nonseparable_modes_are_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Hue).unwrap(),
            motolii_compositor::BlendMode::Hue
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Saturation).unwrap(),
            motolii_compositor::BlendMode::Saturation
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Color).unwrap(),
            motolii_compositor::BlendMode::Color
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Luminosity).unwrap(),
            motolii_compositor::BlendMode::Luminosity
        );
    }
}

#[cfg(test)]
mod translate_matte_mode_tests {
    use super::translate_matte_mode;

    /// **BL4**: matte mode 4値は対応外が無いので、4値とも1対1で写ることを
    /// そのまま固定する(`translate_blend_mode_tests` と同型)。
    #[test]
    fn all_four_matte_modes_translate_one_to_one() {
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::Alpha),
            motolii_compositor::MatteMode::Alpha
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::InvertedAlpha),
            motolii_compositor::MatteMode::InvertedAlpha
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::Luma),
            motolii_compositor::MatteMode::Luma
        );
        assert_eq!(
            translate_matte_mode(motolii_store::MatteMode::InvertedLuma),
            motolii_compositor::MatteMode::InvertedLuma
        );
    }
}

/// `motolii_store::ResolvedEffect` の列(裁定153 S1、`resolve()` が運ぶ effect スタック)を
/// `motolii_compositor::EffectPass` の列(裁定153 S2、`LayerWithPasses::passes`)へ写す。
/// `translate_blend_mode` と**同型の語彙変換**だが、失敗のさせ方は逆にしてある:
///
/// **未知 plugin_id は `Err` にしない。無音で skip する**(pass を1本も積まない)。
/// 理由 — `motolii_compositor::EffectPass` は今のところ `Identity`(絵を変えない pass、
/// 枠の正しさを固定するためだけの variant)・`Glow`(裁定153 S4、最初の実 shader
/// pass)・`Isf`(2026-08-29、`docs/vism-package-concept.md` §11 条件8 の evidence
/// probe、`motolii_compositor::effects::isf` モジュール doc 参照)しか持たない
/// (`motolii_compositor::effects` モジュール doc 参照)。つまり**「対応している」
/// plugin_id は `"motolii.glow"`/`"motolii.isf_bloom"` の2本だけ**で、それ以外は
/// 全て未知である。`translate_blend_mode` のように未知を `Err` で fail-closed に
/// すると、対応外の effect を1つでも積んだ layer が一律描画不能になる — それは
/// 「壊れているのではなくまだ描けない」という effect の実情に反する。
/// blend mode(16値のうち対応外があれば明確に壊れている)と effect
/// (対応する pass がまだ一部しか実装されていないのが今の常態)とでは
/// 「未対応」の意味が違う、というのがこの非対称の理由。
///
/// **`"motolii.glow"` → `EffectPass::Glow` が最初の対応表エントリ**(裁定153 S4)。
/// 名前つき param(`threshold`/`intensity`/`radius`)は `ResolvedEffect.params` から
/// 探す — 無い param(track を触っていない)は proof の既定値で埋める
/// (`translate_glow_params` 参照)。**型が合わない値が入っていたら pass を1本も
/// 積まない**(EXACT TARGET #2 — パニックしない。fail-closed だが `Err` にはしない、
/// この layer の他の effect や layer 自体は普通に描ける)。
///
/// **`"motolii.isf_bloom"` → `EffectPass::Isf` の腕は `translate_glow_params` と
/// 違い、個別の param 名を1つも書かない**——`ResolvedEffect.params` のうち
/// `Value::F64` である物をそのまま `(name, value)` へ詰め替えるだけ
/// (`translate_isf_params` 参照)。名前と既定値の対応は
/// `motolii_compositor::IsfProgram::record` 側(manifest 由来)が持つので、
/// ここで二重に持たない——`ISF_BLOOM_PARAMS`(front 向けカタログ)だけが手書きで、
/// それも `known_effects_isf_bloom_catalog_matches_the_generic_manifest` が
/// 実体の manifest と食い違っていないか毎回検査する。
///
/// パニックしない: `effects` が空でも、全 plugin_id が未知でも、param の型が
/// 壊れていても、ここは常に正常終了する。
pub(crate) fn translate_effect_passes(
    effects: &[motolii_store::ResolvedEffect],
) -> Vec<motolii_compositor::EffectPass> {
    effects
        .iter()
        .filter_map(|effect| match effect.plugin_id.as_str() {
            "motolii.glow" => translate_glow_params(&effect.params),
            "motolii.isf_bloom" => Some(translate_isf_params(&effect.params)),
            // それ以外の plugin_id はまだ対応する pass が無い。無音で skip する
            // (= pass を積まない、`translate_blend_mode` とは非対称——上のdoc参照)。
            _ => None,
        })
        .collect()
}

/// `"motolii.isf_bloom"` の named param map を `EffectPass::Isf` へ写す。
/// **`translate_glow_params` と違い、"threshold" も "intensity" も名指さない**
/// (module doc「境界の名は ISF」節)——`Value::F64` である param をそのまま
/// 名前つきで渡すだけで、名前と既定値の対応・型検査は
/// `motolii_compositor::IsfProgram::record`(manifest 由来)側が担う。
fn translate_isf_params(params: &[(String, motolii_store::Value)]) -> motolii_compositor::EffectPass {
    motolii_compositor::EffectPass::Isf {
        params: params
            .iter()
            .filter_map(|(name, value)| match value {
                motolii_store::Value::F64(v) => Some((name.clone(), *v as f32)),
                _ => None,
            })
            .collect(),
    }
}

/// proof(`spikes/m5-known-implementation/M5-R0/src/glow.rs`)の既定値。
/// `threshold`/`intensity` は proof のハードコード値そのまま
/// (`bright_fs` の `1.0`、`composite_fs` の `0.75`)。`radius` は proof に
/// 名前つき param が無い(5-tap のオフセットが固定 1texel/2texel)ので、
/// `motolii_compositor::effects::glow` が選んだ「`radius = 1.0` が proof の固定
/// オフセットと厳密に一致する」写像に合わせた値(`EffectPass::Glow` の doc 参照)。
const GLOW_DEFAULT_THRESHOLD: f64 = 1.0;
const GLOW_DEFAULT_INTENSITY: f64 = 0.75;
const GLOW_DEFAULT_RADIUS: f64 = 1.0;

/// front 向けの effect 在庫(公開口 #3、`docs/reviews/2026-08-28-current-position.md`
/// 「★ 次の一手」)。**この一覧の外の plugin_id は engine が描けない**——2026-08-27 の
/// `TURBULENT_DISPLACE` 事故(engine が一つも知らない名前を FX STACK が出していた)の
/// 再発防止。front はここを読むだけにし、写しを持たない。
///
/// `known_effects()` に載っている plugin_id は必ず `translate_effect_passes` が
/// 実際に pass を積める集合と一致する——`known_effects_are_exactly_what_translate_effect_passes_accepts`
/// が両者の食い違いを縛る。
pub struct EffectDescriptor {
    pub plugin_id: &'static str,
    pub params: &'static [EffectParamDescriptor],
}

/// 1個の named param。
pub struct EffectParamDescriptor {
    pub name: &'static str,
    pub default: f64,
    /// **engine 側のシェーダ/合成コードに宣言された範囲が無ければ `None`**——
    /// 無い範囲を発明しない(Q0)。今のところ glow の3paramはどれも範囲を宣言
    /// していない(`motolii-compositor::effects::glow` 参照、shader は clamp しない)。
    pub range: Option<(f64, f64)>,
}

const GLOW_PARAMS: &[EffectParamDescriptor] = &[
    EffectParamDescriptor {
        name: "threshold",
        default: GLOW_DEFAULT_THRESHOLD,
        range: None,
    },
    EffectParamDescriptor {
        name: "intensity",
        default: GLOW_DEFAULT_INTENSITY,
        range: None,
    },
    EffectParamDescriptor {
        name: "radius",
        default: GLOW_DEFAULT_RADIUS,
        range: None,
    },
];

/// `"motolii.isf_bloom"` の3param。**手書き**(supervisor の timebox 指示で
/// 認められた fallback——`docs/vism-package-concept.md` §11 の evidence probe と
/// しては、front カタログを manifest から都度動的に生成する所まではやらず、この
/// 定数を手で書いた。ただし手書きが実体(`bloom.fs` の `INPUTS`)と黙って
/// ずれないよう、`known_effects_isf_bloom_catalog_matches_the_generic_manifest`
/// が `motolii_compositor::isf_bloom_manifest()` と突き合わせる——2026-08-27 の
/// `TURBULENT_DISPLACE` 事故(front が engine の知らない名前を持っていた)と同種の
/// drift を、今回は compile-time 一本化ではなく test-time cross-check で防ぐ)。
const ISF_BLOOM_PARAMS: &[EffectParamDescriptor] = &[
    EffectParamDescriptor {
        name: "threshold",
        default: 1.0,
        range: Some((0.0, 4.0)),
    },
    EffectParamDescriptor {
        name: "intensity",
        default: 0.75,
        range: Some((0.0, 4.0)),
    },
    EffectParamDescriptor {
        name: "radius",
        default: 1.0,
        range: Some((1.0, 8.0)),
    },
];

const KNOWN_EFFECTS: &[EffectDescriptor] = &[
    EffectDescriptor {
        plugin_id: "motolii.glow",
        params: GLOW_PARAMS,
    },
    EffectDescriptor {
        plugin_id: "motolii.isf_bloom",
        params: ISF_BLOOM_PARAMS,
    },
];

/// 現在 engine が実際に描ける effect の一覧(`plugin_id` + named param の名前・既定値・
/// 範囲)。**この関数が唯一の正本**——front の FX STACK・パラメータパネルはここを
/// 読むだけにし、`translate.rs` の写しを front 側に持たせない。
pub fn known_effects() -> &'static [EffectDescriptor] {
    KNOWN_EFFECTS
}

/// `"motolii.glow"` の named param map(`effect.{id}.param.{name}` track が実在する分
/// だけ、裁定153 S1 `ResolvedEffect::params`)を `EffectPass::Glow` へ写す。
/// track の無い param は proof の既定値(上記定数)。**値はあるが型が `Value::F64`
/// でない**場合は `None` を返して pass を1本も積まない(EXACT TARGET #2、
/// `translate_effect_passes` の doc 参照)。
fn translate_glow_params(params: &[(String, motolii_store::Value)]) -> Option<motolii_compositor::EffectPass> {
    let find = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, motolii_store::Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let threshold = find("threshold", GLOW_DEFAULT_THRESHOLD)?;
    let intensity = find("intensity", GLOW_DEFAULT_INTENSITY)?;
    let radius = find("radius", GLOW_DEFAULT_RADIUS)?;
    Some(motolii_compositor::EffectPass::Glow {
        threshold: threshold as f32,
        intensity: intensity as f32,
        radius: radius as f32,
    })
}

#[cfg(test)]
mod translate_effect_passes_tests {
    use super::translate_effect_passes;
    use motolii_store::ResolvedEffect;

    /// effect が無い layer は pass も無い(空 → 空)。
    #[test]
    fn no_effects_yields_no_passes() {
        assert_eq!(translate_effect_passes(&[]), Vec::new());
    }

    /// 未知 plugin_id はパニックせず無音で skip される —
    /// 2026-08-21 時点は「既知」の plugin_id が1つも無いので、これは
    /// 「何を入れても今は空になる」ことの直接固定でもある。
    #[test]
    fn unknown_plugin_id_is_skipped_silently() {
        let effects = vec![
            ResolvedEffect {
                plugin_id: "motolii.not-yet-implemented".to_owned(),
                params: vec![],
            },
            ResolvedEffect {
                plugin_id: "third-party.whatever".to_owned(),
                params: vec![],
            },
        ];
        assert_eq!(translate_effect_passes(&effects), Vec::new());
    }
}

#[cfg(test)]
mod known_effects_tests {
    use super::{known_effects, translate_effect_passes};
    use motolii_store::ResolvedEffect;

    /// **虚報防止**(Q0、2026-08-27 の `TURBULENT_DISPLACE` 事故の再発防止)。
    /// `known_effects()` に載っている plugin_id は、param を1つも渡さなくても
    /// `translate_effect_passes` が必ず1本 pass を積める(= 実際に描ける)。
    /// 「窓を叩いても見えない嘘」の一種——この一覧が engine の描画能力と食い違うと、
    /// front はここを読むだけで存在しない effect を見せてしまう。
    #[test]
    fn known_effects_are_all_actually_drawable() {
        for descriptor in known_effects() {
            let effect = ResolvedEffect {
                plugin_id: descriptor.plugin_id.to_owned(),
                params: vec![],
            };
            assert_eq!(
                translate_effect_passes(&[effect]).len(),
                1,
                "known_effects() says {} is drawable but translate_effect_passes skipped it",
                descriptor.plugin_id,
            );
        }
    }

    /// 逆方向の固定: 今 engine が描けるのは glow/isf_bloom の2本だけという事実
    /// そのものを縛る(`translate_effect_passes` の doc と同じ主張を
    /// `known_effects()` 側からも固定する)。
    #[test]
    fn known_effects_is_exactly_glow_and_isf_bloom_today() {
        assert_eq!(known_effects().len(), 2);
        assert_eq!(known_effects()[0].plugin_id, "motolii.glow");
        assert_eq!(known_effects()[0].params.len(), 3);
        assert_eq!(known_effects()[1].plugin_id, "motolii.isf_bloom");
        assert_eq!(known_effects()[1].params.len(), 3);
    }

    /// `ISF_BLOOM_PARAMS`(手書き、supervisor timebox fallback——module doc
    /// 参照)が、実際に GPU pipeline を組む側の manifest
    /// (`motolii_compositor::isf_bloom_manifest()`、`bloom.fs` の `INPUTS` を
    /// 汎用に読んだ実体)と食い違っていないかを毎回検査する。ここが赤くなったら
    /// `bloom.fs` の `INPUTS` を変えたのに `ISF_BLOOM_PARAMS` を直し忘れている
    /// ——2026-08-27 の `TURBULENT_DISPLACE` 事故と同じ形の drift を、
    /// compile-time の一本化ではなく test-time の cross-check で塞ぐ。
    #[test]
    fn known_effects_isf_bloom_catalog_matches_the_generic_manifest() {
        let hand_written = known_effects()
            .iter()
            .find(|descriptor| descriptor.plugin_id == "motolii.isf_bloom")
            .expect("motolii.isf_bloom is in the catalog");
        let manifest = motolii_compositor::isf_bloom_manifest();
        let generic: Vec<_> = manifest.param_inputs().collect();

        assert_eq!(
            hand_written.params.len(),
            generic.len(),
            "ISF_BLOOM_PARAMS と bloom.fs の INPUTS で param 数が食い違っている"
        );
        for param in hand_written.params {
            let from_manifest = generic
                .iter()
                .find(|input| input.name == param.name)
                .unwrap_or_else(|| panic!("bloom.fs の INPUTS に `{}` が無い", param.name));
            assert_eq!(
                param.default,
                f64::from(from_manifest.default[0]),
                "`{}` の既定値が ISF_BLOOM_PARAMS と bloom.fs の INPUTS で食い違っている",
                param.name
            );
        }
    }
}
