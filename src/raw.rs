use anyhow::{format_err, Result};
use arrayvec::ArrayString;
use std::collections::HashMap;

use power_flow_data::{AreaNum, BusNum, CaseID, Stat, ZoneNum};

use crate::{IN_SERVICE, NONE, OUT_OF_SERVICE, PQ};

pub fn raw_to_case(
    network: &power_flow_data::Network,
) -> Result<(
    crate::Case,
    Vec<crate::Bus>,
    Vec<crate::Gen>,
    Vec<crate::Branch>,
    Vec<crate::DCLine>,
)> {
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
        let mut builder = crate::Bus::new(raw_bus.i as usize);
        builder
            .bus_type(raw_bus.ide as usize)
            .base_kv(raw_bus.basekv)
            .bus_area(raw_bus.area as usize)
            .zone(raw_bus.zone as usize)
            .vm(raw_bus.vm)
            .va(raw_bus.va)
            .vmax(raw_bus.nvhi)
            .vmin(raw_bus.nvlo);
        bus_vec.push(builder.build()?);
    }

    let bus_index: HashMap<usize, usize> = bus_vec
        .iter()
        .enumerate()
        .map(|(i, bus)| (bus.bus_i, i))
        .collect();

    for raw_load in network.loads.iter().filter(|ld| ld.status != 0) {
        let i = raw_load.i as usize;
        let j = bus_index.get(&i).unwrap();
        let bus = &mut bus_vec[*j];

        let vm = bus.vm;
        let vm2 = bus.vm.powi(2);

        bus.pd += raw_load.pl + raw_load.ip * vm + raw_load.yp * vm2;
        bus.qd += raw_load.ql + raw_load.iq * vm - raw_load.yq * vm2;
    }

    for raw_shunt in network.fixed_shunts.iter().filter(|fs| fs.status != 0) {
        let i = raw_shunt.i as usize;
        let j = bus_index.get(&i).unwrap();
        let bus = &mut bus_vec[*j];

        bus.gs += raw_shunt.gl;
        bus.bs += raw_shunt.bl;
    }

    for raw_shunt in &network.switched_shunts {
        let i = raw_shunt.i as usize;
        let j = bus_index.get(&i).unwrap();
        let bus = &mut bus_vec[*j];
        bus.bs += raw_shunt.binit;
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
            .gen_status(if raw_gen.stat != 0 {
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
        let mut builder = crate::Branch::new(raw_branch.i as usize, raw_branch.j.abs() as usize);
        builder
            .br_r(raw_branch.r)
            .br_x(raw_branch.x)
            .br_b(raw_branch.b)
            .rate_a(raw_branch.rate_a)
            .rate_b(raw_branch.rate_b)
            .rate_c(raw_branch.rate_c)
            .br_status(if raw_branch.st != 0 {
                IN_SERVICE
            } else {
                OUT_OF_SERVICE
            });
        branch_vec.push(builder.build()?);
    }

    for raw_branch in network.branches.iter().filter(|br| br.st != 0) {
        let i = raw_branch.i as usize;
        let ii = bus_index.get(&i).unwrap();
        let fbus = &mut bus_vec[*ii];
        fbus.gs += raw_branch.gi * base_mva;
        fbus.bs += raw_branch.bi * base_mva;

        let j = raw_branch.j as usize;
        let jj = bus_index.get(&j).unwrap();
        let tbus = &mut bus_vec[*jj];
        tbus.gs += raw_branch.gj * base_mva;
        tbus.bs += raw_branch.bj * base_mva;
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

    let mut dcline_vec = vec![];
    for raw_dcline in &network.two_terminal_dc {
        let busr = {
            let ipr = raw_dcline.ipr as usize; // rectifier
            let ipri = bus_index.get(&ipr).unwrap();
            &bus_vec[*ipri]
        };
        let busi = {
            let ipi = raw_dcline.ipi as usize; // inverter
            let ipii = bus_index.get(&ipi).unwrap();
            &bus_vec[*ipii]
        };

        let setvl = raw_dcline.setvl.abs();
        let p_mw = match raw_dcline.mdc {
            1 => setvl,
            2 => setvl * raw_dcline.vschd / 1000.0,
            _ => 0.0,
        };

        let (qr_min, qr_max) = hvdc_q_lims(raw_dcline.alfmx, raw_dcline.alfmn, p_mw);
        let (qi_min, qi_max) = hvdc_q_lims(raw_dcline.gammx, raw_dcline.gammn, p_mw);

        let dcline = crate::DCLine::new(raw_dcline.ipr as usize, raw_dcline.ipi as usize)
            .br_status(if raw_dcline.mdc == 0 {
                OUT_OF_SERVICE
            } else {
                IN_SERVICE
            })
            .pf(p_mw)
            .pt(p_mw)
            .vf(busr.vm)
            .vt(busi.vm)
            .pmin(0.85 * p_mw)
            .pmax(1.15 * p_mw)
            .qminf(qr_min)
            .qmaxf(qr_max)
            .qmint(qi_min)
            .qmaxt(qi_max)
            .build()?;
        dcline_vec.push(dcline);
    }

    Ok((case, bus_vec, gen_vec, branch_vec, dcline_vec))
}

// Calculate HVDC line reactive power limits.
//
// This function calculates the reactive power at the rectifier or inverter end.
// It is assumed the maximum overlap angle is 60 degree (see Kimbark's book).
//
// Based on `psse_convert_hvdc_Qlims` from `psse_convert_hvdc.m` in MATPOWER 7.1.
fn hvdc_q_lims(alphamax: f64, alphamin: f64, p_mw: f64) -> (f64, f64) {
    // Minimum reactive power calculated under assumption of no overlap angle
    // i.e. power factor equals to tan(alpha).
    let q_min = p_mw * alphamin.to_radians().tan();

    // Maximum reactive power calculated when overlap angle reaches max
    // value (60 deg). I.e.
    //      cos(phi) = 1/2*(cos(alpha)+cos(delta))
    //      Q = P*tan(phi)

    let phi = (0.5 * (alphamax.to_radians().cos() + 60_f64.to_radians().cos()))
        .to_radians()
        .acos();
    let q_max = p_mw * phi.to_radians().tan();

    (
        if q_min < 0.0 { -q_min } else { q_min },
        if q_max < 0.0 { -q_max } else { q_max },
    )
}

pub fn case_to_raw(
    case: &crate::Case,
    bus: &[crate::Bus],
    gen: &[crate::Gen],
    branch: &[crate::Branch],
    dcline: &[crate::DCLine],
) -> power_flow_data::Network {
    let bus_index = crate::bus_index(bus);

    let buses = bus
        .iter()
        .map(|bus| power_flow_data::Bus {
            i: bus.bus_i as BusNum,
            name: Default::default(),
            basekv: bus.base_kv,
            ide: bus.bus_type as i8,
            area: bus.bus_area as AreaNum,
            zone: bus.zone as ZoneNum,
            owner: 0,
            vm: bus.vm,
            va: bus.va,
            nvhi: bus.vmax,
            nvlo: bus.vmin,
            evhi: bus.vmax,
            evlo: bus.vmin,
        })
        .collect();

    let is_load = |bus: &&crate::Bus| bus.pd != 0.0 || bus.qd != 0.0;
    let is_shunt = |bus: &&crate::Bus| bus.gs != 0.0 || bus.bs != 0.0;
    let is_tfmr = |br: &&crate::Branch| br.tap != 0.0 || br.shift != 0.0;

    let mut loads: Vec<power_flow_data::Load> = bus
        .iter()
        .filter(is_load)
        .map(|bus| power_flow_data::Load {
            i: bus.bus_i as BusNum,
            id: ArrayString::from("1").unwrap(),
            area: bus.bus_area as AreaNum,
            zone: bus.zone as ZoneNum,
            pl: bus.pd,
            ql: bus.qd,
            ..Default::default()
        })
        .collect();

    {
        let mut load_counts: HashMap<usize, usize> = bus
            .iter()
            .filter(is_load)
            .map(|bus| (bus.bus_i, 1))
            .collect();

        loads.extend(gen.iter().filter(|gen| gen.is_load()).map(|dl| {
            let dlbus = &bus[bus_index[&dl.gen_bus]];
            let c = load_counts.entry(dl.gen_bus).or_insert(0);
            *c += 1;
            power_flow_data::Load {
                i: dl.gen_bus as BusNum,
                id: ArrayString::from(&format!("{}", *c)).unwrap(),
                status: dl.gen_status as Stat,
                area: dlbus.bus_area as AreaNum,
                zone: dlbus.zone as ZoneNum,
                pl: -dl.pmin,
                ql: -dl.qmin,
                ..Default::default()
            }
        }));
    }

    let fixed_shunts = bus
        .iter()
        .filter(is_shunt)
        .map(|bus| power_flow_data::FixedShunt {
            i: bus.bus_i as BusNum,
            id: ArrayString::from("1").unwrap(),
            gl: bus.gs,
            bl: bus.bs,
            ..Default::default()
        })
        .collect();

    let generators = gen
        .iter()
        .filter(|gen| !gen.is_load())
        .map(|gen| power_flow_data::Generator {
            i: gen.gen_bus as BusNum,
            id: ArrayString::from("1").unwrap(),
            pg: gen.pg,
            qg: gen.qg,
            qt: gen.qmax,
            qb: gen.qmin,
            vs: gen.vg,
            mbase: gen.mbase,
            stat: gen.gen_status as Stat,
            pt: gen.pmax,
            pb: gen.pmin,
            ..Default::default()
        })
        .collect();

    let branches = {
        let mut ckts: HashMap<(usize, usize), usize> = HashMap::new();
        branch
            .iter()
            .filter(|br| !is_tfmr(br))
            .map(|br| {
                let ckt = ckts.entry((br.f_bus, br.t_bus)).or_insert(0);
                *ckt += 1;
                power_flow_data::Branch {
                    i: br.f_bus as BusNum,
                    j: br.t_bus as BusNum,
                    ckt: ArrayString::from(&format!("{}", ckt)).unwrap(),
                    r: br.br_r,
                    x: br.br_x,
                    b: br.br_b,
                    rate_a: br.rate_a,
                    rate_b: br.rate_b,
                    rate_c: br.rate_c,
                    st: br.br_status as Stat,
                    ..Default::default()
                }
            })
            .collect()
    };

    let transformers = {
        let mut ckts: HashMap<(usize, usize), usize> = HashMap::new();
        branch
            .iter()
            .filter(is_tfmr)
            .map(|tr| {
                let ckt = ckts.entry((tr.f_bus, tr.t_bus)).or_insert(0);
                *ckt += 1;
                power_flow_data::Transformer {
                    i: tr.f_bus as BusNum,
                    j: tr.t_bus as BusNum,
                    ckt: ArrayString::from(&format!("{}", ckt)).unwrap(),
                    stat: tr.br_status as Stat,
                    r1_2: tr.br_r,
                    x1_2: tr.br_x,
                    sbase1_2: case.base_mva,
                    windv1: tr.tap,
                    ang1: tr.shift,
                    rata1: tr.rate_a,
                    ratb1: tr.rate_b,
                    ratc1: tr.rate_c,
                    ..Default::default()
                }
            })
            .collect()
    };

    let two_terminal_dc = dcline
        .iter()
        .enumerate()
        .map(|(i, dcline)| power_flow_data::TwoTerminalDCLine {
            name: ArrayString::from(&format!("DCLINE {}", i + 1)).unwrap(),
            setvl: dcline.pf,
            ipr: dcline.f_bus as BusNum,
            ebasr: bus[bus_index[&dcline.f_bus]].base_kv,
            ipi: dcline.t_bus as BusNum,
            ebasi: bus[bus_index[&dcline.t_bus]].base_kv,
            ..Default::default()
        })
        .collect();

    power_flow_data::Network {
        version: 0,
        caseid: CaseID {
            ic: 0,
            sbase: case.base_mva,
            rev: Some(33),
            basfrq: case.f,
            ..Default::default()
        },
        buses,
        loads,
        fixed_shunts,
        generators,
        branches,
        transformers,
        two_terminal_dc,
        ..Default::default()
    }
}
