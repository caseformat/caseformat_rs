use serde::{Deserialize, Serialize};
use validator::Validate;

/// Generator or dispatchable load.
#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[validate(schema(function = "crate::validate::validate_gen"))]
pub struct Gen {
    /// Bus number.
    #[serde(rename = "GEN_BUS")]
    #[validate(range(min = 1))]
    pub bus: usize,

    /// Real power output (MW).
    pub pg: f64,

    /// Reactive power output (MVAr).
    pub qg: f64,

    /// Maximum reactive power output (MVAr).
    pub qmax: f64,

    /// Minimum reactive power output (MVAr).
    pub qmin: f64,

    /// Voltage magnitude setpoint (p.u.).
    pub vg: f64,

    /// Total MVA base of this machine, defaults to base_mva.
    pub mbase: f64,

    /// Machine status.
    #[serde(rename = "GEN_STATUS")]
    #[validate(range(min = 0, max = 1))]
    pub status: usize,

    /// Maximum real power output (MW).
    pub pmax: f64,

    /// Minimum real power output (MW).
    pub pmin: f64,

    /// Lower real power output of PQ capability curve (MW).
    pub pc1: Option<f64>,

    /// Upper real power output of PQ capability curve (MW).
    pub pc2: Option<f64>,

    /// Minimum reactive power output at Pc1 (MVAr).
    pub qc1min: Option<f64>,

    /// Maximum reactive power output at Pc1 (MVAr).
    pub qc1max: Option<f64>,

    /// Minimum reactive power output at Pc2 (MVAr).
    pub qc2min: Option<f64>,

    /// Maximum reactive power output at Pc2 (MVAr).
    pub qc2max: Option<f64>,

    /// Ramp rate for load following/AGC (MW/min).
    pub ramp_agc: Option<f64>,

    /// Ramp rate for 10 minute reserves (MW).
    pub ramp_10: Option<f64>,

    /// Ramp rate for 30 minute reserves (MW).
    pub ramp_30: Option<f64>,

    /// Ramp rate for reactive power (2 sec timescale) (MVAr/min).
    pub ramp_q: Option<f64>,

    /// Area participation factor.
    pub apf: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Pg limit (u/MW).
    pub mu_pmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Pg limit (u/MW).
    pub mu_pmin: Option<f64>,

    /// Kuhn-Tucker multiplier on upper Qg limit (u/MVAr).
    pub mu_qmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower Qg limit (u/MVAr).
    pub mu_qmin: Option<f64>,
}

impl Gen {
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

    /// Is Optimal Power Flow (OPF) result.
    pub fn is_opf(&self) -> bool {
        self.mu_pmax.is_some()
            && self.mu_pmin.is_some()
            && self.mu_qmax.is_some()
            && self.mu_qmin.is_some()
    }
}
