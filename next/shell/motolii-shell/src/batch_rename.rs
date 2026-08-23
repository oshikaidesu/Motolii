//! Selected-layer auto-number rename.
//!
//! This is deliberately separate from Timeline's single-row `RenameDraft`:
//! inline editing owns one transient text field, while this component owns the
//! deterministic plan for a selection-wide operation and its one `apply_all`.

use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId};

const DEFAULT_PREFIX: &str = "Layer";
const FIRST_NUMBER: u64 = 1;
const MIN_WIDTH: usize = 3;

/// Rename live selected layers in Timeline row order as `Layer 001`, `Layer 002`, ….
///
/// The operation is all-or-nothing for locked layers and creates one undo step.
/// Stale or duplicate selection ids are ignored. The returned count is the
/// number of changed names, so a repeated invocation is a no-op.
pub(crate) fn apply_selected(doc: &mut Document, selected_layers: &[LayerId]) -> Result<usize, String> {
    let selected: std::collections::HashSet<LayerId> = selected_layers.iter().copied().collect();
    if selected.is_empty() {
        return Ok(0);
    }

    let view = doc.view();
    let mut targets = Vec::new();
    for layer in view.layers() {
        if !selected.contains(&layer) {
            continue;
        }
        let attrs = view
            .attrs(layer)
            .map_err(|error| format!("layer {} の属性を読めない: {error}", layer.0))?
            .unwrap_or_default();
        if attrs.locked {
            return Err(format!("layer {} はロックされているので一括改名できない", layer.0));
        }
        targets.push((layer, attrs.name));
    }
    drop(view);

    if targets.is_empty() {
        return Ok(0);
    }

    let last_number = FIRST_NUMBER + targets.len() as u64 - 1;
    let width = MIN_WIDTH.max(digits(last_number));
    let mut intents = Vec::with_capacity(targets.len());
    for (offset, (layer, current)) in targets.into_iter().enumerate() {
        let number = FIRST_NUMBER + offset as u64;
        let next = format!("{DEFAULT_PREFIX} {number:0width$}");
        if current != next {
            intents.push(Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { name: Some(next), ..Default::default() },
            });
        }
    }

    let changed = intents.len();
    if changed == 0 {
        return Ok(0);
    }
    doc.apply_all(intents)
        .map_err(|error| format!("一括改名を書けない: {error}"))?;
    Ok(changed)
}

fn digits(value: u64) -> usize {
    value.to_string().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{Intent, LayerMeta, LayerSource, LayerTiming};

    fn doc_with_layers(count: u64) -> Document {
        let mut doc = Document::new();
        for id in 1..=count {
            doc.apply_all([
                Intent::AddLayer(LayerId(id)),
                Intent::SetMeta {
                    layer: LayerId(id),
                    meta: LayerMeta {
                        source: LayerSource::Solid { rgba: [0, 0, 0, 255], width: 8, height: 8 },
                        order: id as i16,
                        timing: LayerTiming::place(0, None, 100),
                    },
                },
                Intent::SetAttrs {
                    layer: LayerId(id),
                    patch: LayerAttrsPatch { name: Some(format!("old-{id}")), ..Default::default() },
                },
            ])
            .unwrap();
        }
        doc
    }

    #[test]
    fn auto_rename_follows_row_order_and_undoes_as_one_step() {
        let mut doc = doc_with_layers(4);
        doc.mark_undo_floor();

        let changed = apply_selected(&mut doc, &[LayerId(4), LayerId(1), LayerId(3), LayerId(1)]).unwrap();

        assert_eq!(changed, 3);
        assert_eq!(doc.view().attrs(LayerId(1)).unwrap().unwrap().name, "Layer 001");
        assert_eq!(doc.view().attrs(LayerId(3)).unwrap().unwrap().name, "Layer 002");
        assert_eq!(doc.view().attrs(LayerId(4)).unwrap().unwrap().name, "Layer 003");
        assert_eq!(doc.view().attrs(LayerId(2)).unwrap().unwrap().name, "old-2");
        assert!(doc.undo(), "一括改名は1回で undo できるはず");
        for id in [1, 3, 4] {
            assert_eq!(doc.view().attrs(LayerId(id)).unwrap().unwrap().name, format!("old-{id}"));
        }
        assert!(!doc.can_undo());
    }

    #[test]
    fn locked_selection_rejects_without_partial_writes() {
        let mut doc = doc_with_layers(2);
        doc.apply(Intent::SetAttrs {
            layer: LayerId(2),
            patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
        })
        .unwrap();
        doc.mark_undo_floor();

        let error = apply_selected(&mut doc, &[LayerId(1), LayerId(2)]).unwrap_err();

        assert!(error.contains("ロック"));
        assert_eq!(doc.view().attrs(LayerId(1)).unwrap().unwrap().name, "old-1");
        assert!(!doc.can_undo());
    }
}
