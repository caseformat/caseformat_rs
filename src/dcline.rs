use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Dispatchable DC transmission line.
#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[builder(setter(into))]
pub struct DCLine {
    /// "from" bus number.
    #[builder(setter(custom))]
    pub f_bus: usize,

    /// "to" bus number.
    #[builder(setter(custom))]
    pub t_bus: usize,

    /// Initial DC line status.
    #[serde(rename = "BR_STATUS")]
    #[builder(setter(into = false), default = "1")]
    pub status: usize,

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

    /// DC line is in-service.
    pub fn is_on(&self) -> bool {
        self.status != 0
    }

    /// DC line is out-of-service.
    pub fn is_off(&self) -> bool {
        self.status == 0
    }
}

impl DCLineBuilder {
    /// In-service DC line status.
    pub fn in_service(&mut self) -> &mut Self {
        self.status = Some(1);
        self
    }

    /// Out-of-service DC line status.
    pub fn out_of_service(&mut self) -> &mut Self {
        self.status = Some(0);
        self
    }
}
