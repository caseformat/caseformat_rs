use anyhow::{format_err, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::ZipArchive;

use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

const CASE_FILE: &str = "case.csv";
const BUS_FILE: &str = "bus.csv";
const GEN_FILE: &str = "gen.csv";
const BRANCH_FILE: &str = "branch.csv";
const GENCOST_FILE: &str = "gencost.csv";
const DCLINE_FILE: &str = "dcline.csv";

pub fn read_zip(
    zip_path: &PathBuf,
) -> Result<(
    Case,
    Vec<Bus>,
    Vec<Gen>,
    Vec<Branch>,
    Vec<GenCost>,
    Vec<DCLine>,
)> {
    let file = File::open(zip_path).expect("Unable to open input file");
    let reader = BufReader::new(file);
    let mut zip_archive = ZipArchive::new(reader).unwrap();

    let case = match zip_archive.by_name(CASE_FILE) {
        Ok(case_file) => read_case_file(case_file)?,
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
        Ok(bus_file) => read_bus_file(bus_file)?,
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
        Ok(gen_file) => read_gen_file(gen_file)?,
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
        Ok(branch_file) => read_branch_file(branch_file)?,
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
        Ok(gencost_file) => read_gencost_file(gencost_file)?,
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
        Ok(dcline_file) => read_dcline_file(dcline_file)?,
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

    Ok((case, bus, gen, branch, gencost, dcline))
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
)> {
    let case_path = dir_path.join(Path::new(CASE_FILE));
    let case_file = File::open(case_path)?;
    let case = read_case_file(case_file)?;

    let bus_path = dir_path.join(Path::new(BUS_FILE));
    let bus_file = File::open(bus_path)?;
    let bus = read_bus_file(bus_file)?;

    let gen_path = dir_path.join(Path::new(GEN_FILE));
    let gen = if gen_path.exists() {
        let gen_file = File::open(gen_path)?;
        read_gen_file(gen_file)?
    } else {
        Vec::default()
    };

    let branch_path = dir_path.join(Path::new(BRANCH_FILE));
    let branch = if branch_path.exists() {
        let branch_file = File::open(branch_path)?;
        read_branch_file(branch_file)?
    } else {
        Vec::default()
    };

    let gencost_path = dir_path.join(Path::new(GENCOST_FILE));
    let gencost = if gencost_path.exists() {
        let gencost_file = File::open(gencost_path)?;
        read_gencost_file(gencost_file)?
    } else {
        Vec::default()
    };

    let dcline_path = dir_path.join(Path::new(DCLINE_FILE));
    let dcline = if dcline_path.exists() {
        let dcline_file = File::open(dcline_path)?;
        read_dcline_file(dcline_file)?
    } else {
        Vec::default()
    };

    Ok((case, bus, gen, branch, gencost, dcline))
}

fn read_case_file(file_reader: impl Read) -> Result<Case> {
    let mut reader = csv::Reader::from_reader(file_reader);
    let case: Case = match reader.deserialize().next() {
        Some(result) => result?,
        None => {
            return Err(format_err!("one case record must exist"));
        }
    };
    Ok(case)
}

fn read_bus_file(file_reader: impl Read) -> Result<Vec<Bus>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut bus = Vec::new();
    for result in csv_reader.deserialize() {
        bus.push(result?);
    }
    Ok(bus)
}

fn read_gen_file(file_reader: impl Read) -> Result<Vec<Gen>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut gen = Vec::new();
    for result in csv_reader.deserialize() {
        gen.push(result?);
    }
    Ok(gen)
}

fn read_branch_file(file_reader: impl Read) -> Result<Vec<Branch>> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut branch = Vec::new();
    for result in csv_reader.deserialize() {
        branch.push(result?);
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
    for result in csv_reader.deserialize() {
        dcline.push(match result {
            Ok(ln) => ln,
            Err(err) => {
                return Err(format_err!("dcline record parse error: {}", err));
            }
        });
    }
    Ok(dcline)
}
