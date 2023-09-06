use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// PQ bus type.
pub const PQ: usize = 1;
/// PV bus type.
pub const PV: usize = 2;
/// Reference bus type.
pub const REF: usize = 3;
/// Isolated bus type.
pub const NONE: usize = 4;

#[derive(Serialize, Deserialize, Validate, Clone, Debug, Builder, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[builder(setter(into))]
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
    #[serde(rename = "BUS_AREA")]
    #[builder(setter(into = false), default = "1")]
    pub area: usize,

    /// Voltage magnitude (p.u.).
    #[builder(default = "1.0")]
    pub vm: f64,

    /// Voltage angle (degrees).
    #[builder(default)]
    pub va: f64,

    /// Base voltage (kV).
    pub base_kv: f64,

    /// Maximum voltage magnitude (p.u.).
    #[builder(default = "f64::INFINITY")]
    pub vmax: f64,

    /// Minimum voltage magnitude (p.u.).
    #[builder(default = "f64::NEG_INFINITY")]
    pub vmin: f64,

    /// Loss zone.
    #[builder(setter(into = false), default = "1")]
    pub zone: usize,

    /// Lagrange multiplier on real power mismatch (u/MW).
    #[builder(setter(custom), default)]
    pub lam_p: Option<f64>,

    /// Lagrange multiplier on reactive power mismatch (u/MVAr).
    #[builder(setter(custom), default)]
    pub lam_q: Option<f64>,

    /// Kuhn-Tucker multiplier on upper voltage limit (u/p.u.).
    #[builder(setter(custom), default)]
    pub mu_vmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower voltage limit (u/p.u.).
    #[builder(setter(custom), default)]
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

    /// Fixed active and reactive power.
    pub fn is_pq(&self) -> bool {
        self.bus_type == 1
    }

    /// Fixed voltage magnitude and active power.
    pub fn is_pv(&self) -> bool {
        self.bus_type == 2
    }

    /// Voltage angle reference. Slack active and reactive power.
    pub fn is_ref(&self) -> bool {
        self.bus_type == 3
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
