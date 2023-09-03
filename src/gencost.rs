use csv::StringRecord;
use derive_builder::Builder;
use validator::Validate;

/// Piecewise linear cost model.
pub const PW_LINEAR: usize = 1;
/// Polynomial cost model.
pub const POLYNOMIAL: usize = 2;

/// Generator cost function.
#[derive(Clone, Debug, Validate, Builder)]
#[builder(setter(into))]
#[validate(schema(function = "crate::validate::validate_gencost"))]
pub struct GenCost {
    /// Cost function model.
    #[builder(default = "POLYNOMIAL")]
    #[validate(range(min = 1, max = 2))]
    pub model: usize,

    /// Startup cost (US dollars).
    #[builder(default)]
    pub startup: f64,

    /// Shutdown cost (US dollars).
    #[builder(default)]
    pub shutdown: f64,

    /// Number of end/breakpoints in piecewise linear cost function
    /// or coefficients in polynomial cost function.
    #[builder(setter(into = false))]
    #[validate(range(min = 1))]
    pub ncost: usize,

    /// Piecewise linear cost function end/breakpoints.
    #[builder(setter(strip_option, each(name = "point")), default)]
    pub points: Option<Vec<(f64, f64)>>,

    /// Polynomial cost function coefficients.
    #[builder(setter(strip_option, each(name = "coeff")), default)]
    pub coeffs: Option<Vec<f64>>,
}

impl GenCost {
    /// Build new [GenCost].
    pub fn new(model: usize) -> GenCostBuilder {
        GenCostBuilder {
            model: Some(model),
            ..Default::default()
        }
    }

    /// Piecewise linear cost function.
    pub fn is_pwl(&self) -> bool {
        self.model == PW_LINEAR
    }

    /// Polynomial cost function.
    pub fn is_polynomial(&self) -> bool {
        self.model == POLYNOMIAL
    }

    pub(crate) fn from_string_record(record: StringRecord) -> Result<Self, String> {
        let mut iter = record.iter();

        let mut cost = Self {
            model: 0,
            startup: 0.0,
            shutdown: 0.0,
            ncost: 0,
            points: None,
            coeffs: None,
        };

        match iter.next() {
            Some(model_field) => match model_field.parse::<usize>() {
                Ok(model) => cost.model = model,
                Err(err) => {
                    return Err(format!("cost model parse error: {} ({})", err, model_field));
                }
            },
            None => {
                return Err("record must have a cost model field".to_string());
            }
        }
        match iter.next() {
            Some(startup_field) => match startup_field.parse::<f64>() {
                Ok(startup) => cost.startup = startup,
                Err(err) => {
                    return Err(format!(
                        "startup cost parse error: {} ({})",
                        err, startup_field
                    ));
                }
            },
            None => {
                return Err("record must have a startup cost field".to_string());
            }
        }
        match iter.next() {
            Some(shutdown_field) => match shutdown_field.parse::<f64>() {
                Ok(shutdown) => cost.shutdown = shutdown,
                Err(err) => {
                    return Err(format!(
                        "shutdown cost parse error: {} ({})",
                        err, shutdown_field
                    ));
                }
            },
            None => {
                return Err("record must have a shutdown cost field".to_string());
            }
        }
        match iter.next() {
            Some(ncost_field) => match ncost_field.parse::<usize>() {
                Ok(ncost) => cost.ncost = ncost,
                Err(err) => {
                    return Err(format!("ncost parse error: {} ({})", err, ncost_field));
                }
            },
            None => {
                return Err("record must have a ncost field".to_string());
            }
        }

        if cost.is_pwl() {
            let mut points = Vec::default();
            for n in 1..=cost.ncost {
                let mut point = (0.0, 0.0);
                match iter.next() {
                    Some(p_field) => match p_field.parse::<f64>() {
                        Ok(p) => point.0 = p,
                        Err(err) => {
                            return Err(format!(
                                "pwl point (p{}) parse error: {} ({})",
                                n, err, p_field
                            ));
                        }
                    },
                    None => {
                        return Err(format!("record must have a pwl p{} point field", n));
                    }
                }
                match iter.next() {
                    Some(f_field) => match f_field.parse::<f64>() {
                        Ok(f) => point.1 = f,
                        Err(err) => {
                            return Err(format!(
                                "pwl point (f{}) parse error: {} ({})",
                                n, err, f_field
                            ));
                        }
                    },
                    None => {
                        return Err(format!("record must have a pwl f{} point field", n));
                    }
                }
                points.push(point);
            }
            cost.points = Some(points);
        } else if cost.is_polynomial() {
            let mut coeffs = Vec::default();
            for n in (0..cost.ncost).rev() {
                match iter.next() {
                    Some(coeff_field) => match coeff_field.parse::<f64>() {
                        Ok(coeff) => coeffs.push(coeff),
                        Err(err) => {
                            return Err(format!(
                                "coefficient ({}) parse error: {} ({})",
                                n, err, coeff_field
                            ));
                        }
                    },
                    None => {
                        return Err(format!("record must have a coefficient n={} field", n));
                    }
                }
            }
            cost.coeffs = Some(coeffs);
        } else {
            return Err(format!("cost model must be 1 or 2 ({})", cost.model));
        }

        Ok(cost)
    }
}
