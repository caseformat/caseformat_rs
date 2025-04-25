use anyhow::{format_err, Result};
use std::fs::File;
use std::io::{read_to_string, Read};
use std::path::{Path, PathBuf};

use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

pub(crate) const CASE_FILE: &str = "case.csv";
pub(crate) const BUS_FILE: &str = "bus.csv";
pub(crate) const GEN_FILE: &str = "gen.csv";
pub(crate) const BRANCH_FILE: &str = "branch.csv";
pub(crate) const GENCOST_FILE: &str = "gencost.csv";
pub(crate) const DCLINE_FILE: &str = "dcline.csv";

pub(crate) const README_FILE: &str = "README";
pub(crate) const LICENSE_FILE: &str = "LICENSE";

#[macro_export]
macro_rules! parse_record {
    ($iter:expr, $T:ty) => {{
        match $iter.next() {
            Some(field) => match field.parse::<$T>() {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow::format_err!("parse error ({}): {}", field, err));
                }
            },
            None => {
                return Err(anyhow::format_err!("record must exist"));
            }
        }
    }};
}

#[macro_export]
macro_rules! parse_optional_record {
    ($iter:expr, $T:ty) => {{
        match $iter.next() {
            Some(field) => match field.parse::<$T>() {
                Ok(value) => Some(value),
                Err(err) => {
                    return Err(anyhow::format_err!("parse error ({}): {}", field, err));
                }
            },
            None => None,
        }
    }};
}

pub fn read_tar(
    reader: impl Read,
) -> Result<(
    Case,
    Vec<Bus>,
    Vec<Gen>,
    Vec<Branch>,
    Vec<GenCost>,
    Vec<DCLine>,
    Option<String>,
    Option<String>,
)> {
    let mut ar = tar::Archive::new(reader);

    let mut case = Some(Case::new("").build().unwrap());
    // let mut case = None;
    let mut bus = Vec::default();
    let mut gen = Vec::default();
    let mut branch = Vec::default();
    let mut gencost = Vec::default();
    let mut dcline = Vec::default();
    let mut readme = None;
    let mut license = None;

    for entry in ar.entries()? {
        let file = entry?;
        // let s = read_to_string(file).unwrap();
        // println!("{}", s.len());
        match file
            .path()?
            .as_os_str()
            .to_os_string()
            .into_string()
            .map_err(|err| format_err!("path error for: {:?}", err))?
            .as_str()
        {
            CASE_FILE => {
                case = Some(read_case_file(file)?);
            }
                BUS_FILE => {
                    bus = read_bus_file(file)?;
                }
                GEN_FILE => {
                    gen = read_gen_file(file)?;
                }
                BRANCH_FILE => {
                    branch = read_branch_file(file)?;
                }
                GENCOST_FILE => {
                    gencost = read_gencost_file(file)?;
                }
                DCLINE_FILE => {
                    dcline = read_dcline_file(file)?;
                }
                README_FILE => {
                    readme = Some(read_to_string(file)?);
                }
                LICENSE_FILE => {
                    license = Some(read_to_string(file)?);
                }
            _ => {}
        }
    }

    if case.is_none() {
        return Err(format_err!("archive must contain {} file", CASE_FILE));
    }

    Ok((
        case.unwrap(),
        bus,
        gen,
        branch,
        gencost,
        dcline,
        readme,
        license,
    ))
}

pub fn read_dir(
    dir_path: &PathBuf,
) -> Result<(
    Case,
    Vec<Bus>,
    Vec<Gen>,
    Vec<Branch>,
    Vec<GenCost>,
    Vec<DCLine>,
    Option<String>,
    Option<String>,
)> {
    let case_path = dir_path.join(Path::new(CASE_FILE));
    let case_file = File::open(case_path)?;
    let case =
        read_case_file(case_file).map_err(|err| format_err!("case file read error: {}", err))?;

    let bus_path = dir_path.join(Path::new(BUS_FILE));
    let bus_file = File::open(bus_path)?;
    let bus = read_bus_file(bus_file).map_err(|err| format_err!("bus file read error: {}", err))?;

    let gen_path = dir_path.join(Path::new(GEN_FILE));
    let gen = if gen_path.exists() {
        let gen_file = File::open(gen_path)?;
        read_gen_file(gen_file).map_err(|err| format_err!("gen file read error: {}", err))?
    } else {
        Vec::default()
    };

    let branch_path = dir_path.join(Path::new(BRANCH_FILE));
    let branch = if branch_path.exists() {
        let branch_file = File::open(branch_path)?;
        read_branch_file(branch_file)
            .map_err(|err| format_err!("branch file read error: {}", err))?
    } else {
        Vec::default()
    };

    let gencost_path = dir_path.join(Path::new(GENCOST_FILE));
    let gencost = if gencost_path.exists() {
        let gencost_file = File::open(gencost_path)?;
        read_gencost_file(gencost_file)
            .map_err(|err| format_err!("gencost file read error: {}", err))?
    } else {
        Vec::default()
    };

    let dcline_path = dir_path.join(Path::new(DCLINE_FILE));
    let dcline = if dcline_path.exists() {
        let dcline_file = File::open(dcline_path)?;
        read_dcline_file(dcline_file)
            .map_err(|err| format_err!("dcline file read error: {}", err))?
    } else {
        Vec::default()
    };

    let readme_path = dir_path.join(Path::new(README_FILE));
    let readme = if readme_path.exists() {
        let readme_file = File::open(readme_path)?;
        Some(read_to_string(readme_file)?)
    } else {
        None
    };

    let license_path = dir_path.join(Path::new(LICENSE_FILE));
    let license = if license_path.exists() {
        let license_file = File::open(license_path)?;
        Some(read_to_string(license_file)?)
    } else {
        None
    };

    Ok((case, bus, gen, branch, gencost, dcline, readme, license))
}

fn read_case_file(file_reader: impl Read) -> Result<Case> {
    let mut reader = csv::Reader::from_reader(file_reader);
    let case: Case = match reader.records().next() {
        Some(result) => Case::from_string_record(result?)?,
        None => {
            return Err(format_err!("one case record must exist"));
        }
    };
    Ok(case)
}

fn read_bus_file(file_reader: impl Read) -> Result<Vec<Bus>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut bus = Vec::new();
    for result in csv_reader.records() {
        bus.push(Bus::from_string_record(result?)?);
    }
    Ok(bus)
}

fn read_gen_file(file_reader: impl Read) -> Result<Vec<Gen>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut gen = Vec::new();
    for result in csv_reader.records() {
        gen.push(Gen::from_string_record(result?)?);
    }
    Ok(gen)
}

fn read_branch_file(file_reader: impl Read) -> Result<Vec<Branch>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut branch = Vec::new();
    for result in csv_reader.records() {
        branch.push(Branch::from_string_record(result?)?);
    }
    Ok(branch)
}

fn read_gencost_file(file_reader: impl Read) -> Result<Vec<GenCost>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut gencost = Vec::new();
    for result in csv_reader.records() {
        gencost.push(GenCost::from_string_record(result?)?);
    }
    Ok(gencost)
}

fn read_dcline_file(file_reader: impl Read) -> Result<Vec<DCLine>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut dcline = Vec::new();
    for result in csv_reader.records() {
        dcline.push(DCLine::from_string_record(result?)?);
    }
    Ok(dcline)
}
