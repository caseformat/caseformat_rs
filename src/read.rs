use anyhow::{format_err, Result};
use std::fs::File;
use std::io::{read_to_string, Read, Seek};
use std::path::{Path, PathBuf};
use zip::{result::ZipError, ZipArchive};

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

pub fn read_zip(
    reader: impl Read + Seek,
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
    let mut zip_archive = ZipArchive::new(reader).unwrap();

    let case = match zip_archive.by_name(CASE_FILE) {
        Ok(case_file) => {
            read_case_file(case_file).map_err(|err| format_err!("case file read error: {}", err))?
        }
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("case file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("case file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!("case file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => {
                return Err(format_err!("zip archive must contain {} file", CASE_FILE));
            }
        },
    };

    let bus = match zip_archive.by_name(BUS_FILE) {
        Ok(bus_file) => {
            read_bus_file(bus_file).map_err(|err| format_err!("bus file read error: {}", err))?
        }
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("bus file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("bus file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!("bus file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => {
                return Err(format_err!("zip archive must contain {} file", BUS_FILE));
            }
        },
    };

    let gen = match zip_archive.by_name(GEN_FILE) {
        Ok(gen_file) => {
            read_gen_file(gen_file).map_err(|err| format_err!("gen file read error: {}", err))?
        }
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("gen file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("gen file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!("gen file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => Vec::default(),
        },
    };

    let branch = match zip_archive.by_name(BRANCH_FILE) {
        Ok(branch_file) => read_branch_file(branch_file)
            .map_err(|err| format_err!("branch file read error: {}", err))?,
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("branch file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("branch file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!(
                    "branch file unsupported archive error: {}",
                    err
                ));
            }
            ZipError::FileNotFound => Vec::default(),
        },
    };

    let gencost = match zip_archive.by_name(GENCOST_FILE) {
        Ok(gencost_file) => read_gencost_file(gencost_file)
            .map_err(|err| format_err!("gencost file read error: {}", err))?,
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("gencost file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("gencost file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!(
                    "gencost file unsupported archive error: {}",
                    err
                ));
            }
            ZipError::FileNotFound => Vec::default(),
        },
    };

    let dcline = match zip_archive.by_name(DCLINE_FILE) {
        Ok(dcline_file) => read_dcline_file(dcline_file)
            .map_err(|err| format_err!("dcline file read error: {}", err))?,
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("dcline file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("dcline file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!(
                    "dcline file unsupported archive error: {}",
                    err
                ));
            }
            ZipError::FileNotFound => Vec::default(),
        },
    };

    let readme = match zip_archive.by_name(README_FILE) {
        Ok(readme_file) => Some(read_to_string(readme_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("readme file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("readme file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!(
                    "readme file unsupported archive error: {}",
                    err
                ));
            }
            ZipError::FileNotFound => None,
        },
    };

    let license = match zip_archive.by_name(LICENSE_FILE) {
        Ok(license_file) => Some(read_to_string(license_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format_err!("license file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format_err!("license file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format_err!(
                    "license file unsupported archive error: {}",
                    err
                ));
            }
            ZipError::FileNotFound => None,
        },
    };

    Ok((case, bus, gen, branch, gencost, dcline, readme, license))
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
