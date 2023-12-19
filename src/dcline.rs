use anyhow::Result;
use csv::StringRecord;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{parse_optional_record, parse_record};

#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

/// Dispatchable DC transmission line.
#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[builder(setter(into))]
pub struct DCLine {
    /// "from" bus number.
    #[builder(setter(custom))]
    pub f_bus: usize,

    /// "to" bus number.
    #[builder(setter(custom))]
    pub t_bus: usize,

    /// Initial DC line status.
    // #[serde(rename = "BR_STATUS")]
    #[builder(setter(into = false), default = "1")]
    pub br_status: usize,

    /// Flow at "from" bus ("from" -> "to") (MW).
    #[builder(default)]
    pub pf: f64,

    /// Flow at "to" bus ("from" -> "to") (MW).
    #[builder(default)]
    pub pt: f64,

    /// Injection at "from" bus ("from" -> "to") (MVAr).
    #[builder(default)]
    pub qf: f64,

    /// Injection at "to" bus ("from" -> "to") (MVAr).
    #[builder(default)]
    pub qt: f64,

    /// Voltage setpoint at "from" bus (p.u.).
    #[builder(default = "1.0")]
    pub vf: f64,

    /// Voltage setpoint at "to" bus (p.u.).
    #[builder(default = "1.0")]
    pub vt: f64,

    /// Lower limit on MW flow at "from" end (MW).
    #[builder(default = "f64::NEG_INFINITY")]
    pub pmin: f64,

    /// Upper limit on MW flow at "from" end (MW).
    #[builder(default = "f64::INFINITY")]
    pub pmax: f64,

    /// Lower limit on MVAr injection at "from" bus (MVAr).
    #[builder(default = "f64::NEG_INFINITY")]
    pub qminf: f64,

    /// Upper limit on MVAr injection at "from" bus (MVAr).
    #[builder(default = "f64::INFINITY")]
    pub qmaxf: f64,

    /// Lower limit on MVAr injection at "to" bus (MVAr).
    #[builder(default = "f64::NEG_INFINITY")]
    pub qmint: f64,

    /// Upper limit on MVAr injection at "to" bus (MVAr).
    #[builder(default = "f64::INFINITY")]
    pub qmaxt: f64,

    /// Constant term of linear loss function (MW).
    #[builder(default)]
    pub loss0: f64,

    /// Linear term of linear loss function (MW).
    #[builder(default)]
    pub loss1: f64,

    /// Kuhn-Tucker multiplier on lower flow lim at "from" bus (u/MW).
    #[builder(setter(custom), default)]
    pub mu_pmin: Option<f64>,

    /// Kuhn-Tucker multiplier on upper flow lim at "from" bus (u/MW).
    #[builder(setter(custom), default)]
    pub mu_pmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower VAr lim at "from" bus (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qminf: Option<f64>,

    /// Kuhn-Tucker multiplier on upper VAr lim at "from" bus (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qmaxf: Option<f64>,

    /// Kuhn-Tucker multiplier on lower VAr lim at "to" bus (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qmint: Option<f64>,

    /// Kuhn-Tucker multiplier on upper VAr lim at "to" bus (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qmaxt: Option<f64>,
}

impl DCLine {
    /// Build new [DCLine].
    pub fn new(f_bus: usize, t_bus: usize) -> DCLineBuilder {
        DCLineBuilder {
            f_bus: Some(f_bus),
            t_bus: Some(t_bus),
            ..Default::default()
        }
    }
}

impl DCLine {
    /// DC line is in-service.
    pub fn is_on(&self) -> bool {
        self.br_status != 0
    }

    /// DC line is out-of-service.
    pub fn is_off(&self) -> bool {
        self.br_status == 0
    }

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.mu_pmin.is_some()
            && self.mu_pmax.is_some()
            && self.mu_qminf.is_some()
            && self.mu_qmaxf.is_some()
            && self.mu_qmint.is_some()
            && self.mu_qmaxt.is_some()
    }
}

impl DCLine {
    pub(crate) fn to_string_record(&self, is_opf: bool) -> StringRecord {
        let mut record = StringRecord::new();

        record.push_field(&format!("{}", self.f_bus));
        record.push_field(&format!("{}", self.t_bus));
        record.push_field(&format!("{}", self.br_status));
        record.push_field(&format!("{}", self.pf));
        record.push_field(&format!("{}", self.pt));
        record.push_field(&format!("{}", self.qf));
        record.push_field(&format!("{}", self.qt));
        record.push_field(&format!("{}", self.vf));
        record.push_field(&format!("{}", self.vt));
        record.push_field(&format!("{}", self.pmin));
        record.push_field(&format!("{}", self.pmax));
        record.push_field(&format!("{}", self.qminf));
        record.push_field(&format!("{}", self.qmaxf));
        record.push_field(&format!("{}", self.qmint));
        record.push_field(&format!("{}", self.qmaxt));
        record.push_field(&format!("{}", self.loss0));

        if is_opf {
            record.push_field(&format!("{}", self.mu_pmin.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_pmax.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_qminf.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_qmaxf.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_qmint.unwrap_or_default()));
            record.push_field(&format!("{}", self.mu_qmaxt.unwrap_or_default()));
        }

        record
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self> {
        let mut iter = record.iter();

        Ok(Self {
            f_bus: parse_record!(iter, usize),
            t_bus: parse_record!(iter, usize),
            br_status: parse_record!(iter, usize),
            pf: parse_record!(iter, f64),
            pt: parse_record!(iter, f64),
            qf: parse_record!(iter, f64),
            qt: parse_record!(iter, f64),
            vf: parse_record!(iter, f64),
            vt: parse_record!(iter, f64),
            pmin: parse_record!(iter, f64),
            pmax: parse_record!(iter, f64),
            qminf: parse_record!(iter, f64),
            qmaxf: parse_record!(iter, f64),
            qmint: parse_record!(iter, f64),
            qmaxt: parse_record!(iter, f64),
            loss0: parse_record!(iter, f64),
            loss1: parse_record!(iter, f64),
            mu_pmin: parse_optional_record!(iter, f64),
            mu_pmax: parse_optional_record!(iter, f64),
            mu_qminf: parse_optional_record!(iter, f64),
            mu_qmaxf: parse_optional_record!(iter, f64),
            mu_qmint: parse_optional_record!(iter, f64),
            mu_qmaxt: parse_optional_record!(iter, f64),
        })
    }
}

impl DCLineBuilder {
    /// In-service DC line status.
    pub fn in_service(&mut self) -> &mut Self {
        self.br_status = Some(1);
        self
    }

    /// Out-of-service DC line status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.br_status = Some(0);
        self
    }
}
