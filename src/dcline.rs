use serde::{Deserialize, Serialize};

/// Dispatchable DC transmission line.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DCLine {
    /// "from" bus number.
    pub f_bus: usize,

    /// "to" bus number.
    pub t_bus: usize,

    /// Initial DC line status.
    #[serde(rename = "BR_STATUS")]
    pub status: usize,

    /// Flow at "from" bus ("from" -> "to") (MW).
    pub pf: f64,

    /// Flow at "to" bus ("from" -> "to") (MW).
    pub pt: f64,

    /// Injection at "from" bus ("from" -> "to") (MVAr).
    pub qf: f64,

    /// Injection at "to" bus ("from" -> "to") (MVAr).
    pub qt: f64,

    /// Voltage setpoint at "from" bus (p.u.).
    pub vf: f64,

    /// Voltage setpoint at "to" bus (p.u.).
    pub vt: f64,

    /// Lower limit on MW flow at "from" end (MW).
    pub pmin: f64,

    /// Upper limit on MW flow at "from" end (MW).
    pub pmax: f64,

    /// Lower limit on MVAr injection at "from" bus (MVAr).
    pub qminf: f64,

    /// Upper limit on MVAr injection at "from" bus (MVAr).
    pub qmaxf: f64,

    /// Lower limit on MVAr injection at "to" bus (MVAr).
    pub qmint: f64,

    /// Upper limit on MVAr injection at "to" bus (MVAr).
    pub qmaxt: f64,

    /// Constant term of linear loss function (MW).
    pub loss0: f64,

    /// Linear term of linear loss function (MW).
    pub loss1: f64,

    /// Kuhn-Tucker multiplier on lower flow lim at "from" bus (u/MW).
    pub mu_pmin: Option<f64>,

    /// Kuhn-Tucker multiplier on upper flow lim at "from" bus (u/MW).
    pub mu_pmax: Option<f64>,

    /// Kuhn-Tucker multiplier on lower VAr lim at "from" bus (u/MVAr).
    pub mu_qminf: Option<f64>,

    /// Kuhn-Tucker multiplier on upper VAr lim at "from" bus (u/MVAr).
    pub mu_qmaxf: Option<f64>,

    /// Kuhn-Tucker multiplier on lower VAr lim at "to" bus (u/MVAr).
    pub mu_qmint: Option<f64>,

    /// Kuhn-Tucker multiplier on upper VAr lim at "to" bus (u/MVAr).
    pub mu_qmaxt: Option<f64>,
}

impl DCLine {
    /// DC line is in-service.
    pub fn is_on(&self) -> bool {
        self.status != 0
    }

    /// DC line is out-of-service.
    pub fn is_off(&self) -> bool {
        self.status == 0
    }
}
