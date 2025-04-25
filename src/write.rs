use anyhow::{format_err, Result};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, fs::File};

use crate::read::*;
use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

macro_rules! append {
    ($ar:expr, $records:expr, $path:expr, $mtime:expr) => {
        if !$records.is_empty() {
            let mut wtr = csv::Writer::from_writer(vec![]);
            for record in $records {
                wtr.serialize(record)?;
            }
            wtr.flush()?;

            let data = wtr.into_inner()?;

            let mut header = tar::Header::new_gnu();
            header.set_path($path)?;
            header.set_mode(0o664);
            header.set_mtime($mtime);
            header.set_size(data.len() as u64);
            header.set_cksum();

            $ar.append(&header, data.as_slice())?;
        }
    };
}

pub fn write_tar<W>(
    writer: W,
    case: &Case,
    bus: &[Bus],
    gen: &[Gen],
    branch: &[Branch],
    gencost: &[GenCost],
    dcline: &[DCLine],
    readme: Option<String>,
    license: Option<String>,
) -> Result<W>
where
    W: Write,
{
    let mut ar = tar::Builder::new(writer);

    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let mtime = since_epoch.as_secs();

    append!(ar, &vec![case], CASE_FILE, mtime);
    append!(ar, bus, BUS_FILE, mtime);
    append!(ar, gen, GEN_FILE, mtime);
    append!(ar, branch, BRANCH_FILE, mtime);
    append!(ar, gencost, GENCOST_FILE, mtime);
    append!(ar, dcline, DCLINE_FILE, mtime);

    if let Some(readme) = readme {
        let data = readme.as_bytes();

        let mut header = tar::Header::new_gnu();
        header.set_path(README_FILE)?;
        header.set_mode(0o664);
        header.set_mtime(mtime);
        header.set_size(data.len() as u64);
        header.set_cksum();

        ar.append(&header, data)?;
    }
    if let Some(license) = license {
        let data = license.as_bytes();

        let mut header = tar::Header::new_gnu();
        header.set_path(LICENSE_FILE)?;
        header.set_mode(0o664);
        header.set_mtime(mtime);
        header.set_size(data.len() as u64);
        header.set_cksum();

        ar.append(&header, data)?;
    }
    ar.finish()?;
    Ok(ar.into_inner()?)
}

pub fn write_dir(
    dir_path: &PathBuf,
    case: &Case,
    bus: &[Bus],
    gen: &[Gen],
    branch: &[Branch],
    gencost: &[GenCost],
    dcline: &[DCLine],
    readme: Option<String>,
    license: Option<String>,
) -> Result<()> {
    let case_file = File::create(dir_path.join(CASE_FILE))?;
    write_case(case_file, case)?;

    if !bus.is_empty() {
        let file = File::create(dir_path.join(BUS_FILE))?;
        write_bus(file, bus)?;
    }
    if !gen.is_empty() {
        let file = File::create(dir_path.join(GEN_FILE))?;
        write_gen(file, gen)?;
    }
    if !branch.is_empty() {
        let file = File::create(dir_path.join(BRANCH_FILE))?;
        write_branch(file, branch)?;
    }
    if !gencost.is_empty() {
        let file = File::create(dir_path.join(GENCOST_FILE))?;
        write_gencost(file, gencost)?;
    }
    if !dcline.is_empty() {
        let file = File::create(dir_path.join(DCLINE_FILE))?;
        write_dcline(file, dcline)?;
    }

    if let Some(readme) = readme {
        fs::write(dir_path.join(README_FILE), readme)?;
    }
    if let Some(license) = license {
        fs::write(dir_path.join(LICENSE_FILE), license)?;
    }

    Ok(())
}

fn write_case<W: Write>(wtr: W, case: &Case) -> Result<W> {
    let mut w = csv::Writer::from_writer(wtr);
    if case.f.is_none() {
        w.write_record(CASE_HEADER)?;
    } else {
        w.write_record(CASE_HEADER_F)?;
    }
    w.write_record(&case.to_string_record())?;
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

fn write_bus<W: Write>(wtr: W, bus: &[Bus]) -> Result<W> {
    let is_opf = bus.iter().any(|b| b.is_opf());
    let mut w = csv::Writer::from_writer(wtr);
    if !is_opf {
        w.write_record(BUS_HEADER)?;
    } else {
        w.write_record(BUS_HEADER_OPF)?;
    }
    for r in bus {
        w.write_record(&r.to_string_record(is_opf))?;
    }
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

fn write_gen<W: Write>(wtr: W, gen: &[Gen]) -> Result<W> {
    let is_version_1 = gen.iter().any(|g| g.is_version_1());
    let is_opf = gen.iter().any(|g| g.is_opf());
    let mut w = csv::Writer::from_writer(wtr);
    if !is_opf && is_version_1 {
        w.write_record(GEN_HEADER)?;
    } else if !is_opf {
        w.write_record(GEN_HEADER_2)?;
    } else {
        w.write_record(GEN_HEADER_OPF)?;
    }
    for r in gen {
        w.write_record(&r.to_string_record(is_version_1, is_opf))?;
    }
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

fn write_branch<W: Write>(wtr: W, branch: &[Branch]) -> Result<W> {
    let is_pf = branch.iter().any(|br| br.is_pf());
    let is_opf = branch.iter().any(|br| br.is_opf());
    let mut w = csv::Writer::from_writer(wtr);
    if !is_opf && !is_pf {
        w.write_record(BRANCH_HEADER)?;
    } else if !is_opf {
        w.write_record(BRANCH_HEADER_PF)?;
    } else {
        w.write_record(BRANCH_HEADER_OPF)?;
    }
    for r in branch {
        w.write_record(&r.to_string_record(is_pf, is_opf))?;
    }
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

fn write_gencost<W: Write>(wtr: W, gencost: &[GenCost]) -> Result<W> {
    let is_poly = gencost.iter().any(|c| c.is_polynomial());
    let is_pwl = gencost.iter().any(|c| c.is_pwl());
    if is_poly && is_pwl {
        return Err(format_err!(
            "cost functions must not be mixed polynomial/pwl"
        ));
    }
    let ncost = gencost.iter().map(|c| c.ncost).max().unwrap_or_default();
    let mut w = csv::Writer::from_writer(wtr);
    let mut header = Vec::from(GENCOST_HEADER.map(|h| h.to_string()));
    if is_poly {
        for i in 0..ncost {
            header.push(format!("C{}", i));
        }
    } else if is_pwl {
        for i in 0..ncost {
            header.push(format!("X{}", i));
            header.push(format!("Y{}", i));
        }
    }
    w.write_record(header)?;
    for r in gencost {
        w.write_record(&r.to_string_record())?;
    }
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

fn write_dcline<W: Write>(wtr: W, dcline: &[DCLine]) -> Result<W> {
    let is_opf = dcline.iter().any(|br| br.is_opf());
    let mut w = csv::Writer::from_writer(wtr);
    if !is_opf {
        w.write_record(DCLINE_HEADER)?;
    } else {
        w.write_record(DCLINE_HEADER_OPF)?;
    }
    for r in dcline {
        w.write_record(&r.to_string_record(is_opf))?;
    }
    w.flush()?;
    w.into_inner().map_err(|err| format_err!("{}", err))
}

const CASE_HEADER: [&str; 3] = ["CASENAME", "VERSION", "BASE_MVA"];
const CASE_HEADER_F: [&str; 4] = ["CASENAME", "VERSION", "BASE_MVA", "F"];

pub(crate) const BUS_HEADER: [&str; 13] = [
    "BUS_I", "BUS_TYPE", "PD", "QD", "GS", "BS", "BUS_AREA", "VM", "VA", "BASE_KV", "ZONE", "VMAX",
    "VMIN",
];
pub(crate) const BUS_HEADER_OPF: [&str; 17] = [
    "BUS_I", "BUS_TYPE", "PD", "QD", "GS", "BS", "BUS_AREA", "VM", "VA", "BASE_KV", "ZONE", "VMAX",
    "VMIN", "LAM_P", "LAM_Q", "MU_VMAX", "MU_VMIN",
];

pub(crate) const GEN_HEADER: [&str; 10] = [
    "GEN_BUS",
    "PG",
    "QG",
    "QMAX",
    "QMIN",
    "VG",
    "MBASE",
    "GEN_STATUS",
    "PMAX",
    "PMIN",
];
pub(crate) const GEN_HEADER_2: [&str; 21] = [
    "GEN_BUS",
    "PG",
    "QG",
    "QMAX",
    "QMIN",
    "VG",
    "MBASE",
    "GEN_STATUS",
    "PMAX",
    "PMIN",
    "PC1",
    "PC2",
    "QC1MIN",
    "QC1MAX",
    "QC2MIN",
    "QC2MAX",
    "RAMP_AGC",
    "RAMP_10",
    "RAMP_30",
    "RAMP_Q",
    "APF",
];
pub(crate) const GEN_HEADER_OPF: [&str; 25] = [
    "GEN_BUS",
    "PG",
    "QG",
    "QMAX",
    "QMIN",
    "VG",
    "MBASE",
    "GEN_STATUS",
    "PMAX",
    "PMIN",
    "PC1",
    "PC2",
    "QC1MIN",
    "QC1MAX",
    "QC2MIN",
    "QC2MAX",
    "RAMP_AGC",
    "RAMP_10",
    "RAMP_30",
    "RAMP_Q",
    "APF",
    "MU_PMAX",
    "MU_PMIN",
    "MU_QMAX",
    "MU_QMIN",
];

pub(crate) const BRANCH_HEADER: [&str; 13] = [
    "F_BUS",
    "T_BUS",
    "BR_R",
    "BR_X",
    "BR_B",
    "RATE_A",
    "RATE_B",
    "RATE_C",
    "TAP",
    "SHIFT",
    "BR_STATUS",
    "ANGMIN",
    "ANGMAX",
];
pub(crate) const BRANCH_HEADER_PF: [&str; 17] = [
    "F_BUS",
    "T_BUS",
    "BR_R",
    "BR_X",
    "BR_B",
    "RATE_A",
    "RATE_B",
    "RATE_C",
    "TAP",
    "SHIFT",
    "BR_STATUS",
    "ANGMIN",
    "ANGMAX",
    "PF",
    "QF",
    "PT",
    "QT",
];
pub(crate) const BRANCH_HEADER_OPF: [&str; 21] = [
    "F_BUS",
    "T_BUS",
    "BR_R",
    "BR_X",
    "BR_B",
    "RATE_A",
    "RATE_B",
    "RATE_C",
    "TAP",
    "SHIFT",
    "BR_STATUS",
    "ANGMIN",
    "ANGMAX",
    "PF",
    "QF",
    "PT",
    "QT",
    "MU_SF",
    "MU_ST",
    "MU_ANGMIN",
    "MU_ANGMAX",
];

// , "C2", "C1", "C0"
// , "X1", "Y1", "X2", "Y2", "X3", "Y3", "X4", "Y4"
pub(crate) const GENCOST_HEADER: [&str; 4] = ["MODEL", "STARTUP", "SHUTDOWN", "NCOST"];

pub(crate) const DCLINE_HEADER: [&str; 17] = [
    "F_BUS",
    "T_BUS",
    "BR_STATUS",
    "PF",
    "PT",
    "QF",
    "QT",
    "VF",
    "VT",
    "PMIN",
    "PMAX",
    "QMINF",
    "QMAXF",
    "QMINT",
    "QMAXT",
    "LOSS0",
    "LOSS1",
];
pub(crate) const DCLINE_HEADER_OPF: [&str; 23] = [
    "F_BUS",
    "T_BUS",
    "BR_STATUS",
    "PF",
    "PT",
    "QF",
    "QT",
    "VF",
    "VT",
    "PMIN",
    "PMAX",
    "QMINF",
    "QMAXF",
    "QMINT",
    "QMAXT",
    "LOSS0",
    "LOSS1",
    "MU_PMIN",
    "MU_PMAX",
    "MU_QMINF",
    "MU_QMAXF",
    "MU_QMINT",
    "MU_QMAXT",
];
