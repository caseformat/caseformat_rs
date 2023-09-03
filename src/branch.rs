use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Transmission line/cable or two winding transformer.
#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[builder(setter(into))]
#[validate(schema(function = "crate::validate::validate_branch"))]
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
    #[serde(rename = "BR_R")]
    #[builder(default)]
    pub r: f64,

    /// Reactance (p.u.).
    #[serde(rename = "BR_X")]
    #[builder(default)]
    pub x: f64,

    /// Total line charging susceptance (p.u.).
    #[serde(rename = "BR_B")]
    #[builder(default)]
    pub b: f64,

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
    #[serde(rename = "BR_STATUS")]
    #[builder(setter(into = false), default = "1")]
    #[validate(range(min = 0, max = 1))]
    pub status: usize,

    /// Minimum angle difference; angle(Vf) - angle(Vt) (degrees).
    #[builder(setter(strip_option), default)]
    pub angmin: Option<f64>,

    /// Maximum angle difference; angle(Vf) - angle(Vt) (degrees).
    #[builder(setter(strip_option), default)]
    pub angmax: Option<f64>,

    /// Real power injected at "from" bus end (MW).
    #[builder(setter(custom), default)]
    pub pf: Option<f64>,

    /// Reactive power injected at "from" bus end (MVAr).
    #[builder(setter(custom), default)]
    pub qf: Option<f64>,

    /// Real power injected at "to" bus end (MW).
    #[builder(setter(custom), default)]
    pub pt: Option<f64>,

    /// Reactive power injected at "to" bus end (MVAr).
    #[builder(setter(custom), default)]
    pub qt: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "from" bus (u/MVA).
    #[builder(setter(custom), default)]
    pub mu_sf: Option<f64>,

    /// Kuhn-Tucker multiplier on MVA limit at "to" bus (u/MVA).
    #[builder(setter(custom), default)]
    pub mu_st: Option<f64>,

    /// Kuhn-Tucker multiplier lower angle difference limit (u/degree).
    #[builder(setter(custom), default)]
    pub mu_angmin: Option<f64>,

    /// Kuhn-Tucker multiplier upper angle difference limit (u/degree).
    #[builder(setter(custom), default)]
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

    /// Is OPF result.
    pub fn is_opf(&self) -> bool {
        self.is_pf()
            && self.mu_sf.is_some()
            && self.mu_sf.is_some()
            && self.mu_angmin.is_some()
            && self.mu_angmax.is_some()
    }
}

impl BranchBuilder {
    /// In-service branch status.
    pub fn in_service(&mut self) -> &mut Self {
        self.status = Some(1);
        self
    }

    /// Out-of-service branch status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.status = Some(0);
        self
    }
}
