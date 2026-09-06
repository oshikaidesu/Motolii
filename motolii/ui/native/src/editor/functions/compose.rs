use crate::doc::store::{Document, Intent, LayerId, StoreError};

pub(crate) fn independent_layers(
    baseline: &Document,
    targets: &[LayerId],
    mut one: impl FnMut(&Document, LayerId) -> Result<Vec<Intent>, StoreError>,
) -> Result<(Vec<Intent>, usize), StoreError> {
    independent_blocks(baseline, targets, |doc, layer| {
        one(doc, layer).map(Block::Edits)
    })
    .map(|(intents, count, _)| (intents, count))
}

pub(crate) enum Block {
    Edits(Vec<Intent>),
    Rejected(String),
}

pub(crate) fn independent_blocks(
    baseline: &Document,
    targets: &[LayerId],
    mut one: impl FnMut(&Document, LayerId) -> Result<Block, StoreError>,
) -> Result<(Vec<Intent>, usize, Vec<(LayerId, String)>), StoreError> {
    let mut seen = std::collections::HashSet::new();
    let mut intents = Vec::new();
    let mut count = 0;
    let mut rejected = Vec::new();
    for &layer in targets {
        if !seen.insert(layer) {
            continue;
        }
        match one(baseline, layer)? {
            Block::Edits(block) => {
                if !block.is_empty() {
                    count += 1;
                }
                intents.extend(block);
            }
            Block::Rejected(reason) => rejected.push((layer, reason)),
        }
    }
    Ok((intents, count, rejected))
}
