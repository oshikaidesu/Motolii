//! 文字効果 — 文字の層の輪郭を、文字を形にする段で変える効果。絵は返さない。
//! 棚の欄は配置効果・パス効果と同じ契約(`kind.rs`)。値は層の property なのでキーが打てる。
//! 文字の層にだけ掛かる(裁定 2026-09-13: 書体を跨ぐ morph は核でなく効果)。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use crate::doc::eval::Value;
pub mod morph;
use crate::doc::store::kind::Param;
use crate::doc::store::{LayerId, };

pub struct TextOpKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const TEXT_MORPH: &str = "motolii.text_morph";

/// 相手は別の文字の層(その書体・大きさで同じ文字を組み、輪郭を対応付けて混ぜる)。量は %。
pub const KINDS: &[TextOpKind] = &[
    TextOpKind { plugin_id: TEXT_MORPH, label: "Text Morph", params: &[
        Param::layer("target", "Target"),
        Param::number("amount", "Amount", 0.0, Some((0.0, 100.0))),
    ] },
];

pub fn kind(plugin_id: &str) -> Option<&'static TextOpKind> {
    KINDS.iter().find(|k| k.plugin_id == plugin_id)
}

/// 積まれた効果のうち morph が指す相手と混合率(0..1)。相手が無い(0)なら効かない。
/// 複数積まれていれば最後の 1 枚。
pub fn morph(effects: &[ResolvedEffect]) -> Option<(LayerId, f64)> {
    effects.iter().rev().find(|e| e.plugin_id == TEXT_MORPH).and_then(|effect| {
        let target = effect.params.iter().find_map(|(n, v)| match v { Value::LayerId(id) if n == "target" && *id != 0 => Some(LayerId(*id)), _ => None })?;
        let amount = effect.params.iter().find_map(|(n, v)| match v { Value::F64(a) if n == "amount" => Some(*a), _ => None }).unwrap_or(0.0);
        Some((target, (amount / 100.0).clamp(0.0, 1.0)))
    })
}

#[cfg(test)]
mod tests {
    use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
    use super::*;

    fn effect(params: &[(&str, Value)]) -> ResolvedEffect {
        ResolvedEffect { plugin_id: TEXT_MORPH.into(), params: params.iter().map(|(n, v)| ((*n).to_owned(), v.clone())).collect(), ..Default::default() }
    }

    /// 相手の無い札は効かず、量は % から割合へ、範囲の外は縁で止まる。
    #[test]
    fn morph_reads_target_and_amount() {
        assert_eq!(morph(&[effect(&[])]), None);
        assert_eq!(morph(&[effect(&[("target", Value::LayerId(0)), ("amount", Value::F64(50.0))])]), None);
        assert_eq!(morph(&[effect(&[("target", Value::LayerId(7))])]), Some((LayerId(7), 0.0)));
        assert_eq!(morph(&[effect(&[("target", Value::LayerId(7)), ("amount", Value::F64(150.0))])]), Some((LayerId(7), 1.0)));
        let other = ResolvedEffect { plugin_id: "motolii.blur".into(), ..Default::default() };
        assert_eq!(morph(&[other, effect(&[("target", Value::LayerId(2)), ("amount", Value::F64(25.0))])]), Some((LayerId(2), 0.25)));
    }
}
