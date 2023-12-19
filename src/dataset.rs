use serde::Serialize;

use crate::soa::{BranchVec, BusVec, GenVec};
use crate::{Branch, Bus, Case, Gen};

#[derive(Serialize)]
pub struct Dataset {
    pub casename: String,
    pub base_mva: f64,

    #[serde(flatten)]
    pub bus: BusVec,

    #[serde(flatten)]
    pub gen: GenVec,

    #[serde(flatten)]
    pub branch: BranchVec,
    // pub bus_i: Vec<usize>,
    // pub bus_type: Vec<usize>,
    // pub pd: Vec<f64>,
    // pub qd: Vec<f64>,
    // pub gs: Vec<f64>,
    // pub bs: Vec<f64>,
    // pub vm: Vec<f64>,
    // pub va: Vec<f64>,
    // pub base_kv: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub vmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub vmin: Vec<f64>,

    // pub gen_bus: Vec<usize>,
    // pub pg: Vec<f64>,
    // pub qg: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub qmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub qmin: Vec<f64>,
    // pub vg: Vec<f64>,
    // pub gen_status: Vec<usize>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub pmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub pmin: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_pmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_pmin: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_qmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_qmin: Vec<f64>,

    // pub f_bus: Vec<usize>,
    // pub t_bus: Vec<usize>,
    // pub br_r: Vec<f64>,
    // pub br_x: Vec<f64>,
    // pub br_b: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub rate_a: Vec<f64>,
    // pub tap: Vec<f64>,
    // pub shift: Vec<f64>,
    // pub br_status: Vec<usize>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub angmin: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub angmax: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub pf: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub qf: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub pt: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub qt: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_sf: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_st: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_angmin: Vec<f64>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub mu_angmax: Vec<f64>,
}

impl Dataset {
    pub fn new(case: &Case, bus: &[Bus], gen: &[Gen], branch: &[Branch]) -> Self {
        let mut bus_vec = BusVec::new();
        bus.iter().for_each(|b| bus_vec.push(b.clone()));

        let mut gen_vec = GenVec::new();
        gen.iter().for_each(|b| gen_vec.push(b.clone()));

        let mut branch_vec = BranchVec::new();
        branch.iter().for_each(|b| branch_vec.push(b.clone()));

        Self {
            casename: case.name.clone(),
            base_mva: case.base_mva,

            bus: bus_vec,
            gen: gen_vec,
            branch: branch_vec,
            // bus_i: bus.iter().map(|b| b.bus_i).collect(),
            // bus_type: bus.iter().map(|b| b.bus_type).collect(),
            // pd: bus.iter().map(|b| b.pd).collect(),
            // qd: bus.iter().map(|b| b.qd).collect(),
            // gs: bus.iter().map(|b| b.gs).collect(),
            // bs: bus.iter().map(|b| b.bs).collect(),
            // vm: bus.iter().map(|b| b.vm).collect(),
            // va: bus.iter().map(|b| b.va).collect(),
            // base_kv: bus.iter().map(|b| b.base_kv).collect(),
            // vmax: bus.iter().map(|b| b.vmax).collect(),
            // vmin: bus.iter().map(|b| b.vmin).collect(),

            // gen_bus: gen.iter().map(|g| g.gen_bus).collect(),
            // pg: gen.iter().map(|g| g.pg).collect(),
            // qg: gen.iter().map(|g| g.qg).collect(),
            // qmax: gen.iter().map(|g| g.qmax).collect(),
            // qmin: gen.iter().map(|g| g.qmin).collect(),
            // vg: gen.iter().map(|g| g.vg).collect(),
            // gen_status: gen.iter().map(|g| g.gen_status).collect(),
            // pmax: gen.iter().map(|g| g.pmax).collect(),
            // pmin: gen.iter().map(|g| g.pmin).collect(),
            // mu_pmax: gen.iter().filter_map(|g| g.mu_pmax).collect(),
            // mu_pmin: gen.iter().filter_map(|g| g.mu_pmin).collect(),
            // mu_qmax: gen.iter().filter_map(|g| g.mu_qmax).collect(),
            // mu_qmin: gen.iter().filter_map(|g| g.mu_qmin).collect(),

            // f_bus: branch.iter().map(|br| br.f_bus).collect(),
            // t_bus: branch.iter().map(|br| br.t_bus).collect(),
            // br_r: branch.iter().map(|br| br.br_r).collect(),
            // br_x: branch.iter().map(|br| br.br_x).collect(),
            // br_b: branch.iter().map(|br| br.br_b).collect(),
            // rate_a: branch.iter().map(|br| br.rate_a).collect(),
            // tap: branch.iter().map(|br| br.tap).collect(),
            // shift: branch.iter().map(|br| br.shift).collect(),
            // br_status: branch.iter().map(|br| br.br_status).collect(),
            // angmin: branch.iter().filter_map(|br| br.angmin).collect(),
            // angmax: branch.iter().filter_map(|br| br.angmax).collect(),
            // pf: branch.iter().filter_map(|br| br.pf).collect(),
            // qf: branch.iter().filter_map(|br| br.qf).collect(),
            // pt: branch.iter().filter_map(|br| br.pt).collect(),
            // qt: branch.iter().filter_map(|br| br.qt).collect(),
            // mu_sf: branch.iter().filter_map(|br| br.mu_sf).collect(),
            // mu_st: branch.iter().filter_map(|br| br.mu_st).collect(),
            // mu_angmin: branch.iter().filter_map(|br| br.mu_angmin).collect(),
            // mu_angmax: branch.iter().filter_map(|br| br.mu_angmax).collect(),
        }
    }
}
