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
/// 枠の正しさを固定するためだけの variant)と `Glow`(裁定153 S4、最初の実 shader pass)
/// しか持たない(`motolii_compositor::effects` モジュール doc 参照)。つまり
/// **「対応している」plugin_id は `"motolii.glow"` 1本だけ**で、それ以外は全て未知
/// である。`translate_blend_mode` のように未知を `Err` で fail-closed にすると、
/// 対応外の effect を1つでも積んだ layer が一律描画不能になる — それは
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
/// パニックしない: `effects` が空でも、全 plugin_id が未知でも、param の型が
/// 壊れていても、ここは常に正常終了する。
pub(crate) fn translate_effect_passes(
    effects: &[motolii_store::ResolvedEffect],
) -> Vec<motolii_compositor::EffectPass> {
    effects
        .iter()
        .filter_map(|effect| match effect.plugin_id.as_str() {
            "motolii.glow" => translate_glow_params(&effect.params),
            // それ以外の plugin_id はまだ対応する pass が無い。無音で skip する
            // (= pass を積まない、`translate_blend_mode` とは非対称——上のdoc参照)。
            _ => None,
        })
        .collect()
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

const KNOWN_EFFECTS: &[EffectDescriptor] = &[EffectDescriptor {
    plugin_id: "motolii.glow",
    params: GLOW_PARAMS,
}];

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

    /// 逆方向の固定: 今 engine が描けるのは glow 1本だけという事実そのものを縛る
    /// (`translate_effect_passes` の doc「対応している plugin_id は
    /// "motolii.glow" 1本だけ」と同じ主張を `known_effects()` 側からも固定する)。
    #[test]
    fn known_effects_is_exactly_glow_today() {
        assert_eq!(known_effects().len(), 1);
        assert_eq!(known_effects()[0].plugin_id, "motolii.glow");
        assert_eq!(known_effects()[0].params.len(), 3);
    }
}
