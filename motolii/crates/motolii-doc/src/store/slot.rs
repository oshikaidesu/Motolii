
use serde::{Deserialize, Serialize};

use crate::doc::store::{LayerId, PropertyId, StoreError};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId(pub String);

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub id: SlotId,
    pub track: crate::doc::eval::KeyframeTrack,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PropertySource {
    pub base: Option<PropertyBase>,
    pub modulators: Vec<PropertyLink>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyBase {
    Track(crate::doc::eval::KeyframeTrack),
    Slot(SlotId),
    /// 動かない値。**キーを持たずに値だけ在る**状態(AE の形)。
    /// これが無いと、動かない値も1つのキーで表すしかなく、
    /// 利用者が触っていないのに菱形が出る。
    Constant(crate::doc::eval::Value),
}

impl PropertySource {
    pub fn track(track: crate::doc::eval::KeyframeTrack) -> Self {
        Self {
            base: Some(PropertyBase::Track(track)),
            modulators: Vec::new(),
        }
    }

    pub fn constant(value: crate::doc::eval::Value) -> Self {
        Self {
            base: Some(PropertyBase::Constant(value)),
            modulators: Vec::new(),
        }
    }

    pub fn slot(id: SlotId) -> Self {
        Self {
            base: Some(PropertyBase::Slot(id)),
            modulators: Vec::new(),
        }
    }

    pub fn link_only(link: PropertyLink) -> Self {
        Self {
            base: None,
            modulators: vec![link],
        }
    }

    pub fn as_link_only(&self) -> Option<&PropertyLink> {
        match (&self.base, self.modulators.as_slice()) {
            (None, [link]) => Some(link),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for PropertySource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Keys,
            Base,
            Modulators,
            SourceLayer,
            SourceProperty,
            TimeOffset,
            PluginId,
            Params,
        }

        struct PropertySourceVisitor;

        impl<'de> serde::de::Visitor<'de> for PropertySourceVisitor {
            type Value = PropertySource;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(
                    "a PropertySource: bare KeyframeTrack object, bare Slot string, \
                     legacy Link object, or {\"base\":...,\"modulators\":[...]}",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PropertySource::slot(SlotId(v.to_owned())))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PropertySource::slot(SlotId(v)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde::de::Error as _;

                let Some(first_key) = map.next_key::<Field>()? else {
                    return Err(A::Error::custom(
                        "PropertySource: 空オブジェクトはどの形式にも合わない",
                    ));
                };

                match first_key {
                    Field::Keys => {
                        let keys: Vec<crate::doc::eval::Keyframe> = map.next_value()?;
                        while map
                            .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                            .is_some()
                        {}
                        let track = crate::doc::eval::KeyframeTrack::try_from_keys(keys)
                            .map_err(A::Error::custom)?;
                        Ok(PropertySource::track(track))
                    }
                    Field::Base | Field::Modulators => {
                        let mut base: Option<PropertyBase> = None;
                        let mut modulators: Option<Vec<PropertyLink>> = None;
                        let mut key = Some(first_key);
                        loop {
                            let field = match key.take() {
                                Some(f) => f,
                                None => match map.next_key::<Field>()? {
                                    Some(f) => f,
                                    None => break,
                                },
                            };
                            match field {
                                Field::Base => base = map.next_value()?,
                                Field::Modulators => modulators = Some(map.next_value()?),
                                _ => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                }
                            }
                        }
                        let modulators = modulators
                            .ok_or_else(|| A::Error::missing_field("modulators"))?;
                        Ok(PropertySource { base, modulators })
                    }
                    _ => {
                        let mut source_layer = None;
                        let mut source_property = None;
                        let mut time_offset = None;
                        let mut plugin_id = None;
                        let mut params = None;
                        let mut key = Some(first_key);
                        loop {
                            let field = match key.take() {
                                Some(f) => f,
                                None => match map.next_key::<Field>()? {
                                    Some(f) => f,
                                    None => break,
                                },
                            };
                            match field {
                                Field::SourceLayer => source_layer = Some(map.next_value()?),
                                Field::SourceProperty => {
                                    source_property = Some(map.next_value()?)
                                }
                                Field::TimeOffset => time_offset = Some(map.next_value()?),
                                Field::PluginId => plugin_id = Some(map.next_value()?),
                                Field::Params => params = Some(map.next_value()?),
                                Field::Keys | Field::Base | Field::Modulators => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                }
                            }
                        }
                        let link = PropertyLink {
                            source_layer: source_layer
                                .ok_or_else(|| A::Error::missing_field("source_layer"))?,
                            source_property: source_property
                                .ok_or_else(|| A::Error::missing_field("source_property"))?,
                            time_offset: time_offset
                                .ok_or_else(|| A::Error::missing_field("time_offset"))?,
                            plugin_id: plugin_id
                                .ok_or_else(|| A::Error::missing_field("plugin_id"))?,
                            params: params.ok_or_else(|| A::Error::missing_field("params"))?,
                        };
                        Ok(PropertySource::link_only(link))
                    }
                }
            }
        }

        deserializer.deserialize_any(PropertySourceVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyLink {
    pub source_layer: LayerId,
    pub source_property: PropertyId,
    pub time_offset: crate::doc::core::RationalTime,
    pub plugin_id: String,
    pub params: Vec<(String, crate::doc::eval::Value)>,
}

pub(crate) fn translate_link(
    plugin_id: &str,
    params: &[(String, crate::doc::eval::Value)],
    value: crate::doc::eval::Value,
) -> Option<crate::doc::eval::Value> {
    use crate::doc::eval::Value;

    let find_f64 = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let find_bool = |name: &str, default: bool| -> Option<bool> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, Value::Bool(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };

    match plugin_id {
        "motolii.link.identity" => Some(value),
        "motolii.link.linear" => {
            let scale = find_f64("scale", 1.0)?;
            let offset = find_f64("offset", 0.0)?;
            match value {
                Value::F64(v) => Some(Value::F64(v * scale + offset)),
                Value::Vec2(v) => {
                    Some(Value::Vec2(std::array::from_fn(|i| v[i] * scale + offset)))
                }
                _ => None,
            }
        }
        "motolii.link.remap" => {
            let in_min = find_f64("in_min", 0.0)?;
            let in_max = find_f64("in_max", 1.0)?;
            let out_min = find_f64("out_min", 0.0)?;
            let out_max = find_f64("out_max", 1.0)?;
            let clamp = find_bool("clamp", false)?;
            let Value::F64(v) = value else {
                return None;
            };
            if in_max == in_min {
                return None; // 区間の長さ0は写像が定義できない。
            }
            let mut u = (v - in_min) / (in_max - in_min);
            if clamp {
                u = u.clamp(0.0, 1.0);
            }
            Some(Value::F64(out_min + u * (out_max - out_min)))
        }
        _ => None,
    }
}

pub(crate) fn validate_unique_ids(slots: &[Slot]) -> Result<(), StoreError> {
    for (i, slot) in slots.iter().enumerate() {
        if slots[..i].iter().any(|other| other.id == slot.id) {
            return Err(StoreError::Property(format!(
                "スロット id \"{}\" が2枚ある",
                slot.id
            )));
        }
    }
    Ok(())
}
