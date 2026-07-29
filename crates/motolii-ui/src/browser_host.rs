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
}

#[derive(Debug, Serialize)]
struct WireSource<'a> {
    scope_ref: &'a str,
    item_id: &'a str,
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
        assert_eq!(
            session.snapshot_json().unwrap(),
            r#"{"version":1,"direction":"host-to-web","role":"browser","instance_epoch":"7","sequence":"10","browser":{"rectangle_source":{"scope_ref":"catalog-scope-2","item_id":"rectangle"}}}"#
        );
        session.accept(&message("7", "11")).unwrap();
        assert_eq!(session.pop(), Some(intent()));
        assert_eq!(session.pop(), None);
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
}
