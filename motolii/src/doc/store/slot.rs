
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::RationalTime;
    use crate::doc::eval::{Interp, Keyframe, Value};

    fn hold(value: Value) -> crate::doc::eval::KeyframeTrack {
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value,
            interp: Interp::Hold,
            spatial: None,
        });
        track
    }

    #[test]
    fn property_source_track_round_trips_through_the_explicit_wire_shape() {
        let source = PropertySource::track(hold(Value::F64(1.0)));
        let json = serde_json::to_string(&source).unwrap();
        let back: PropertySource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source, "新形式の往復が一致しない: {json}");
    }

    #[test]
    fn a_bare_keyframe_track_json_still_deserializes_as_a_track_base() {
        let track = hold(Value::F64(1.0));
        let bare_json = serde_json::to_string(&track).unwrap();
        let source: PropertySource = serde_json::from_str(&bare_json).unwrap();
        assert_eq!(source, PropertySource::track(track));
    }

    #[test]
    fn slot_id_serializes_identically_to_a_bare_string() {
        let id = SlotId("primary_color".to_owned());
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"primary_color\"");
    }

    #[test]
    fn duplicate_slot_ids_are_rejected() {
        let slots = vec![
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(1.0)),
            },
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(2.0)),
            },
        ];
        assert!(validate_unique_ids(&slots).is_err());
    }

    #[test]
    fn distinct_slot_ids_are_accepted() {
        let slots = vec![
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(1.0)),
            },
            Slot {
                id: SlotId("b".to_owned()),
                track: hold(Value::F64(2.0)),
            },
        ];
        assert!(validate_unique_ids(&slots).is_ok());
    }

    fn link(source_property: PropertyId, plugin_id: &str, params: Vec<(String, Value)>) -> PropertyLink {
        PropertyLink {
            source_layer: LayerId(1),
            source_property,
            time_offset: RationalTime::ZERO,
            plugin_id: plugin_id.to_owned(),
            params,
        }
    }

    #[test]
    fn translate_link_identity_passes_value_through_unchanged() {
        assert_eq!(
            translate_link("motolii.link.identity", &[], Value::F64(3.5)),
            Some(Value::F64(3.5))
        );
        assert_eq!(
            translate_link("motolii.link.identity", &[], Value::Vec2([1.0, 2.0])),
            Some(Value::Vec2([1.0, 2.0]))
        );
    }

    #[test]
    fn translate_link_linear_applies_scale_and_offset() {
        let params = vec![
            ("scale".to_owned(), Value::F64(2.0)),
            ("offset".to_owned(), Value::F64(10.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.linear", &params, Value::F64(5.0)),
            Some(Value::F64(20.0)),
            "5*2+10 = 20 のはず"
        );
    }

    #[test]
    fn translate_link_linear_defaults_to_identity_when_params_are_absent() {
        assert_eq!(
            translate_link("motolii.link.linear", &[], Value::F64(7.0)),
            Some(Value::F64(7.0))
        );
    }

    #[test]
    fn translate_link_linear_applies_uniformly_to_vec2_components() {
        let params = vec![
            ("scale".to_owned(), Value::F64(0.5)),
            ("offset".to_owned(), Value::F64(1.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.linear", &params, Value::Vec2([10.0, 20.0])),
            Some(Value::Vec2([6.0, 11.0]))
        );
    }

    #[test]
    fn translate_link_remap_maps_between_ranges() {
        let params = vec![
            ("in_min".to_owned(), Value::F64(0.0)),
            ("in_max".to_owned(), Value::F64(10.0)),
            ("out_min".to_owned(), Value::F64(0.0)),
            ("out_max".to_owned(), Value::F64(100.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.remap", &params, Value::F64(5.0)),
            Some(Value::F64(50.0))
        );
    }

    #[test]
    fn translate_link_remap_clamps_only_when_requested() {
        let base = vec![
            ("in_min".to_owned(), Value::F64(0.0)),
            ("in_max".to_owned(), Value::F64(10.0)),
            ("out_min".to_owned(), Value::F64(0.0)),
            ("out_max".to_owned(), Value::F64(100.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.remap", &base, Value::F64(20.0)),
            Some(Value::F64(200.0)),
            "clamp を渡さなければ外挿するはず"
        );

        let mut clamped = base;
        clamped.push(("clamp".to_owned(), Value::Bool(true)));
        assert_eq!(
            translate_link("motolii.link.remap", &clamped, Value::F64(20.0)),
            Some(Value::F64(100.0)),
            "clamp=true なら上限で止まるはず"
        );
    }

    #[test]
    fn translate_link_rejects_type_mismatch_instead_of_approximating() {
        let params = vec![("in_max".to_owned(), Value::F64(10.0))];
        assert_eq!(
            translate_link("motolii.link.remap", &params, Value::Vec2([1.0, 2.0])),
            None
        );
        let bad_params = vec![("scale".to_owned(), Value::Bool(true))];
        assert_eq!(
            translate_link("motolii.link.linear", &bad_params, Value::F64(1.0)),
            None
        );
    }

    #[test]
    fn translate_link_returns_none_for_unknown_plugin_id() {
        assert_eq!(
            translate_link("motolii.link.does_not_exist", &[], Value::F64(1.0)),
            None
        );
    }

    #[test]
    fn property_source_round_trips_for_every_shape() {
        let track = PropertySource::track(hold(Value::F64(1.0)));
        let slot = PropertySource::slot(SlotId("s".to_owned()));
        let link_only = PropertySource::link_only(link(
            PropertyId::new("opacity").unwrap(),
            "motolii.link.identity",
            Vec::new(),
        ));
        let summed = PropertySource {
            base: Some(PropertyBase::Track(hold(Value::F64(2.0)))),
            modulators: vec![link(
                PropertyId::new("rotation").unwrap(),
                "motolii.link.identity",
                Vec::new(),
            )],
        };

        for source in [track, slot, link_only, summed] {
            let json = serde_json::to_string(&source).unwrap();
            let back: PropertySource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, source, "新形式の往復が一致しない: {json}");
        }
    }

    #[test]
    fn a_legacy_bare_link_object_still_deserializes_as_link_only() {
        let l = link(
            PropertyId::new("rotation").unwrap(),
            "motolii.link.linear",
            vec![("scale".to_owned(), Value::F64(2.0))],
        );
        let legacy_json = serde_json::to_string(&l).unwrap();
        let source: PropertySource = serde_json::from_str(&legacy_json).unwrap();
        assert_eq!(source, PropertySource::link_only(l));
    }

    #[test]
    fn a_bare_slot_id_string_still_deserializes_as_a_slot_base() {
        let id = SlotId("brand".to_owned());
        let bare_json = serde_json::to_string(&id).unwrap();
        let source: PropertySource = serde_json::from_str(&bare_json).unwrap();
        assert_eq!(source, PropertySource::slot(id));
    }

    #[test]
    fn property_link_serializes_source_property_as_a_bare_string() {
        let l = link(
            PropertyId::new("rotation").unwrap(),
            "motolii.link.linear",
            vec![("scale".to_owned(), Value::F64(2.0))],
        );
        let json = serde_json::to_value(&l).unwrap();
        assert_eq!(
            json["source_property"], "rotation",
            "source_property が裸の文字列で符号化されていない: {json}"
        );

        let back: PropertyLink = serde_json::from_value(json).unwrap();
        assert_eq!(back, l);
    }

    #[test]
    fn a_reserved_name_in_source_property_fails_to_deserialize() {
        let json = r#"{
            "source_layer": 1,
            "source_property": "masks",
            "time_offset": {"num": 0, "den": 1},
            "plugin_id": "motolii.link.identity",
            "params": []
        }"#;
        assert!(
            serde_json::from_str::<PropertyLink>(json).is_err(),
            "予約語 `masks` を持つ PropertyLink が読めてしまっている"
        );
    }
}

#[cfg(test)]
mod constant_tests {
    use super::*;
    use crate::doc::eval::Value;

    /// 動かない値は、保存して読み直しても track と混ざらない。
    /// `PropertyBase` は untagged なので、並びを崩すと静かに壊れる。
    #[test]
    fn a_constant_survives_the_round_trip_without_becoming_a_track() {
        let source = PropertySource::constant(Value::Vec2([160.0, 90.0]));
        let json = serde_json::to_string(&source).expect("書ける");
        let back: PropertySource = serde_json::from_str(&json).expect("読める");
        assert_eq!(back, source, "往復で形が変わった: {json}");
        assert!(
            matches!(back.base, Some(PropertyBase::Constant(_))),
            "track か slot として読まれた: {json}"
        );
    }
}
