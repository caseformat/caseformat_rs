use crate::{IN_SERVICE, NONE, OUT_OF_SERVICE, PQ};
use anyhow::{format_err, Result};
use power_flow_data::Bus::{Bus30, Bus33};
use power_flow_data::Network;
use std::collections::HashMap;

const DEFAULT_VMIN: f64 = 0.9;
const DEFAULT_VMAX: f64 = 1.1;

pub fn raw_to_case(network: &Network) -> Result<(crate::Case, Vec<crate::Bus>, Vec<crate::Gen>)> {
    let base_mva = network.caseid.sbase;

    let case = {
        let mut builder = crate::Case::new("");
        builder.base_mva(base_mva);
        if let Some(basfrq) = network.caseid.basfrq {
            builder.f(basfrq);
        };
        builder.build()?
    };

    // Bus //

    let mut bus_vec = Vec::with_capacity(network.buses.len());
    for raw_bus in &network.buses {
        let mut builder = crate::Bus::new(*raw_bus.i() as usize);
        builder
            .bus_type(*raw_bus.ide() as usize)
            .base_kv(*raw_bus.basekv())
            .bus_area(*raw_bus.area() as usize)
            .zone(*raw_bus.zone() as usize)
            .vm(*raw_bus.vm())
            .va(*raw_bus.va());
        match raw_bus {
            Bus30(bus30) => {
                builder
                    .gs(bus30.gl)
                    .bs(bus30.bl)
                    .vmax(DEFAULT_VMAX)
                    .vmin(DEFAULT_VMIN);
            }
            Bus33(bus33) => {
                builder.vmax(bus33.nvhi);
                builder.vmin(bus33.nvlo);
            }
        }
        bus_vec.push(builder.build()?);
    }

    let bus_index: HashMap<usize, usize> = bus_vec
        .iter()
        .enumerate()
        .map(|(i, bus)| (bus.bus_i, i))
        .collect();

    for raw_load in network.loads.iter().filter(|ld| ld.status) {
        let i = raw_load.i as usize;
        let j = bus_index.get(&i).unwrap();
        let bus = &mut bus_vec[*j];

        let vm = bus.vm;
        let vm2 = bus.vm.powi(2);

        bus.pd += raw_load.pl + raw_load.ip * vm + raw_load.yp * vm2;
        bus.qd += raw_load.ql + raw_load.iq * vm - raw_load.yq * vm2;
    }

    if let Some(fixed_shunts) = &network.fixed_shunts {
        for raw_shunt in fixed_shunts.iter().filter(|fs| fs.status) {
            let i = raw_shunt.i as usize;
            let j = bus_index.get(&i).unwrap();
            let bus = &mut bus_vec[*j];

            bus.gs += raw_shunt.gl;
            bus.bs += raw_shunt.bl;
        }
    }

    for raw_shunt in &network.switched_shunts {
        let i = *raw_shunt.i() as usize;
        let j = bus_index.get(&i).unwrap();
        let bus = &mut bus_vec[*j];
        bus.bs += raw_shunt.binit();
    }

    // Generator //

    let mut gen_vec = Vec::with_capacity(network.generators.len());
    for raw_gen in &network.generators {
        let gen = crate::Gen::new(raw_gen.i as usize)
            .pg(raw_gen.pg)
            .qg(raw_gen.qg)
            .qmax(raw_gen.qt)
            .qmin(raw_gen.qb)
            .vg(raw_gen.vs)
            .mbase(raw_gen.mbase)
            .gen_status(if raw_gen.stat {
                IN_SERVICE
            } else {
                OUT_OF_SERVICE
            })
            .pmax(raw_gen.pt)
            .pmin(raw_gen.pb)
            .build()?;
        gen_vec.push(gen);
    }

    // Branch //

    let mut branch_vec = Vec::with_capacity(network.branches.len() + network.transformers.len());

    for raw_branch in &network.branches {
        let mut builder =
            crate::Branch::new(*raw_branch.i() as usize, raw_branch.j().abs() as usize);
        builder
            .br_r(*raw_branch.r())
            .br_x(*raw_branch.x())
            .br_b(*raw_branch.b())
            .rate_a(*raw_branch.rate_a())
            .rate_b(*raw_branch.rate_b())
            .rate_c(*raw_branch.rate_c())
            .br_status(if *raw_branch.st() {
                IN_SERVICE
            } else {
                OUT_OF_SERVICE
            });
        branch_vec.push(builder.build()?);
    }

    for raw_branch in network.branches.iter().filter(|br| *br.st()) {
        let i = *raw_branch.i() as usize;
        let ii = bus_index.get(&i).unwrap();
        let fbus = &mut bus_vec[*ii];
        fbus.gs += raw_branch.gi() * base_mva;
        fbus.bs += raw_branch.bi() * base_mva;

        let j = *raw_branch.j() as usize;
        let jj = bus_index.get(&j).unwrap();
        let tbus = &mut bus_vec[*jj];
        tbus.gs += raw_branch.gj() * base_mva;
        tbus.bs += raw_branch.bj() * base_mva;
    }

    // Transformer //

    for raw_tr2 in network.transformers.iter().filter(|tr| tr.k == 0) {
        let i = raw_tr2.i as usize;
        let ii = bus_index.get(&i).unwrap();
        let fbus = &bus_vec[*ii];

        let j = raw_tr2.j as usize;
        let jj = bus_index.get(&j).unwrap();
        let tbus = &bus_vec[*jj];

        let tap = match raw_tr2.cw {
            1 => {
                // off-nominal turns ratio in pu of winding bus base voltage
                (raw_tr2.windv1 / raw_tr2.windv2) * (tbus.base_kv / fbus.base_kv)
            }
            2 => {
                // winding voltage in kV
                (raw_tr2.windv1 / raw_tr2.windv2) * (raw_tr2.nomv1 / raw_tr2.nomv2)
            }
            _ => return Err(format_err!("cw ({}) must be 1 or 2", raw_tr2.cw)),
        };

        let zb_bus1 = fbus.base_kv.powi(2) / base_mva;
        let zb_wdg1 = raw_tr2.nomv1.powi(2) / raw_tr2.sbase1_2;
        let (r, x) = match raw_tr2.cz {
            1 => {
                // pu on system base
                (raw_tr2.r1_2, raw_tr2.x1_2)
            }
            2 => {
                // pu on winding one to two base MVA (`sbase1_2`) and winding one bus base voltage

                let r = raw_tr2.r1_2 * zb_wdg1 / zb_bus1;
                let x = raw_tr2.x1_2 * zb_wdg1 / zb_bus1;

                // let r = base_mva * raw_tr2.r1_2 / raw_tr2.sbase1_2;
                // let x = base_mva * raw_tr2.x1_2 / raw_tr2.sbase1_2;

                (r, x)
            }
            3 => {
                // load loss in watts
                let r = 1e-6 * raw_tr2.r1_2 / raw_tr2.sbase1_2;

                // impedance magnitude in pu on winding one to two base MVA (`sbase1_2`) and winding one bus base voltage
                let x = f64::sqrt(raw_tr2.x1_2.powi(2) - raw_tr2.r1_2.powi(2));

                let r = r * zb_bus1 / zb_wdg1;
                let x = x * zb_bus1 / zb_wdg1;

                // let r = base_mva * raw_tr2.r1_2 / raw_tr2.sbase1_2;
                // let x = base_mva * raw_tr2.x1_2 / raw_tr2.sbase1_2;

                (r, x)
            }
            _ => return Err(format_err!("cw ({}) must be 1, 2 or 3", raw_tr2.cw)),
        };

        let branch = crate::Branch::new(raw_tr2.i as usize, raw_tr2.j as usize)
            .br_r(r)
            .br_x(x)
            .rate_a(raw_tr2.rata1)
            .rate_b(raw_tr2.ratb1)
            .rate_c(raw_tr2.ratc1)
            .br_status(if raw_tr2.stat != 0 {
                IN_SERVICE
            } else {
                OUT_OF_SERVICE
            })
            .tap(tap)
            .shift(raw_tr2.ang1)
            .build()?;
        branch_vec.push(branch);
    }

    let max_bus_i = bus_vec
        .iter()
        .map(|bus| bus.bus_i)
        .max()
        .unwrap_or_default();
    let bus_i0 = 10.0_f64.powf((max_bus_i as f64 + 1.0).log10().ceil()) as usize;

    for (i, raw_tr3) in network
        .transformers
        .iter()
        .filter(|tr| tr.k != 0)
        .enumerate()
    {
        // let bus1_i = bus_index.get(&(raw_tr3.i as usize)).unwrap();
        // let bus1 = &bus_vec[*bus1_i];
        let bus1 = {
            let i = raw_tr3.i as usize;
            let ii = bus_index.get(&i).unwrap();
            &bus_vec[*ii]
        };
        let bus2 = {
            let j = raw_tr3.j as usize;
            let jj = bus_index.get(&j).unwrap();
            &bus_vec[*jj]
        };
        let bus3 = {
            let k = raw_tr3.k as usize;
            let kk = bus_index.get(&k).unwrap();
            &bus_vec[*kk]
        };

        let star = crate::Bus::new(bus_i0 + i)
            .bus_type(if raw_tr3.stat != 0 { PQ } else { NONE })
            .va(raw_tr3.anstar.unwrap_or_default())
            .vm(raw_tr3.vmstar.unwrap_or(1.0))
            .bus_area(bus1.bus_area)
            .zone(bus1.zone)
            .base_kv(bus1.base_kv)
            .vmax(bus1.vmax)
            .vmin(bus1.vmin)
            .build()?;

        let tap1 = match raw_tr3.cw {
            1 => raw_tr3.windv1 / bus1.base_kv,
            2 => raw_tr3.windv1 * raw_tr3.nomv1,
            _ => return Err(format_err!("cw ({}) must be 1 or 2", raw_tr3.cw)),
        };

        let (r12, x12) = {
            let zbs1 = bus1.base_kv.powi(2) / base_mva;
            let zb1 = raw_tr3.nomv1.powi(2) / raw_tr3.sbase1_2;

            match raw_tr3.cz {
                1 => (raw_tr3.r1_2, raw_tr3.x1_2),
                2 => {
                    let r = raw_tr3.r1_2 * zb1 / zbs1;
                    let x = raw_tr3.x1_2 * zb1 / zbs1;
                    // let r = base_mva * raw_tr3.r1_2 / raw_tr3.sbase1_2;
                    // let x = base_mva * raw_tr3.x1_2 / raw_tr3.sbase1_2;
                    (r, x)
                }
                3 => {
                    let r = 1e-6 * raw_tr3.r1_2 / raw_tr3.sbase1_2;
                    let x = f64::sqrt(raw_tr3.x1_2.powi(2) - raw_tr3.r1_2.powi(2));
                    let r = r * (zb1 / zbs1);
                    let x = x * (zb1 / zbs1);
                    // let r = base_mva * raw_tr3.r1_2 / raw_tr3.sbase1_2;
                    // let x = base_mva * raw_tr3.x1_2 / raw_tr3.sbase1_2;
                    (r, x)
                }
                _ => return Err(format_err!("cw ({}) must be 1, 2 or 3", raw_tr3.cw)),
            }
        };

        let tap2 = match raw_tr3.cw {
            1 => raw_tr3.windv2 / bus1.base_kv,
            2 => raw_tr3.windv2 * raw_tr3.nomv2,
            _ => return Err(format_err!("cw ({}) must be 1 or 2", raw_tr3.cw)),
        };

        let (r23, x23) = {
            let r2_3 = raw_tr3.r2_3.unwrap();
            let x2_3 = raw_tr3.x2_3.unwrap();
            let sbase2_3 = raw_tr3.sbase2_3.unwrap();

            let zbs2 = bus2.base_kv.powi(2) / base_mva;
            let zb2 = raw_tr3.nomv2.powi(2) / sbase2_3;

            match raw_tr3.cz {
                1 => (r2_3, x2_3),
                2 => {
                    let r = r2_3 * zb2 / zbs2;
                    let x = x2_3 * zb2 / zbs2;
                    // let r = base_mva * r2_3 / sbase2_3;
                    // let x = base_mva * x2_3 / sbase2_3;
                    (r, x)
                }
                3 => {
                    let r = 1e-6 * r2_3 / sbase2_3;
                    let x = f64::sqrt(x2_3.powi(2) - r2_3.powi(2));
                    let r = r * (zb2 / zbs2);
                    let x = x * (zb2 / zbs2);
                    // let r = base_mva * r2_3 / sbase2_3;
                    // let x = base_mva * x2_3 / sbase2_3;
                    (r, x)
                }
                _ => return Err(format_err!("cw ({}) must be 1, 2 or 3", raw_tr3.cw)),
            }
        };

        let windv3 = raw_tr3.windv3.unwrap();
        let nomv3 = raw_tr3.nomv3.unwrap();
        let tap3 = match raw_tr3.cw {
            1 => windv3 / bus1.base_kv,
            2 => windv3 * nomv3,
            _ => return Err(format_err!("cw ({}) must be 1 or 2", raw_tr3.cw)),
        };

        let (r31, x31) = {
            let sbase3_1 = raw_tr3.sbase3_1.unwrap();
            let r3_1 = raw_tr3.r3_1.unwrap();
            let x3_1 = raw_tr3.x3_1.unwrap();

            let zbs3 = bus3.base_kv.powi(2) / base_mva;
            let zb3 = nomv3.powi(2) / sbase3_1;

            match raw_tr3.cz {
                1 => (r3_1, x3_1),
                2 => {
                    let r = r3_1 * zb3 / zbs3;
                    let x = x3_1 * zb3 / zbs3;
                    // let r = base_mva * r3_1 / sbase3_1;
                    // let x = base_mva * x3_1 / sbase3_1;
                    (r, x)
                }
                3 => {
                    let r = 1e-6 * r3_1 / sbase3_1;
                    let x = f64::sqrt(x3_1.powi(2) - r3_1.powi(2));
                    let r = r * (zb3 / zbs3);
                    let x = x * (zb3 / zbs3);
                    // let r = base_mva * r3_1 / sbase3_1;
                    // let x = base_mva * x3_1 / sbase3_1;
                    (r, x)
                }
                _ => return Err(format_err!("cw ({}) must be 1, 2 or 3", raw_tr3.cw)),
            }
        };

        let r1 = (r12 + r31 - r23) / 2.0;
        let r2 = (r12 + r23 - r31) / 2.0;
        let r3 = (r31 + r23 - r12) / 2.0;
        let x1 = (x12 + x31 - x23) / 2.0;
        let x2 = (x12 + x23 - x31) / 2.0;
        let x3 = (x31 + x23 - x12) / 2.0;

        let branch12 = crate::Branch::new(raw_tr3.i as usize, star.bus_i)
            .br_r(r1)
            .br_x(x1)
            .rate_a(raw_tr3.rata1)
            .rate_b(raw_tr3.ratb1)
            .rate_c(raw_tr3.ratc1)
            .br_status(if raw_tr3.stat == 0 || raw_tr3.stat == 4 {
                OUT_OF_SERVICE
            } else {
                IN_SERVICE
            })
            .tap(tap1)
            .shift(raw_tr3.ang1)
            .build()?;

        let branch23 = crate::Branch::new(raw_tr3.j as usize, star.bus_i)
            .br_r(r2)
            .br_x(x2)
            .rate_a(raw_tr3.rata2.unwrap())
            .rate_b(raw_tr3.ratb2.unwrap())
            .rate_c(raw_tr3.ratc2.unwrap())
            .br_status(if raw_tr3.stat == 0 || raw_tr3.stat == 2 {
                OUT_OF_SERVICE
            } else {
                IN_SERVICE
            })
            .tap(tap2)
            .shift(raw_tr3.ang2.unwrap())
            .build()?;

        let branch31 = crate::Branch::new(raw_tr3.k as usize, star.bus_i)
            .br_r(r3)
            .br_x(x3)
            .rate_a(raw_tr3.rata3.unwrap())
            .rate_b(raw_tr3.ratb3.unwrap())
            .rate_c(raw_tr3.ratc3.unwrap())
            .br_status(if raw_tr3.stat == 0 || raw_tr3.stat == 3 {
                OUT_OF_SERVICE
            } else {
                IN_SERVICE
            })
            .tap(tap3)
            .shift(raw_tr3.ang3.unwrap())
            .build()?;

        bus_vec.push(star);
        branch_vec.extend([branch12, branch23, branch31]);
    }

    Ok((case, bus_vec, gen_vec))
}
