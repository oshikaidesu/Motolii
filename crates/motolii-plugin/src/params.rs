//! 解決済みパラメータの型付き取り出し。

use std::collections::{BTreeSet, HashMap};

use motolii_eval::Value;

use crate::contract::{value_matches_type, value_type_name, NodeDesc, PluginError, ValueType};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ResolvedParams {
    values: HashMap<&'static str, Value>,
}

impl ResolvedParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: &'static str, value: Value) {
        self.values.insert(id, value);
    }

    pub fn get(&self, id: &'static str) -> Option<&Value> {
        self.values.get(id)
    }

    /// サイレントフォールバックは「もっともらしく間違う絵」の温床(M2E-8)。新規コードは`require_f64`。
    #[deprecated(note = "use require_f64; silent fallback hides type mistakes")]
    pub fn f64_or(&self, id: &'static str, fallback: f64) -> f64 {
        self.get(id).and_then(Value::as_f64).unwrap_or(fallback)
    }

    pub fn require_f64(&self, plugin: &str, id: &'static str) -> Result<f64, PluginError> {
        match self.get(id) {
            Some(Value::F64(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::F64,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::F64)),
        }
    }

    pub fn require_color(&self, plugin: &str, id: &'static str) -> Result<[f64; 4], PluginError> {
        match self.get(id) {
            Some(Value::Color(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::Color,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::Color)),
        }
    }

    pub fn require_vec2(&self, plugin: &str, id: &'static str) -> Result<[f64; 2], PluginError> {
        match self.get(id) {
            Some(Value::Vec2(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::Vec2,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::Vec2)),
        }
    }
}

impl NodeDesc {
    /// 生JSON params を desc に照合して解決する(M2E-8)。
    /// 未知ID→Err、型不一致→Err、欠落→`ParamDef.default` 充填。
    pub fn resolve_params(
        &self,
        raw: &HashMap<String, Value>,
    ) -> Result<ResolvedParams, PluginError> {
        let plugin = self.id.0;
        let known: BTreeSet<&str> = self.params.iter().map(|p| p.id).collect();
        for key in raw.keys() {
            if !known.contains(key.as_str()) {
                return Err(PluginError::Param {
                    plugin: plugin.to_string(),
                    id: key.clone(),
                    expected: "defined in NodeDesc".into(),
                    got: "unknown".into(),
                });
            }
        }

        let mut params = ResolvedParams::new();
        for def in &self.params {
            let value = match raw.get(def.id) {
                Some(v) if value_matches_type(def.value_type, v) => v.clone(),
                Some(v) => {
                    return Err(PluginError::param_type(
                        plugin,
                        def.id,
                        def.value_type,
                        value_type_name(v),
                    ));
                }
                None => def.default.clone(),
            };
            params.insert(def.id, value);
        }
        Ok(params)
    }
}
