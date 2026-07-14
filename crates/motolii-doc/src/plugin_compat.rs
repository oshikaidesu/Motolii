//! D1f: 未知プラグインID・既知プラグインの未来版の「開く」側契約(F-9、実装ガード9、S13)。
//!
//! 開く=警告+パススルー・無変更保持・pass-through評価。書き出しの厳格化はD6の担当で、
//! ここでは拒否しない。既知だが`effect_version`が現行より新しい場合も、
//! downgrade errorにせず未知プラグインと同じdegraded層へ揃える(第二監査S13の採用分)。
//!
//! plugin_id自体は既知だが構造上の種別(Filter/LayerSource/...)が違う場合は、
//! degradeでは救えない「バグ」として`validate`側の型付きエラー(`DocumentError::PluginKindMismatch`)
//! に振り分ける(こちらはこのモジュールでは警告にしない)。

use crate::param_expect::{known_plugin_info, DocPluginKind};
use crate::schema::{ClipSource, EffectDefinition, TrackItem};
use crate::Document;

/// 既知idの将来版か、そもそも未知idか(実装ガード9・S13の層別)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginDegradation {
    /// plugin_idがdoc側の既知表に無い。
    UnknownPluginId,
    /// idは既知・種別も一致だが、`effect_version`が現行より新しい。
    FutureVersion {
        known_version: u32,
        found_version: u32,
    },
}

impl std::fmt::Display for PluginDegradation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPluginId => write!(f, "unknown plugin id"),
            Self::FutureVersion {
                known_version,
                found_version,
            } => write!(
                f,
                "future version (known={known_version}, found={found_version})"
            ),
        }
    }
}

/// 「開く」側で観測されたdegraded plugin 1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginOpenWarning {
    pub path: String,
    pub plugin_id: String,
    pub reason: PluginDegradation,
}

impl std::fmt::Display for PluginOpenWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: plugin `{}` is degraded ({})",
            self.path, self.plugin_id, self.reason
        )
    }
}

/// 種別不一致は呼び出し側(`validate`)がエラーとして扱うため、ここでは`None`を返す。
pub(crate) fn classify(
    plugin_id: &str,
    effect_version: u32,
    expected_kind: DocPluginKind,
) -> Option<PluginDegradation> {
    match known_plugin_info(plugin_id) {
        None => Some(PluginDegradation::UnknownPluginId),
        Some(info) if info.kind != expected_kind => None,
        Some(info) if effect_version > info.current_version => {
            Some(PluginDegradation::FutureVersion {
                known_version: info.current_version,
                found_version: effect_version,
            })
        }
        Some(_) => None,
    }
}

impl Document {
    /// 未知plugin_id・既知プラグインの将来版を警告として列挙する(ドキュメントは無変更)。
    ///
    /// `validate()`とは異なりエラーにしない。「開く」は縮退表示のための診断であり、
    /// 拒否判断(書き出し厳格化)はD6の担当。種別不一致(バグ)は`validate()`が別途エラーにする。
    pub fn plugin_open_warnings(&self) -> Vec<PluginOpenWarning> {
        let mut warnings = Vec::new();
        for track in &self.tracks {
            for item in &track.items {
                collect_item_warnings(item, &mut warnings);
            }
        }
        // D1l: params/plugin_idはUse本体ではなくDefinitionが持つため、台帳側を1回だけ歩く
        // (orphanも含む — 「開く」時点では参照有無に関わらず縮退を報告する)。
        collect_effect_definition_warnings(&self.effect_definitions, &mut warnings);
        warnings
    }
}

fn collect_item_warnings(item: &TrackItem, out: &mut Vec<PluginOpenWarning>) {
    match item {
        TrackItem::Clip(clip) => {
            let layer_id = clip.envelope.layer_id.get();
            if let ClipSource::Plugin {
                plugin_id,
                effect_version,
                ..
            } = &clip.source
            {
                push_if_degraded(
                    plugin_id,
                    *effect_version,
                    DocPluginKind::LayerSource,
                    &format!("layer{layer_id}.source"),
                    out,
                );
            }
        }
        TrackItem::Group(group) => {
            for child in &group.children {
                collect_item_warnings(child, out);
            }
        }
    }
}

fn collect_effect_definition_warnings(
    definitions: &[EffectDefinition],
    out: &mut Vec<PluginOpenWarning>,
) {
    for def in definitions {
        push_if_degraded(
            &def.plugin_id,
            def.effect_version,
            DocPluginKind::Filter,
            &format!("effect_definitions[{}]", def.id.get()),
            out,
        );
    }
}

fn push_if_degraded(
    plugin_id: &str,
    effect_version: u32,
    expected_kind: DocPluginKind,
    path: &str,
    out: &mut Vec<PluginOpenWarning>,
) {
    if let Some(reason) = classify(plugin_id, effect_version, expected_kind) {
        out.push(PluginOpenWarning {
            path: path.to_string(),
            plugin_id: plugin_id.to_string(),
            reason,
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn classify_unknown_plugin_id() {
        let reason = classify("vendor.filter.mystery", 1, DocPluginKind::Filter).unwrap();
        assert!(matches!(reason, PluginDegradation::UnknownPluginId));
    }

    #[test]
    fn classify_known_plugin_future_version_is_degraded_not_error() {
        // core.filter.opacity は現行 version 1。未来版(S13: downgrade errorにしない)。
        let reason = classify("core.filter.opacity", 2, DocPluginKind::Filter).unwrap();
        assert_eq!(
            reason,
            PluginDegradation::FutureVersion {
                known_version: 1,
                found_version: 2,
            }
        );
    }

    #[test]
    fn classify_known_plugin_current_or_past_version_is_supported() {
        assert!(classify("core.filter.opacity", 1, DocPluginKind::Filter).is_none());
        // sine は current_version=2。旧versionの参照(migrate対象)もdegradedにしない。
        assert!(classify("core.param.sine", 1, DocPluginKind::ParamDriver).is_none());
        assert!(classify("core.param.sine", 2, DocPluginKind::ParamDriver).is_none());
    }

    #[test]
    fn classify_kind_mismatch_is_not_a_warning() {
        // kind不一致は`validate`側の型付きエラーの範疇。ここではNone(警告にしない)。
        assert!(classify("core.filter.opacity", 1, DocPluginKind::LayerSource).is_none());
    }
}
