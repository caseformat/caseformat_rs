use anyhow::Result;
use csv::StringRecord;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{parse_optional_record, parse_record};

#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Power flow case.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[builder(setter(into))]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct Case {
    /// Case name.
    #[serde(rename = "casename")]
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

impl Case {
    pub(crate) fn to_string_record(&self) -> StringRecord {
        let mut record = StringRecord::new();

        record.push_field(&format!("{}", self.name));
        record.push_field(&format!("{}", self.version));
        record.push_field(&format!("{}", self.base_mva));

        if let Some(f) = self.f {
            record.push_field(&format!("{}", f));
        }

        record
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self> {
        let mut iter = record.iter();

        Ok(Self {
            name: parse_record!(iter, String),
            version: parse_record!(iter, String),
            base_mva: parse_record!(iter, f64),

            f: parse_optional_record!(iter, f64),
        })
    }
}
