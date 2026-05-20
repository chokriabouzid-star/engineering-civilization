#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_fitness::fitness::FitnessVector;
use serde::Deserialize;
use std::collections::HashMap;

/// سياسة قابلة للتحميل من ملف TOML.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PolicyValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

/// مجموعة من السياسات تمثل مجالاً معيناً (e.g., web_api)
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct PolicySet {
    #[serde(flatten)]
    pub values: HashMap<String, PolicyValue>,
}

impl PolicySet {
    /// تحميل مجموعة سياسات من نص TOML.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|v| match v {
            PolicyValue::Float(f) => Some(*f),
            PolicyValue::Integer(i) => Some(*i as f64),
            _ => None,
        })
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| match v {
            PolicyValue::Integer(i) => Some(*i),
            _ => None,
        })
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| match v {
            PolicyValue::String(s) => Some(s.as_str()),
            _ => None,
        })
    }
}

/// ثابت يفرض سياسة محددة من مجموعة سياسات.
#[derive(Debug, Clone)]
pub struct PolicyInvariant {
    pub policy_key: String,
    pub fitness_dimension: fn(&mut FitnessVector) -> &mut f64,
}
