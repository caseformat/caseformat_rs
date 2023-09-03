use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Power flow case.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder)]
#[builder(setter(into))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Case {
    /// Case name.
    #[serde(rename = "CASENAME")]
    pub name: String,

    /// Case format version.
    #[builder(default = "String::from(\"2\")")]
    pub version: String,

    /// System MVA base.
    #[builder(default = "100.0")]
    #[validate(range(min = 1))]
    pub base_mva: f64,

    /// Total system cost (US dollars).
    #[builder(setter(strip_option), default)]
    pub f: Option<f64>,
}

impl Case {
    /// Build new [Case].
    pub fn new(name: impl Into<String>) -> CaseBuilder {
        CaseBuilder {
            name: Some(name.into()),
            ..Default::default()
        }
    }
}
