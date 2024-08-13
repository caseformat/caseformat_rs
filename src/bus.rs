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

pub type BusType = usize;

/// PQ bus type.
pub const PQ: BusType = 1;
/// PV bus type.
pub const PV: BusType = 2;
/// Reference bus type.
pub const REF: BusType = 3;
/// Isolated bus type.
pub const NONE: BusType = 4;

#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[builder(setter(into))]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(
    feature = "dataset",
    derive(StructOfArray),
    soa_derive(Serialize, Deserialize)
)]
// #[cfg_attr(feature = "dataset", soa_derive(Serialize, Deserialize))]
pub struct Bus {
    /// Bus number.
    #[builder(setter(custom))]
    pub bus_i: usize,

    /// Bus type.
    #[builder(setter(into = false), default = "PQ")]
    #[validate(range(min = 1, max = 4))]
    pub bus_type: usize,

    /// Real power demand (MW).
    #[builder(default)]
    pub pd: f64,

    /// Reactive power demand (MVAr).
    #[builder(default)]
    pub qd: f64,

    /// Shunt conductance (MW at V = 1.0 p.u.).
    #[builder(default)]
    pub gs: f64,

    /// Shunt susceptance (MVAr at V = 1.0 p.u.).
    #[builder(default)]
    pub bs: f64,

    /// Area number, 1-100.
    // #[serde(rename = "BUS_AREA")]
    #[builder(setter(into = false), default = "1")]
    pub bus_area: usize,

    /// Voltage magnitude (p.u.).
    #[builder(default = "1.0")]
    pub vm: f64,

    /// Voltage angle (degrees).
    #[builder(default)]
    pub va: f64,

    /// Base voltage (kV).
    pub base_kv: f64,

    /// Loss zone.
    #[builder(setter(into = false), default = "1")]
    pub zone: usize,

    /// Maximum voltage magnitude (p.u.).
    #[builder(default = "f64::INFINITY")]
    pub vmax: f64,

    /// Minimum voltage magnitude (p.u.).
    #[builder(default = "f64::NEG_INFINITY")]
    pub vmin: f64,

    /// Lagrange multiplier on real power mismatch (u/MW).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lam_p: Option<f64>,

    /// Lagrange multiplier on reactive power mismatch (u/MVAr).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lam_q: Option<f64>,

    /// Kuhn-Tucker multiplier on upper voltage limit (u/p.u.).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_vmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower voltage limit (u/p.u.).
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mu_vmin: Option<f64>,
}

impl Bus {
    /// Build a new [Bus].
    pub fn new(bus_i: usize) -> BusBuilder {
        BusBuilder {
            bus_i: Some(bus_i),
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Bus {
    /// Fixed active and reactive power.
    pub fn is_pq(&self) -> bool {
        self.bus_type == PQ
    }

    /// Fixed voltage magnitude and active power.
    pub fn is_pv(&self) -> bool {
        self.bus_type == PV
    }

    /// Voltage angle reference. Slack active and reactive power.
    pub fn is_ref(&self) -> bool {
        self.bus_type == REF
    }

    /// Isolated bus.
    pub fn is_isolated(&self) -> bool {
        !(self.is_pq() || self.is_pv() || self.is_ref())
    }

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.lam_p.is_some()
            && self.lam_q.is_some()
            && self.mu_vmax.is_some()
            && self.mu_vmin.is_some()
    }
}

impl Bus {
    pub(crate) fn to_string_record(&self, is_opf: bool) -> StringRecord {
        let mut record = StringRecord::new();

        record.push_field(&format!("{}", self.bus_i));
        record.push_field(&format!("{}", self.bus_type));
        record.push_field(&format!("{}", self.pd));
        record.push_field(&format!("{}", self.qd));
        record.push_field(&format!("{}", self.gs));
        record.push_field(&format!("{}", self.bs));
        record.push_field(&format!("{}", self.bus_area));
        record.push_field(&format!("{}", self.vm));
        record.push_field(&format!("{}", self.va));
        record.push_field(&format!("{}", self.base_kv));
        record.push_field(&format!("{}", self.zone));
        record.push_field(&format!("{}", self.vmax));
        record.push_field(&format!("{}", self.vmin));

        if is_opf {
            record.push_field(&format!("{}", self.lam_p.unwrap_or_default()));
            record.push_field(&format!("{}", self.lam_q.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_vmax.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_vmin.unwrap_or_default()));
        }

        record
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self> {
        let mut iter = record.iter();

        Ok(Self {
            bus_i: parse_record!(iter, usize),
            bus_type: parse_record!(iter, usize),
            pd: parse_record!(iter, f64),
            qd: parse_record!(iter, f64),
            gs: parse_record!(iter, f64),
            bs: parse_record!(iter, f64),
            bus_area: parse_record!(iter, usize),
            vm: parse_record!(iter, f64),
            va: parse_record!(iter, f64),
            base_kv: parse_record!(iter, f64),
            zone: parse_record!(iter, usize),
            vmax: parse_record!(iter, f64),
            vmin: parse_record!(iter, f64),

            lam_p: parse_optional_record!(iter, f64),
            lam_q: parse_optional_record!(iter, f64),
            mu_vmax: parse_optional_record!(iter, f64),
            mu_vmin: parse_optional_record!(iter, f64),
        })
    }
}

impl BusBuilder {
    /// PQ bus type.
    pub fn pq(&mut self) -> &mut Self {
        self.bus_type = Some(1);
        self
    }

    /// PV bus type.
    pub fn pv(&mut self) -> &mut Self {
        self.bus_type = Some(2);
        self
    }

    /// Reference bus type.
    pub fn slack(&mut self) -> &mut Self {
        self.bus_type = Some(3);
        self
    }
}
