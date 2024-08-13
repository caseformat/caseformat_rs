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

/// Generator or dispatchable load.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder, PartialEq)]
#[cfg_attr(
target_arch = "wasm32",
derive(Tsify),
tsify(into_wasm_abi, from_wasm_abi)
)]
#[builder(setter(into))]
#[validate(schema(function = "crate::validate::validate_gen"))]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(
feature = "dataset",
derive(StructOfArray),
soa_derive(Serialize, Deserialize)
)]
pub struct Gen {
    /// Bus number.
    // #[serde(rename = "GEN_BUS")]
    #[builder(setter(custom))]
    #[validate(range(min = 1))]
    pub gen_bus: usize,

    /// Real power output (MW).
    #[builder(default)]
    pub pg: f64,

    /// Reactive power output (MVAr).
    #[builder(default)]
    pub qg: f64,

    /// Maximum reactive power output (MVAr).
    #[builder(default = "f64::INFINITY")]
    pub qmax: f64,

    /// Minimum reactive power output (MVAr).
    #[builder(default = "f64::NEG_INFINITY")]
    pub qmin: f64,

    /// Voltage magnitude setpoint (p.u.).
    #[builder(default = "1.0")]
    pub vg: f64,

    /// Total MVA base of this machine, defaults to base_mva.
    #[builder(default)]
    pub mbase: f64,

    /// Machine status.
    // #[serde(rename = "GEN_STATUS")]
    #[builder(setter(into = false), default = "1")]
    #[validate(range(min = 0, max = 1))]
    pub gen_status: usize,

    /// Maximum real power output (MW).
    #[builder(default = "f64::INFINITY")]
    pub pmax: f64,

    /// Minimum real power output (MW).
    #[builder(default = "f64::NEG_INFINITY")]
    pub pmin: f64,

    /// Lower real power output of PQ capability curve (MW).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pc1: Option<f64>,

    /// Upper real power output of PQ capability curve (MW).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pc2: Option<f64>,

    /// Minimum reactive power output at Pc1 (MVAr).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc1min: Option<f64>,

    /// Maximum reactive power output at Pc1 (MVAr).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc1max: Option<f64>,

    /// Minimum reactive power output at Pc2 (MVAr).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc2min: Option<f64>,

    /// Maximum reactive power output at Pc2 (MVAr).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc2max: Option<f64>,

    /// Ramp rate for load following/AGC (MW/min).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ramp_agc: Option<f64>,

    /// Ramp rate for 10 minute reserves (MW).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ramp_10: Option<f64>,

    /// Ramp rate for 30 minute reserves (MW).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ramp_30: Option<f64>,

    /// Ramp rate for reactive power (2 sec timescale) (MVAr/min).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ramp_q: Option<f64>,

    /// Area participation factor.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apf: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Pg limit (u/MW).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_pmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Pg limit (u/MW).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_pmin: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Qg limit (u/MVAr).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_qmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Qg limit (u/MVAr).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_qmin: Option<f64>,
}

impl Gen {
    /// Build new [Gen].
    pub fn new(gen_bus: usize) -> GenBuilder {
        GenBuilder {
            gen_bus: Some(gen_bus),
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Gen {
    /// Machine in-service.
    pub fn is_on(&self) -> bool {
        self.gen_status != 0
    }

    /// Machine out-of-service.
    pub fn is_off(&self) -> bool {
        self.gen_status == 0
    }

    /// Checks for dispatchable loads.
    pub fn is_load(&self) -> bool {
        self.pmin < 0.0 && self.pmax == 0.0
    }

    /// Columns `PC1` to `APF` are not included in version 1 case format.
    pub fn is_version_1(&self) -> bool {
        self.pc1.is_none()
            && self.pc2.is_none()
            && self.qc1min.is_none()
            && self.qc1max.is_none()
            && self.qc2min.is_none()
            && self.qc2max.is_none()
            && self.ramp_agc.is_none()
            && self.ramp_10.is_none()
            && self.ramp_30.is_none()
            && self.ramp_q.is_none()
            && self.apf.is_none()
    }

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.mu_pmax.is_some()
            && self.mu_pmin.is_some()
            && self.mu_qmax.is_some()
            && self.mu_qmin.is_some()
    }
}

impl Gen {
    pub(crate) fn to_string_record(&self, is_version_1: bool, is_opf: bool) -> StringRecord {
        let mut record = StringRecord::new();

        record.push_field(&format!("{}", self.gen_bus));
        record.push_field(&format!("{}", self.pg));
        record.push_field(&format!("{}", self.qg));
        record.push_field(&format!("{}", self.qmax));
        record.push_field(&format!("{}", self.qmin));
        record.push_field(&format!("{}", self.vg));
        record.push_field(&format!("{}", self.mbase));
        record.push_field(&format!("{}", self.gen_status));
        record.push_field(&format!("{}", self.pmax));
        record.push_field(&format!("{}", self.pmin));

        if !is_version_1 {
            record.push_field(&format!("{}", self.pc1.unwrap()));
            record.push_field(&format!("{}", self.pc2.unwrap()));
            record.push_field(&format!("{}", self.qc1min.unwrap()));
            record.push_field(&format!("{}", self.qc1max.unwrap()));
            record.push_field(&format!("{}", self.qc2min.unwrap()));
            record.push_field(&format!("{}", self.qc2max.unwrap()));
            record.push_field(&format!("{}", self.ramp_agc.unwrap()));
            record.push_field(&format!("{}", self.ramp_10.unwrap()));
            record.push_field(&format!("{}", self.ramp_30.unwrap()));
            record.push_field(&format!("{}", self.ramp_q.unwrap()));
            record.push_field(&format!("{}", self.apf.unwrap()));
        }

        if is_opf {
            record.push_field(&format!("{}", self.mu_pmax.unwrap()));
            record.push_field(&format!("{}", self.mu_pmin.unwrap()));
            record.push_field(&format!("{}", self.mu_qmax.unwrap()));
            record.push_field(&format!("{}", self.mu_qmin.unwrap()));
        }

        record
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self> {
        let mut iter = record.iter();

        Ok(Self {
            gen_bus: parse_record!(iter, usize),
            pg: parse_record!(iter, f64),
            qg: parse_record!(iter, f64),
            qmax: parse_record!(iter, f64),
            qmin: parse_record!(iter, f64),
            vg: parse_record!(iter, f64),
            mbase: parse_record!(iter, f64),
            gen_status: parse_record!(iter, usize),
            pmax: parse_record!(iter, f64),
            pmin: parse_record!(iter, f64),

            pc1: parse_optional_record!(iter, f64),
            pc2: parse_optional_record!(iter, f64),
            qc1min: parse_optional_record!(iter, f64),
            qc1max: parse_optional_record!(iter, f64),
            qc2min: parse_optional_record!(iter, f64),
            qc2max: parse_optional_record!(iter, f64),
            ramp_agc: parse_optional_record!(iter, f64),
            ramp_10: parse_optional_record!(iter, f64),
            ramp_30: parse_optional_record!(iter, f64),
            ramp_q: parse_optional_record!(iter, f64),
            apf: parse_optional_record!(iter, f64),

            mu_pmax: parse_optional_record!(iter, f64),
            mu_pmin: parse_optional_record!(iter, f64),
            mu_qmax: parse_optional_record!(iter, f64),
            mu_qmin: parse_optional_record!(iter, f64),
        })
    }
}

impl GenBuilder {
    /// In-service gen status.
    pub fn in_service(&mut self) -> &mut Self {
        self.gen_status = Some(1);
        self
    }

    /// Out-of-service gen status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.gen_status = Some(0);
        self
    }
}
