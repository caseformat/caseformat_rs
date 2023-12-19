use anyhow::{format_err, Result};
use std::io::{Seek, Write};
use std::path::PathBuf;
use std::{fs, fs::File};
use time::OffsetDateTime;
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime};

use crate::read::*;
use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

pub fn write_zip<W>(
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
    W: Write + Seek,
{
    let mut ar = zip::ZipWriter::new(writer);

    let now_utc = OffsetDateTime::now_utc();
    let now_dt = DateTime::try_from(now_utc)?;

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o664)
        .last_modified_time(now_dt);

    ar.start_file(CASE_FILE, options)?;
    ar.write_all(
        &write_case(Vec::default(), case)
            .map_err(|err| format_err!("case file write error: {}", err))?,
    )?;

    if !bus.is_empty() {
        ar.start_file(BUS_FILE, options)?;
        ar.write_all(
            &write_bus(Vec::default(), bus)
                .map_err(|err| format_err!("bus file write error: {}", err))?,
        )?;
    }
    if !gen.is_empty() {
        ar.start_file(GEN_FILE, options)?;
        ar.write_all(
            &write_gen(Vec::default(), gen)
                .map_err(|err| format_err!("gen file write error: {}", err))?,
        )?;
    }
    if !branch.is_empty() {
        ar.start_file(BRANCH_FILE, options)?;
        ar.write_all(
            &write_branch(Vec::default(), branch)
                .map_err(|err| format_err!("branch file write error: {}", err))?,
        )?;
    }
    if !gencost.is_empty() {
        ar.start_file(GENCOST_FILE, options)?;
        ar.write_all(
            &write_gencost(Vec::default(), gencost)
                .map_err(|err| format_err!("gencost file write error: {}", err))?,
        )?;
    }
    if !dcline.is_empty() {
        ar.start_file(DCLINE_FILE, options)?;
        ar.write_all(
            &write_dcline(Vec::default(), dcline)
                .map_err(|err| format_err!("dcline file write error: {}", err))?,
        )?;
    }

    if let Some(readme) = readme {
        ar.start_file(README_FILE, options)?;
        ar.write_all(readme.as_bytes())?;
    }
    if let Some(license) = license {
        ar.start_file(LICENSE_FILE, options)?;
        ar.write_all(license.as_bytes())?;
    }

    Ok(ar.finish()?)
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

const BUS_HEADER: [&str; 13] = [
    "BUS_I", "BUS_TYPE", "PD", "QD", "GS", "BS", "BUS_AREA", "VM", "VA", "BASE_KV", "ZONE", "VMAX",
    "VMIN",
];
const BUS_HEADER_OPF: [&str; 17] = [
    "BUS_I", "BUS_TYPE", "PD", "QD", "GS", "BS", "BUS_AREA", "VM", "VA", "BASE_KV", "ZONE", "VMAX",
    "VMIN", "LAM_P", "LAM_Q", "MU_VMAX", "MU_VMIN",
];

const GEN_HEADER: [&str; 10] = [
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
const GEN_HEADER_2: [&str; 21] = [
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
const GEN_HEADER_OPF: [&str; 25] = [
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

const BRANCH_HEADER: [&str; 13] = [
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
const BRANCH_HEADER_PF: [&str; 17] = [
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
const BRANCH_HEADER_OPF: [&str; 21] = [
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
const GENCOST_HEADER: [&str; 4] = ["MODEL", "STARTUP", "SHUTDOWN", "NCOST"];

const DCLINE_HEADER: [&str; 17] = [
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

const DCLINE_HEADER_OPF: [&str; 23] = [
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
