use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Generator or dispatchable load.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[builder(setter(into))]
#[validate(schema(function = "crate::validate::validate_gen"))]
pub struct Gen {
    /// Bus number.
    #[serde(rename = "GEN_BUS")]
    #[builder(setter(custom))]
    #[validate(range(min = 1))]
    pub bus: usize,

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
    #[serde(rename = "GEN_STATUS")]
    #[builder(setter(into = false), default = "1")]
    #[validate(range(min = 0, max = 1))]
    pub status: usize,

    /// Maximum real power output (MW).
    #[builder(default = "f64::INFINITY")]
    pub pmax: f64,

    /// Minimum real power output (MW).
    #[builder(default = "f64::NEG_INFINITY")]
    pub pmin: f64,

    /// Lower real power output of PQ capability curve (MW).
    #[builder(setter(strip_option), default)]
    pub pc1: Option<f64>,

    /// Upper real power output of PQ capability curve (MW).
    #[builder(setter(strip_option), default)]
    pub pc2: Option<f64>,

    /// Minimum reactive power output at Pc1 (MVAr).
    #[builder(setter(strip_option), default)]
    pub qc1min: Option<f64>,

    /// Maximum reactive power output at Pc1 (MVAr).
    #[builder(setter(strip_option), default)]
    pub qc1max: Option<f64>,

    /// Minimum reactive power output at Pc2 (MVAr).
    #[builder(setter(strip_option), default)]
    pub qc2min: Option<f64>,

    /// Maximum reactive power output at Pc2 (MVAr).
    #[builder(setter(strip_option), default)]
    pub qc2max: Option<f64>,

    /// Ramp rate for load following/AGC (MW/min).
    #[builder(setter(strip_option), default)]
    pub ramp_agc: Option<f64>,

    /// Ramp rate for 10 minute reserves (MW).
    #[builder(setter(strip_option), default)]
    pub ramp_10: Option<f64>,

    /// Ramp rate for 30 minute reserves (MW).
    #[builder(setter(strip_option), default)]
    pub ramp_30: Option<f64>,

    /// Ramp rate for reactive power (2 sec timescale) (MVAr/min).
    #[builder(setter(strip_option), default)]
    pub ramp_q: Option<f64>,

    /// Area participation factor.
    #[builder(setter(strip_option), default)]
    pub apf: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Pg limit (u/MW).
    #[builder(setter(custom), default)]
    pub mu_pmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Pg limit (u/MW).
    #[builder(setter(custom), default)]
    pub mu_pmin: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Qg limit (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Qg limit (u/MVAr).
    #[builder(setter(custom), default)]
    pub mu_qmin: Option<f64>,
}

impl Gen {
    /// Build new [Gen].
    pub fn new(gen_bus: usize) -> GenBuilder {
        GenBuilder {
            bus: Some(gen_bus),
            ..Default::default()
        }
    }

    /// Machine in-service.
    pub fn is_on(&self) -> bool {
        self.status != 0
    }

    /// Machine out-of-service.
    pub fn is_off(&self) -> bool {
        self.status == 0
    }

    /// Checks for dispatchable loads.
    pub fn is_load(&self) -> bool {
        self.pmin < 0.0 && self.pmax == 0.0
    }

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.mu_pmax.is_some()
            && self.mu_pmin.is_some()
            && self.mu_qmax.is_some()
            && self.mu_qmin.is_some()
    }
}

impl GenBuilder {
    /// In-service gen status.
    pub fn in_service(&mut self) -> &mut Self {
        self.status = Some(1);
        self
    }

    /// Out-of-service gen status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.status = Some(0);
        self
    }
}
