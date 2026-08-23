use std::collections::HashSet;

use motolii_store::{Document, Intent, LayerId, LayerSource, StoreView};

/// Build one atomic edit for the selected text layers.
///
/// Selection is front state, so stale ids and non-text layers are ignored here
/// rather than becoming unsupported writes. The caller only supplies the
/// per-layer intent builder; this boundary owns filtering and the single
/// `apply_all` that makes the gesture one undo step.
pub(crate) fn apply_to_selected_text_layers<F>(
    doc: &mut Document,
    selected_layers: &[LayerId],
    mut build_intent: F,
) -> Result<(), String>
where
    F: FnMut(LayerId, &StoreView<'_>) -> Result<Option<Intent>, String>,
{
    let store = doc.view();
    let mut seen = HashSet::new();
    let mut intents = Vec::new();

    for &layer in selected_layers {
        if !seen.insert(layer) || !store.has_layer(layer) {
            continue;
        }
        let Some(meta) = store
            .meta(layer)
            .map_err(|error| format!("選択レイヤーを読めない: {error}"))?
        else {
            continue;
        };
        if !matches!(meta.source, LayerSource::Text) {
            continue;
        }
        if let Some(intent) = build_intent(layer, &store)? {
            intents.push(intent);
        }
    }
    drop(store);

    doc.apply_all(intents)
        .map_err(|error| format!("一括編集を書けない: {error}"))
}
