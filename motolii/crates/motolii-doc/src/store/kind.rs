//! 棚の 1 枚の宣言 — shader を持たない効果(配置)の契約。shader を持つ効果(pass・surface・field)は
//! `vism/*.wgsl` の manifest が同じ欄を宣言し、render の catalog が両方を 1 つの表にする(最小コア A、2026-09-07)。

use crate::doc::eval::Value;

pub enum ParamKind {
    Number,
    Vec2,
    /// 形のトグル。値は選択肢の番号(F64 で持つ)。
    Choice(&'static [&'static str]),
}

pub struct Param {
    pub name: &'static str,
    /// 窓に出る英語。
    pub label: &'static str,
    /// 欄の組。無ければ空文字。
    pub section: &'static str,
    pub kind: ParamKind,
    pub default: [f64; 2],
    pub range: Option<(f64, f64)>,
    /// この形の時だけ出る(配置効果の形のトグル)。None は全形。
    pub modes: Option<&'static [u8]>,
}

impl Param {
    pub const fn number(name: &'static str, label: &'static str, default: f64, range: Option<(f64, f64)>) -> Self {
        Self { name, label, section: "", kind: ParamKind::Number, default: [default, 0.0], range, modes: None }
    }
    pub const fn choice(name: &'static str, label: &'static str, choices: &'static [&'static str]) -> Self {
        Self { name, label, section: "", kind: ParamKind::Choice(choices), default: [0.0, 0.0], range: Some((0.0, (choices.len() - 1) as f64)), modes: None }
    }

    pub fn shown(&self, mode: u8) -> bool {
        self.modes.is_none_or(|modes| modes.contains(&mode))
    }

    pub fn default_value(&self) -> Value {
        match self.kind {
            ParamKind::Vec2 => Value::Vec2(self.default),
            _ => Value::F64(self.default[0]),
        }
    }

    pub fn choices(&self) -> Option<&'static [&'static str]> {
        match self.kind {
            ParamKind::Choice(choices) => Some(choices),
            _ => None,
        }
    }
}

/// shader を持たない棚の 1 枚が何を返すか。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// 配置の集合。
    Placement,
    /// 形の層の輪郭。
    Path,
    /// 平らな素材から立体を起こす(押し出し・縁の丸み)。
    Solid,
}

/// 棚に並ぶ 1 枚の見え方。
#[derive(Clone, Copy)]
pub struct Kind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
    pub family: Family,
}

/// shader を持たない棚の 1 枚の全部。棚と admission はこの表と ISF の manifest だけを読む。
pub fn all() -> impl Iterator<Item = Kind> {
    crate::doc::store::placement::KINDS
        .iter()
        .map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Placement })
        .chain(crate::doc::store::pathop::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Path }))
        .chain(crate::doc::store::solid::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Solid }))
}

pub fn kind(plugin_id: &str) -> Option<Kind> {
    all().find(|k| k.plugin_id == plugin_id)
}

/// 棚と Inspector に出す名前。
pub fn label(plugin_id: &str) -> Option<&'static str> {
    kind(plugin_id).map(|k| k.label)
}

pub fn choices(plugin_id: &str, param: &str) -> Option<&'static [&'static str]> {
    kind(plugin_id)?.params.iter().find(|p| p.name == param)?.choices()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 3 つの家が 1 つの表に載り、id は被らず、欄の既定は範囲の中。
    #[test]
    fn every_kind_is_on_one_table_with_sane_params() {
        let kinds: Vec<Kind> = all().collect();
        assert!(!kinds.is_empty());
        for (i, k) in kinds.iter().enumerate() {
            assert!(kinds[..i].iter().all(|o| o.plugin_id != k.plugin_id), "{} が 2 枚ある", k.plugin_id);
            assert!(!k.label.is_empty());
            for p in k.params {
                if let Some((lo, hi)) = p.range {
                    assert!(lo <= p.default[0] && p.default[0] <= hi, "{}.{} の既定が範囲外", k.plugin_id, p.name);
                }
                if let Some(c) = p.choices() {
                    assert!((p.default[0] as usize) < c.len());
                }
            }
        }
        assert_eq!(label(crate::doc::store::placement::REPEAT), Some("Repeater"));
        assert_eq!(choices(crate::doc::store::placement::REPEAT, "mode"), Some(crate::doc::store::placement::SHAPES));
        assert_eq!(kind(crate::doc::store::pathop::PUCKER_BLOAT).map(|k| k.family), Some(Family::Path));
    }
}
