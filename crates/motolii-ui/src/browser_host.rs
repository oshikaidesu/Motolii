use motolii_plugin::{F64Domain, PluginCatalog, PluginKind, Value, ValueType};
use motolii_plugins_firstparty::first_party_catalog;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const CODEC_VERSION: u8 = 1;
const BROWSER_ROLE: &str = "browser";
const HOST_TO_WEB: &str = "host-to-web";
const WEB_TO_HOST: &str = "web-to-host";
const PLACE_KIND: &str = "browser.place";
const MAX_ID_BYTES: usize = 128;
const MAX_MESSAGE_BYTES: usize = 1024;
const INBOX_CAPACITY: usize = 16;
const CATALOG_REVISION: u8 = 1;
const OPACITY_ID: &str = "core.filter.opacity";
const OPACITY_NAME: &str = "Opacity";
const OPACITY_CATEGORY: &str = "Color";
const OPACITY_PARAM: &str = "amount";
const EFFECTS_SCOPE: &str = "first-party-effects";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserPlaceIntent {
    pub(crate) scope_ref: String,
    pub(crate) item_id: String,
}

#[derive(Debug)]
pub(crate) struct BrowserHostSession {
    instance_epoch: u64,
    last_sequence: u64,
    rectangle_source: BrowserPlaceIntent,
    inbox: VecDeque<BrowserPlaceIntent>,
}

impl BrowserHostSession {
    pub(crate) fn new(
        instance_epoch: u64,
        sequence: u64,
        rectangle_source: BrowserPlaceIntent,
    ) -> Self {
        Self {
            instance_epoch,
            last_sequence: sequence,
            rectangle_source,
            inbox: VecDeque::with_capacity(INBOX_CAPACITY),
        }
    }

    pub(crate) fn snapshot_json(&self) -> Result<String, BrowserHostError> {
        validate_id(&self.rectangle_source.scope_ref, "scope_ref")?;
        validate_id(&self.rectangle_source.item_id, "item_id")?;
        let first_party = first_party_catalog()?;
        let catalog = opacity_catalog_snapshot(&first_party)?;
        serde_json::to_string(&HostSnapshot {
            version: CODEC_VERSION,
            direction: HOST_TO_WEB,
            role: BROWSER_ROLE,
            instance_epoch: self.instance_epoch.to_string(),
            sequence: self.last_sequence.to_string(),
            browser: HostBrowser {
                rectangle_source: WireSource {
                    scope_ref: &self.rectangle_source.scope_ref,
                    item_id: &self.rectangle_source.item_id,
                },
                catalog,
            },
        })
        .map_err(BrowserHostError::Encode)
    }

    pub(crate) fn accept(&mut self, raw: &str) -> Result<(), BrowserHostError> {
        if raw.len() > MAX_MESSAGE_BYTES {
            return Err(BrowserHostError::MessageTooLarge);
        }
        let message: WebMessage = serde_json::from_str(raw).map_err(BrowserHostError::Decode)?;
        if message.version != CODEC_VERSION {
            return Err(BrowserHostError::Version);
        }
        if message.direction != WEB_TO_HOST {
            return Err(BrowserHostError::Direction);
        }
        if message.role != BROWSER_ROLE {
            return Err(BrowserHostError::Role);
        }
        if message.kind != PLACE_KIND {
            return Err(BrowserHostError::Kind);
        }
        let instance_epoch = parse_canonical_u64(&message.instance_epoch, "instance_epoch")?;
        if instance_epoch != self.instance_epoch {
            return Err(BrowserHostError::StaleInstance);
        }
        let sequence = parse_canonical_u64(&message.sequence, "sequence")?;
        let expected = self
            .last_sequence
            .checked_add(1)
            .ok_or(BrowserHostError::SequenceExhausted)?;
        if sequence != expected {
            return Err(BrowserHostError::Sequence {
                expected,
                actual: sequence,
            });
        }
        validate_id(&message.source.scope_ref, "scope_ref")?;
        validate_id(&message.source.item_id, "item_id")?;
        if message.source.scope_ref != self.rectangle_source.scope_ref
            || message.source.item_id != self.rectangle_source.item_id
        {
            return Err(BrowserHostError::Source);
        }
        if self.inbox.len() == INBOX_CAPACITY {
            return Err(BrowserHostError::InboxFull);
        }
        self.inbox.push_back(BrowserPlaceIntent {
            scope_ref: message.source.scope_ref,
            item_id: message.source.item_id,
        });
        self.last_sequence = sequence;
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Option<BrowserPlaceIntent> {
        self.inbox.pop_front()
    }
}

#[derive(Debug, Serialize)]
struct HostSnapshot<'a> {
    version: u8,
    direction: &'static str,
    role: &'static str,
    instance_epoch: String,
    sequence: String,
    browser: HostBrowser<'a>,
}

#[derive(Debug, Serialize)]
struct HostBrowser<'a> {
    rectangle_source: WireSource<'a>,
    catalog: HostCatalogSnapshot<'a>,
}

#[derive(Debug, Serialize)]
struct WireSource<'a> {
    scope_ref: &'a str,
    item_id: &'a str,
}

#[derive(Debug, Serialize)]
struct HostCatalogSnapshot<'a> {
    catalog_revision: u8,
    vocabularies: HostVocabularies,
    catalogs: [HostScopedCatalog<'a>; 1],
}

#[derive(Debug, Serialize)]
struct HostVocabularies {
    scopes: [HostVocabularyEntry; 1],
    taxonomies: [HostVocabularyEntry; 2],
    providers: [HostVocabularyEntry; 1],
    packs: [HostVocabularyEntry; 0],
    install_states: [HostVocabularyEntry; 1],
    impact_units: [HostVocabularyEntry; 0],
    tags: [HostVocabularyEntry; 0],
}

#[derive(Debug, Serialize)]
struct HostVocabularyEntry {
    id: &'static str,
    label: &'static str,
    scope_ref: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct HostScopedCatalog<'a> {
    scope_ref: &'static str,
    items: [HostCatalogItem<'a>; 1],
}

#[derive(Debug, Serialize)]
struct HostCatalogItem<'a> {
    item_id: &'a str,
    display_name: &'a str,
    taxonomy_refs: [&'static str; 2],
    provider_ref: &'static str,
    pack_ref: Option<&'static str>,
    install_state_ref: &'static str,
    preview_kind: &'static str,
    impact: Option<()>,
    tag_refs: [&'static str; 0],
}

fn opacity_catalog_snapshot(
    catalog: &PluginCatalog,
) -> Result<HostCatalogSnapshot<'_>, BrowserHostError> {
    let matches = catalog
        .iter()
        .filter(|(id, _)| id.0 == OPACITY_ID)
        .collect::<Vec<_>>();
    let [(_, contract)] = matches.as_slice() else {
        return Err(BrowserHostError::OpacityCatalogCardinality {
            actual: matches.len(),
        });
    };
    let node = &contract.node;
    validate_opacity_contract(contract.kind == PluginKind::Filter, "kind")?;
    validate_opacity_contract(node.id.0 == OPACITY_ID, "id")?;
    validate_opacity_contract(node.version == 1, "version")?;
    validate_opacity_contract(node.display_name == OPACITY_NAME, "display_name")?;
    validate_opacity_contract(node.category == OPACITY_CATEGORY, "category")?;
    validate_opacity_contract(node.min_inputs == 1 && node.max_inputs == 1, "inputs")?;
    validate_opacity_contract(contract.migrations.is_empty(), "migrations")?;
    let [param] = node.params.as_slice() else {
        return Err(BrowserHostError::OpacityContractMismatch { field: "params" });
    };
    validate_opacity_contract(param.id == OPACITY_PARAM, "param.id")?;
    validate_opacity_contract(param.value_type == ValueType::F64, "param.value_type")?;
    validate_opacity_contract(param.default == Value::F64(1.0), "param.default")?;
    validate_opacity_contract(
        param.f64_domain == Some(F64Domain::unit()),
        "param.f64_domain",
    )?;

    Ok(HostCatalogSnapshot {
        catalog_revision: CATALOG_REVISION,
        vocabularies: HostVocabularies {
            scopes: [HostVocabularyEntry {
                id: EFFECTS_SCOPE,
                label: "First-party effects",
                scope_ref: None,
            }],
            taxonomies: [
                HostVocabularyEntry {
                    id: "effect",
                    label: "Effect",
                    scope_ref: Some(EFFECTS_SCOPE),
                },
                HostVocabularyEntry {
                    id: "color",
                    label: OPACITY_CATEGORY,
                    scope_ref: Some(EFFECTS_SCOPE),
                },
            ],
            providers: [HostVocabularyEntry {
                id: "built-in",
                label: "Built-in",
                scope_ref: Some(EFFECTS_SCOPE),
            }],
            packs: [],
            install_states: [HostVocabularyEntry {
                id: "installed",
                label: "Installed",
                scope_ref: Some(EFFECTS_SCOPE),
            }],
            impact_units: [],
            tags: [],
        },
        catalogs: [HostScopedCatalog {
            scope_ref: EFFECTS_SCOPE,
            items: [HostCatalogItem {
                item_id: node.id.0,
                display_name: node.display_name,
                taxonomy_refs: ["effect", "color"],
                provider_ref: "built-in",
                pack_ref: None,
                install_state_ref: "installed",
                preview_kind: "poster",
                impact: None,
                tag_refs: [],
            }],
        }],
    })
}

fn validate_opacity_contract(condition: bool, field: &'static str) -> Result<(), BrowserHostError> {
    if condition {
        Ok(())
    } else {
        Err(BrowserHostError::OpacityContractMismatch { field })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebMessage {
    version: u8,
    direction: String,
    role: String,
    instance_epoch: String,
    sequence: String,
    kind: String,
    source: OwnedWireSource,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedWireSource {
    scope_ref: String,
    item_id: String,
}

fn parse_canonical_u64(value: &str, field: &'static str) -> Result<u64, BrowserHostError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(BrowserHostError::UnsignedInteger { field });
    }
    value
        .parse()
        .map_err(|_| BrowserHostError::UnsignedInteger { field })
}

fn validate_id(value: &str, field: &'static str) -> Result<(), BrowserHostError> {
    if value.is_empty() || value.len() > MAX_ID_BYTES {
        return Err(BrowserHostError::Identifier { field });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BrowserHostError {
    #[error("Browser Host message exceeds 1024 bytes")]
    MessageTooLarge,
    #[error("Browser Host message is malformed")]
    Decode(#[source] serde_json::Error),
    #[error("Browser Host snapshot could not be encoded")]
    Encode(#[source] serde_json::Error),
    #[error("Browser Host first-party catalog could not be assembled")]
    Catalog(#[from] motolii_plugin::PluginContractError),
    #[error("Browser Host expected exactly one Opacity contract, found {actual}")]
    OpacityCatalogCardinality { actual: usize },
    #[error("Browser Host Opacity contract field `{field}` mismatched")]
    OpacityContractMismatch { field: &'static str },
    #[error("Browser Host codec version mismatch")]
    Version,
    #[error("Browser Host direction mismatch")]
    Direction,
    #[error("Browser Host role mismatch")]
    Role,
    #[error("Browser Host intent kind mismatch")]
    Kind,
    #[error("Browser Host {field} must be a canonical decimal u64")]
    UnsignedInteger { field: &'static str },
    #[error("Browser Host message belongs to a stale WebView instance")]
    StaleInstance,
    #[error("Browser Host sequence exhausted")]
    SequenceExhausted,
    #[error("Browser Host sequence mismatch: expected {expected}, got {actual}")]
    Sequence { expected: u64, actual: u64 },
    #[error("Browser Host {field} must be a non-empty bounded identifier")]
    Identifier { field: &'static str },
    #[error("Browser Host source does not match the admitted Rectangle source")]
    Source,
    #[error("Browser Host inbox is full")]
    InboxFull,
}

#[cfg(test)]
mod tests {
    use motolii_plugin::{PluginCatalogBuilder, PluginContractError};

    use super::*;

    fn intent() -> BrowserPlaceIntent {
        BrowserPlaceIntent {
            scope_ref: "catalog-scope-2".into(),
            item_id: "rectangle".into(),
        }
    }

    fn message(epoch: &str, sequence: &str) -> String {
        format!(
            r#"{{"version":1,"direction":"web-to-host","role":"browser","instance_epoch":"{epoch}","sequence":"{sequence}","kind":"browser.place","source":{{"scope_ref":"catalog-scope-2","item_id":"rectangle"}}}}"#
        )
    }

    #[test]
    fn snapshot_and_place_delivery_match_the_web_codec() {
        let mut session = BrowserHostSession::new(7, 10, intent());
        let expected_catalog = serde_json::json!({
            "catalog_revision": 1,
            "vocabularies": {
                "scopes": [{
                    "id": "first-party-effects",
                    "label": "First-party effects",
                    "scope_ref": null
                }],
                "taxonomies": [
                    {"id": "effect", "label": "Effect", "scope_ref": "first-party-effects"},
                    {"id": "color", "label": "Color", "scope_ref": "first-party-effects"}
                ],
                "providers": [{
                    "id": "built-in",
                    "label": "Built-in",
                    "scope_ref": "first-party-effects"
                }],
                "packs": [],
                "install_states": [{
                    "id": "installed",
                    "label": "Installed",
                    "scope_ref": "first-party-effects"
                }],
                "impact_units": [],
                "tags": []
            },
            "catalogs": [{
                "scope_ref": "first-party-effects",
                "items": [{
                    "item_id": "core.filter.opacity",
                    "display_name": "Opacity",
                    "taxonomy_refs": ["effect", "color"],
                    "provider_ref": "built-in",
                    "pack_ref": null,
                    "install_state_ref": "installed",
                    "preview_kind": "poster",
                    "impact": null,
                    "tag_refs": []
                }]
            }]
        });
        let snapshot: serde_json::Value =
            serde_json::from_str(&session.snapshot_json().unwrap()).unwrap();
        assert_eq!(
            snapshot,
            serde_json::json!({
                "version": 1,
                "direction": "host-to-web",
                "role": "browser",
                "instance_epoch": "7",
                "sequence": "10",
                "browser": {
                    "rectangle_source": {
                        "scope_ref": "catalog-scope-2",
                        "item_id": "rectangle"
                    },
                    "catalog": expected_catalog
                }
            })
        );
        session.accept(&message("7", "11")).unwrap();
        assert_eq!(session.pop(), Some(intent()));
        assert_eq!(session.pop(), None);
    }

    #[test]
    fn opacity_snapshot_rejects_missing_mismatched_and_duplicate_contracts() {
        let empty = PluginCatalogBuilder::new().build().unwrap();
        assert!(matches!(
            opacity_catalog_snapshot(&empty),
            Err(BrowserHostError::OpacityCatalogCardinality { actual: 0 })
        ));

        let mut mismatched = first_party_catalog()
            .unwrap()
            .get(OPACITY_ID)
            .unwrap()
            .clone();
        mismatched.node.display_name = "Not Opacity";
        let mut mismatch_builder = PluginCatalogBuilder::new();
        mismatch_builder.register(mismatched).unwrap();
        assert!(matches!(
            opacity_catalog_snapshot(&mismatch_builder.build().unwrap()),
            Err(BrowserHostError::OpacityContractMismatch {
                field: "display_name"
            })
        ));

        let contract = first_party_catalog()
            .unwrap()
            .get(OPACITY_ID)
            .unwrap()
            .clone();
        let mut duplicate_builder = PluginCatalogBuilder::new();
        duplicate_builder.register(contract.clone()).unwrap();
        let duplicate = duplicate_builder.register(contract).unwrap_err();
        assert!(matches!(
            BrowserHostError::Catalog(duplicate),
            BrowserHostError::Catalog(PluginContractError::DuplicateContract { id: OPACITY_ID })
        ));
    }

    #[test]
    fn rejects_stale_duplicate_gap_unknown_and_oversized_messages() {
        let mut session = BrowserHostSession::new(7, 10, intent());
        assert!(matches!(
            session.accept(&message("6", "11")),
            Err(BrowserHostError::StaleInstance)
        ));
        assert!(matches!(
            session.accept(&message("7", "10")),
            Err(BrowserHostError::Sequence { .. })
        ));
        assert!(matches!(
            session.accept(&message("7", "12")),
            Err(BrowserHostError::Sequence { .. })
        ));
        let wrong_source = message("7", "11").replace(
            r#""item_id":"rectangle""#,
            r#""item_id":"forged-rectangle""#,
        );
        assert!(matches!(
            session.accept(&wrong_source),
            Err(BrowserHostError::Source)
        ));
        assert!(matches!(
            session.accept(&message("07", "11")),
            Err(BrowserHostError::UnsignedInteger { .. })
        ));
        let unknown = message("7", "11").replace(r#""source":{"#, r#""extra":true,"source":{"#);
        assert!(matches!(
            session.accept(&unknown),
            Err(BrowserHostError::Decode(_))
        ));
        assert!(matches!(
            session.accept(&"x".repeat(MAX_MESSAGE_BYTES + 1)),
            Err(BrowserHostError::MessageTooLarge)
        ));
    }

    #[test]
    fn full_inbox_does_not_consume_sequence() {
        let mut session = BrowserHostSession::new(7, 0, intent());
        for sequence in 1..=INBOX_CAPACITY {
            session
                .accept(&message("7", &sequence.to_string()))
                .unwrap();
        }
        assert!(matches!(
            session.accept(&message("7", &(INBOX_CAPACITY + 1).to_string())),
            Err(BrowserHostError::InboxFull)
        ));
        session.pop();
        session
            .accept(&message("7", &(INBOX_CAPACITY + 1).to_string()))
            .unwrap();
    }

    #[test]
    fn replacement_instance_keeps_source_but_rejects_old_epoch() {
        let source = intent();
        let mut replacement = BrowserHostSession::new(8, 0, source.clone());

        assert!(replacement.snapshot_json().unwrap().contains(
            r#""instance_epoch":"8","sequence":"0","browser":{"rectangle_source":{"scope_ref":"catalog-scope-2","item_id":"rectangle"},"catalog":{"catalog_revision":1"#
        ));
        assert!(matches!(
            replacement.accept(&message("7", "1")),
            Err(BrowserHostError::StaleInstance)
        ));
        replacement.accept(&message("8", "1")).unwrap();
        assert_eq!(replacement.pop(), Some(source));
    }
}
