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

#[cfg(feature = "dataset")]
use soa_derive::StructOfArray;

/// Transmission line/cable or two winding transformer.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[builder(setter(into))]
#[validate(schema(function = "crate::validate::validate_branch"))]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(
    feature = "dataset",
    derive(StructOfArray),
    soa_derive(Serialize, Deserialize)
)]
pub struct Branch {
    /// "from" bus number.
    #[builder(setter(custom))]
    #[validate(range(min = 1))]
    pub f_bus: usize,

    /// "to" bus number.
    #[builder(setter(custom))]
    #[validate(range(min = 1))]
    pub t_bus: usize,

    /// Resistance (p.u.).
    // #[serde(rename = "BR_R")]
    #[builder(default)]
    pub br_r: f64,

    /// Reactance (p.u.).
    // #[serde(rename = "BR_X")]
    #[builder(default)]
    pub br_x: f64,

    /// Total line charging susceptance (p.u.).
    // #[serde(rename = "BR_B")]
    #[builder(default)]
    pub br_b: f64,

    /// MVA rating A (long term rating).
    #[builder(default)]
    pub rate_a: f64,

    /// MVA rating B (short term rating) (MVA).
    #[builder(default)]
    pub rate_b: f64,

    /// MVA rating C (emergency rating) (MVA).
    #[builder(default)]
    pub rate_c: f64,

    /// Transformer off nominal tap ratio.
    #[builder(default)]
    pub tap: f64,

    /// Transformer phase shift angle (degrees).
    #[builder(default)]
    pub shift: f64,

    /// Initial branch status.
    // #[serde(rename = "BR_STATUS")]
    #[builder(setter(into = false), default = "1")]
    #[validate(range(min = 0, max = 1))]
    pub br_status: usize,

    /// Minimum angle difference; angle(Vf) - angle(Vt) (degrees).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angmin: Option<f64>,

    /// Maximum angle difference; angle(Vf) - angle(Vt) (degrees).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angmax: Option<f64>,

    /// Real power injected at "from" bus end (MW).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pf: Option<f64>,

    /// Reactive power injected at "from" bus end (MVAr).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qf: Option<f64>,

    /// Real power injected at "to" bus end (MW).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<f64>,

    /// Reactive power injected at "to" bus end (MVAr).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qt: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "from" bus (u/MVA).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_sf: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "to" bus (u/MVA).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_st: Option<f64>,

    /// Kuhn-Tucker multiplier lower angle difference limit (u/degree).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_angmin: Option<f64>,

    /// Kuhn-Tucker multiplier upper angle difference limit (u/degree).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_angmax: Option<f64>,
}

impl Branch {
    /// Build new [Branch].
    pub fn new(f_bus: usize, t_bus: usize) -> BranchBuilder {
        BranchBuilder {
            f_bus: Some(f_bus),
            t_bus: Some(t_bus),
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Branch {
    /// Branch is in-service.
    pub fn is_on(&self) -> bool {
        self.br_status != 0
    }

    /// Branch is out-of-service.
    pub fn is_off(&self) -> bool {
        self.br_status == 0
    }

    /// Branch is a transformer.
    pub fn is_transformer(&self) -> bool {
        self.tap != 0.0
    }

    /// Is power flow result.
    pub fn is_pf(&self) -> bool {
        self.pf.is_some() && self.qf.is_some() && self.pt.is_some() && self.qt.is_some()
    }

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.is_pf()
            && self.mu_sf.is_some()
            && self.mu_sf.is_some()
            && self.mu_angmin.is_some()
            && self.mu_angmax.is_some()
    }
}

impl Branch {
    pub(crate) fn to_string_record(&self, is_pf: bool, is_opf: bool) -> StringRecord {
        let mut record = StringRecord::new();

        record.push_field(&format!("{}", self.f_bus));
        record.push_field(&format!("{}", self.t_bus));
        record.push_field(&format!("{}", self.br_r));
        record.push_field(&format!("{}", self.br_x));
        record.push_field(&format!("{}", self.br_b));
        record.push_field(&format!("{}", self.rate_a));
        record.push_field(&format!("{}", self.rate_b));
        record.push_field(&format!("{}", self.rate_c));
        record.push_field(&format!("{}", self.tap));
        record.push_field(&format!("{}", self.shift));
        record.push_field(&format!("{}", self.br_status));
        record.push_field(&format!("{}", self.angmin.unwrap_or_default()));
        record.push_field(&format!("{}", self.angmax.unwrap_or_default()));

        if is_pf {
            record.push_field(&format!("{}", self.pf.unwrap_or_default()));
            record.push_field(&format!("{}", self.qf.unwrap_or_default()));
            record.push_field(&format!("{}", self.pt.unwrap_or_default()));
            record.push_field(&format!("{}", self.qt.unwrap_or_default()));
        }

        if is_opf {
            record.push_field(&format!("{}", self.mu_sf.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_st.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_angmin.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_angmax.unwrap_or_default()));
        }

        record
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self> {
        let mut iter = record.iter();

        Ok(Self {
            f_bus: parse_record!(iter, usize),
            t_bus: parse_record!(iter, usize),
            br_r: parse_record!(iter, f64),
            br_x: parse_record!(iter, f64),
            br_b: parse_record!(iter, f64),
            rate_a: parse_record!(iter, f64),
            rate_b: parse_record!(iter, f64),
            rate_c: parse_record!(iter, f64),
            tap: parse_record!(iter, f64),
            shift: parse_record!(iter, f64),
            br_status: parse_record!(iter, usize),

            angmin: parse_optional_record!(iter, f64),
            angmax: parse_optional_record!(iter, f64),

            pf: parse_optional_record!(iter, f64),
            qf: parse_optional_record!(iter, f64),
            pt: parse_optional_record!(iter, f64),
            qt: parse_optional_record!(iter, f64),

            mu_sf: parse_optional_record!(iter, f64),
            mu_st: parse_optional_record!(iter, f64),
            mu_angmin: parse_optional_record!(iter, f64),
            mu_angmax: parse_optional_record!(iter, f64),
        })
    }
}

impl BranchBuilder {
    /// In-service branch status.
    pub fn in_service(&mut self) -> &mut Self {
        self.br_status = Some(1);
        self
    }

    /// Out-of-service branch status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.br_status = Some(0);
        self
    }
}
