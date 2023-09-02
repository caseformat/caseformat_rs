use serde::{Deserialize, Serialize};
use validator::Validate;

/// A transmission line/cable or a two winding transformer.
#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[validate(schema(function = "crate::validate::validate_branch"))]
pub struct Branch {
    /// "from" bus number.
    #[validate(range(min = 1))]
    pub f_bus: usize,

    /// "to" bus number.
    #[validate(range(min = 1))]
    pub t_bus: usize,

    /// Resistance (p.u.).
    #[serde(rename = "BR_R")]
    pub r: f64,

    /// Reactance (p.u.).
    #[serde(rename = "BR_X")]
    pub x: f64,

    /// Total line charging susceptance (p.u.).
    #[serde(rename = "BR_B")]
    pub b: f64,

    /// MVA rating A (long term rating).
    pub rate_a: f64,

    /// MVA rating B (short term rating) (MVA).
    pub rate_b: f64,

    /// MVA rating C (emergency rating) (MVA).
    pub rate_c: f64,

    /// Transformer off nominal tap ratio.
    pub tap: f64,

    /// Transformer phase shift angle (degrees).
    pub shift: f64,

    /// Initial branch status.
    #[serde(rename = "BR_STATUS")]
    #[validate(range(min = 0, max = 1))]
    pub status: usize,

    /// Minimum angle difference; angle(Vf) - angle(Vt) (degrees).
    pub angmin: Option<f64>,

    /// Maximum angle difference; angle(Vf) - angle(Vt) (degrees).
    pub angmax: Option<f64>,

    /// Real power injected at "from" bus end (MW).
    pub pf: Option<f64>,

    /// Reactive power injected at "from" bus end (MVAr).
    pub qf: Option<f64>,

    /// Real power injected at "to" bus end (MW).
    pub pt: Option<f64>,

    /// Reactive power injected at "to" bus end (MVAr).
    pub qt: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "from" bus (u/MVA).
    pub mu_sf: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "to" bus (u/MVA).
    pub mu_st: Option<f64>,

    /// Kuhn-Tucker multiplier lower angle difference limit (u/degree).
    pub mu_angmin: Option<f64>,

    /// Kuhn-Tucker multiplier upper angle difference limit (u/degree).
    pub mu_angmax: Option<f64>,
}

impl Branch {
    /// Branch is in-service.
    pub fn is_on(&self) -> bool {
        self.status != 0
    }

    /// Branch is out-of-service.
    pub fn is_off(&self) -> bool {
        self.status == 0
    }

    /// Branch is a transformer.
    pub fn is_transformer(&self) -> bool {
        self.tap != 0.0
    }

    /// Is power flow result.
    pub fn is_pf(&self) -> bool {
        self.pf.is_some() && self.qf.is_some() && self.pt.is_some() && self.qt.is_some()
    }

    /// Is Optimal Power Flow (OPF) result.
    pub fn is_opf(&self) -> bool {
        self.is_pf()
            && self.mu_sf.is_some()
            && self.mu_sf.is_some()
            && self.mu_angmin.is_some()
            && self.mu_angmax.is_some()
    }
}
