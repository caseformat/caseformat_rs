use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Bus {
    /// Bus number.
    pub bus_i: usize,

    /// Bus type.
    #[validate(range(min = 1, max = 4))]
    pub bus_type: usize,

    /// Real power demand (MW).
    pub pd: f64,

    /// Reactive power demand (MVAr).
    pub qd: f64,

    /// Shunt conductance (MW at V = 1.0 p.u.).
    pub gs: f64,

    /// Shunt susceptance (MVAr at V = 1.0 p.u.).
    pub bs: f64,

    /// Area number, 1-100.
    #[serde(rename = "BUS_AREA")]
    pub area: usize,

    /// Voltage magnitude (p.u.).
    pub vm: f64,

    /// Voltage angle (degrees).
    pub va: f64,

    /// Base voltage (kV).
    pub base_kv: f64,

    /// Maximum voltage magnitude (p.u.).
    pub vmax: f64,

    /// Minimum voltage magnitude (p.u.).
    pub vmin: f64,

    /// Loss zone.
    pub zone: usize,

    /// Lagrange multiplier on real power mismatch (u/MW).
    pub lam_p: Option<f64>,

    /// Lagrange multiplier on reactive power mismatch (u/MVAr).
    pub lam_q: Option<f64>,

    /// Kuhn-Tucker multiplier on upper voltage limit (u/p.u.).
    pub mu_vmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower voltage limit (u/p.u.).
    pub mu_vmin: Option<f64>,
}

impl Bus {
    /// Fixed active and reactive power.
    pub fn is_pq(&self) -> bool {
        self.bus_type == 1
    }

    /// Fixed voltage magnitude and active power.
    pub fn is_pv(&self) -> bool {
        self.bus_type == 2
    }

    /// Reference voltage angle. Slack active and reactive power.
    pub fn is_ref(&self) -> bool {
        self.bus_type == 3
    }

    /// Is Optimal Power Flow result.
    pub fn is_opf(&self) -> bool {
        self.lam_p.is_some()
            && self.lam_q.is_some()
            && self.mu_vmax.is_some()
            && self.mu_vmin.is_some()
    }
}
