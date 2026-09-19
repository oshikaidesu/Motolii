//! 同梱の効果: 契約は doc、実装はここ。コアは誰が何を実装しているか知らない。
mod bundled;
pub mod blob;
pub mod motion;
pub mod overlay;
pub mod pathop;
pub mod placement;
pub mod solid;
pub mod text;

pub use bundled::{bundled, placement_program, sampling_program, snap_program};
use crate::doc::store::kind::{Kind, Family};
#[cfg(test)]
use crate::doc::eval::Value;

/// shader を持たない棚の 1 枚の全部。棚と admission はこの表と ISF の manifest だけを読む。
pub fn all() -> impl Iterator<Item = Kind> {
    placement::KINDS
        .iter()
        .map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Placement })
        .chain(pathop::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Path }))
        .chain(blob::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Placement }))
        .chain(motion::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Placement }))
        .chain(solid::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Solid }))
        .chain(overlay::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Output }))
        .chain(text::KINDS.iter().map(|k| Kind { plugin_id: k.plugin_id, label: k.label, params: k.params, family: Family::Text }))
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

    /// 4 つの家が 1 つの表に載り、id は被らず、欄の既定は範囲の中。
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
        assert_eq!(label(placement::REPEAT), Some("Repeater"));
        assert_eq!(choices(placement::REPEAT, "mode"), Some(placement::SHAPES));
        assert_eq!(kind(pathop::PUCKER_BLOAT).map(|k| k.family), Some(Family::Path));
        assert_eq!(kind(text::TEXT_MORPH).map(|k| k.family), Some(Family::Text));
        assert_eq!(kind(text::TEXT_MORPH).and_then(|k| k.params.iter().find(|p| p.name == "target")).map(|p| p.default_value()), Some(Value::LayerId(0)));
    }
}
