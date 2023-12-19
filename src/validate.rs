use crate::{Branch, Bus, DCLine, Gen, GenCost};
use std::collections::HashSet;
use validator::ValidationError;

pub fn validate_bus_numbers(
    bus: &[Bus],
    gen: Option<&[Gen]>,
    branch: Option<&[Branch]>,
    dcline: Option<&[DCLine]>,
) -> Result<(), ValidationError> {
    let mut bus_numbers = HashSet::new();
    for b in bus {
        if bus_numbers.contains(&b.bus_i) {
            let mut err = ValidationError::new("bus numbers must be unique");
            err.add_param("bus_i".into(), &b.bus_i);
            return Err(err);
        }
        bus_numbers.insert(&b.bus_i);
    }

    if let Some(gen) = gen {
        for g in gen {
            if !bus_numbers.contains(&g.gen_bus) {
                let mut err = ValidationError::new("gen bus must exist");
                err.add_param("bus".into(), &g.gen_bus);
                return Err(err);
            }
        }
    }

    if let Some(branch) = branch {
        for br in branch {
            if !bus_numbers.contains(&br.f_bus) {
                let mut err = ValidationError::new("branch f_bus must exist");
                err.add_param("f_bus".into(), &br.f_bus);
                return Err(err);
            }
            if !bus_numbers.contains(&br.t_bus) {
                let mut err = ValidationError::new("branch t_bus must exist");
                err.add_param("t_bus".into(), &br.t_bus);
                return Err(err);
            }
        }
    }

    if let Some(dcline) = dcline {
        for ln in dcline {
            if !bus_numbers.contains(&ln.f_bus) {
                let mut err = ValidationError::new("dcline f_bus must exist");
                err.add_param("f_bus".into(), &ln.f_bus);
                return Err(err);
            }
            if !bus_numbers.contains(&ln.t_bus) {
                let mut err = ValidationError::new("dcline t_bus must exist");
                err.add_param("t_bus".into(), &ln.t_bus);
                return Err(err);
            }
        }
    }

    Ok(())
}

pub(crate) fn validate_gen(g: &Gen) -> Result<(), ValidationError> {
    if g.qmax < g.qmin {
        let mut err = ValidationError::new("qmax must be >= qmin");
        err.add_param("qmax".into(), &g.qmax);
        err.add_param("qmin".into(), &g.qmin);
        return Err(err);
    }
    if g.pmax < g.pmin {
        let mut err = ValidationError::new("pmax must be >= pmin");
        err.add_param("pmax".into(), &g.pmax);
        err.add_param("pmin".into(), &g.pmin);
        return Err(err);
    }

    let v2: Vec<Option<f64>> = vec![
        g.pc1, g.pc2, g.qc1min, g.qc1max, g.qc2min, g.qc2max, g.ramp_agc, g.ramp_10, g.ramp_30,
        g.ramp_q, g.apf,
    ];
    if v2.iter().any(|a| a.is_some()) {
        if !v2.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("version 2 fields must all be set if one is set");
            err.add_param("pc1".into(), &g.pc1);
            err.add_param("pc2".into(), &g.pc2);
            err.add_param("qc1min".into(), &g.qc1min);
            err.add_param("qc1max".into(), &g.qc1max);
            err.add_param("qc2min".into(), &g.qc2min);
            err.add_param("qc2max".into(), &g.qc2max);
            err.add_param("ramp_agc".into(), &g.ramp_agc);
            err.add_param("ramp_10".into(), &g.ramp_10);
            err.add_param("ramp_30".into(), &g.ramp_30);
            err.add_param("ramp_q".into(), &g.ramp_q);
            err.add_param("apf".into(), &g.apf);
            return Err(err);
        }
    }

    let opf = vec![g.mu_pmax, g.mu_pmin, g.mu_qmax, g.mu_qmin];
    if opf.iter().any(|a| a.is_some()) {
        if !opf.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("opf result fields must all be set if one is set");
            err.add_param("mu_pmax".into(), &g.mu_pmax);
            err.add_param("mu_pmin".into(), &g.mu_pmin);
            err.add_param("mu_qmax".into(), &g.mu_qmax);
            err.add_param("mu_qmin".into(), &g.mu_qmin);
            return Err(err);
        }
        if !v2.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new(
                "version 2 fields must all be set if opf result fields are set",
            );
            err.add_param("pc1".into(), &g.pc1);
            err.add_param("pc2".into(), &g.pc2);
            err.add_param("qc1min".into(), &g.qc1min);
            err.add_param("qc1max".into(), &g.qc1max);
            err.add_param("qc2min".into(), &g.qc2min);
            err.add_param("qc2max".into(), &g.qc2max);
            err.add_param("ramp_agc".into(), &g.ramp_agc);
            err.add_param("ramp_10".into(), &g.ramp_10);
            err.add_param("ramp_30".into(), &g.ramp_30);
            err.add_param("ramp_q".into(), &g.ramp_q);
            err.add_param("apf".into(), &g.apf);

            err.add_param("mu_pmax".into(), &g.mu_pmax);
            err.add_param("mu_pmin".into(), &g.mu_pmin);
            err.add_param("mu_qmax".into(), &g.mu_qmax);
            err.add_param("mu_qmin".into(), &g.mu_qmin);
            return Err(err);
        }
    }

    Ok(())
}

pub(crate) fn validate_branch(br: &Branch) -> Result<(), ValidationError> {
    if br.f_bus == br.t_bus {
        let mut err = ValidationError::new("f_bus and t_bus numbers must be different");
        err.add_param("f_bus".into(), &br.f_bus);
        err.add_param("t_bus".into(), &br.t_bus);
        return Err(err);
    }

    let anglim = vec![br.angmin, br.angmax];
    if anglim.iter().any(|a| a.is_some()) {
        if !anglim.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("both angle limits must be set if one is set");
            err.add_param("angmin".into(), &br.angmin);
            err.add_param("angmax".into(), &br.angmax);
            return Err(err);
        }
    }

    let flows = vec![br.pf, br.qf, br.pt, br.qt];
    if flows.iter().any(|a| a.is_some()) {
        if !anglim.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("angle limits must be set if branch flows are set");
            err.add_param("angmin".into(), &br.angmin);
            err.add_param("angmax".into(), &br.angmax);

            err.add_param("pf".into(), &br.pf);
            err.add_param("qf".into(), &br.qf);
            err.add_param("pt".into(), &br.pt);
            err.add_param("qt".into(), &br.qt);
            return Err(err);
        }

        if !flows.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("all branch flows must be set if one is set");
            err.add_param("pf".into(), &br.pf);
            err.add_param("qf".into(), &br.qf);
            err.add_param("pt".into(), &br.pt);
            err.add_param("qt".into(), &br.qt);
            return Err(err);
        }
    }

    let opf = vec![br.mu_sf, br.mu_st, br.mu_angmin, br.mu_angmax];
    if opf.iter().any(|a| a.is_some()) {
        if !anglim.iter().all(|a| a.is_some()) {
            let mut err = ValidationError::new("angle limits must be set if opf results are set");
            err.add_param("angmin".into(), &br.angmin);
            err.add_param("angmax".into(), &br.angmax);

            err.add_param("mu_sf".into(), &br.mu_sf);
            err.add_param("mu_st".into(), &br.mu_st);
            err.add_param("mu_angmin".into(), &br.mu_angmin);
            err.add_param("mu_angmax".into(), &br.mu_angmax);
            return Err(err);
        }

        if !flows.iter().all(|a| a.is_some()) {
            let mut err =
                ValidationError::new("all branch flows must be set if opf results are set");
            err.add_param("pf".into(), &br.pf);
            err.add_param("qf".into(), &br.qf);
            err.add_param("pt".into(), &br.pt);
            err.add_param("qt".into(), &br.qt);

            err.add_param("mu_sf".into(), &br.mu_sf);
            err.add_param("mu_st".into(), &br.mu_st);
            err.add_param("mu_angmin".into(), &br.mu_angmin);
            err.add_param("mu_angmax".into(), &br.mu_angmax);
            return Err(err);
        }

        if !opf.iter().all(|a| a.is_some()) {
            let mut err =
                ValidationError::new("all opf results must be set if one opf result is set");
            err.add_param("mu_sf".into(), &br.mu_sf);
            err.add_param("mu_st".into(), &br.mu_st);
            err.add_param("mu_angmin".into(), &br.mu_angmin);
            err.add_param("mu_angmax".into(), &br.mu_angmax);
            return Err(err);
        }
    }

    Ok(())
}

pub(crate) fn validate_gencost(cost: &GenCost) -> Result<(), ValidationError> {
    if cost.is_pwl() {
        if let Some(points) = cost.points.as_ref() {
            if cost.ncost != points.len() {
                let mut err =
                    ValidationError::new("ncost must equal the number of pwl end/breakpoints");
                err.add_param("ncost".into(), &cost.ncost);
                err.add_param("len".into(), &points.len());
                return Err(err);
            }
        } else {
            let mut err = ValidationError::new("end/breakpoints must be set if model is pwl");
            err.add_param("model".into(), &cost.model);
            return Err(err);
        }
    }

    if cost.is_polynomial() {
        if let Some(coeffs) = cost.coeffs.as_ref() {
            if cost.ncost != coeffs.len() {
                let mut err = ValidationError::new("ncost must equal the number of coefficients");
                err.add_param("ncost".into(), &cost.ncost);
                err.add_param("len".into(), &coeffs.len());
                return Err(err);
            }
        } else {
            let mut err = ValidationError::new("coefficients must be set if model is polynomial");
            err.add_param("model".into(), &cost.model);
            return Err(err);
        }
    }
    Ok(())
}
