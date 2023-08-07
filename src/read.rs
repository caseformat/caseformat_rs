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
) -> Result<
    (
        Case,
        Vec<Bus>,
        Option<Vec<Gen>>,
        Option<Vec<Branch>>,
        Option<Vec<GenCost>>,
        Option<Vec<DCLine>>,
    ),
    String,
> {
    let file = File::open(zip_path).expect("Unable to open input file");
    let reader = BufReader::new(file);
    let mut zip_archive = ZipArchive::new(reader).unwrap();

    let case = match zip_archive.by_name(CASE_FILE) {
        Ok(case_file) => read_case_file(case_file)?,
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("case file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("case file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("case file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => {
                return Err(format!("zip archive must contain {} file", CASE_FILE));
            }
        },
    };

    let bus = match zip_archive.by_name(BUS_FILE) {
        Ok(bus_file) => read_bus_file(bus_file)?,
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("bus file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("bus file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("bus file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => {
                return Err(format!("zip archive must contain {} file", BUS_FILE));
            }
        },
    };

    let gen = match zip_archive.by_name(GEN_FILE) {
        Ok(gen_file) => Some(read_gen_file(gen_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("gen file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("gen file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("gen file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => None,
        },
    };

    let branch = match zip_archive.by_name(BRANCH_FILE) {
        Ok(branch_file) => Some(read_branch_file(branch_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("branch file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("branch file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("branch file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => None,
        },
    };

    let gencost = match zip_archive.by_name(GENCOST_FILE) {
        Ok(gencost_file) => Some(read_gencost_file(gencost_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("gencost file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("gencost file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("gencost file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => None,
        },
    };

    let dcline = match zip_archive.by_name(DCLINE_FILE) {
        Ok(dcline_file) => Some(read_dcline_file(dcline_file)?),
        Err(zip_err) => match zip_err {
            ZipError::Io(err) => {
                return Err(format!("dcline file I/O error: {}", err));
            }
            ZipError::InvalidArchive(err) => {
                return Err(format!("dcline file invalid archive error: {}", err));
            }
            ZipError::UnsupportedArchive(err) => {
                return Err(format!("dcline file unsupported archive error: {}", err));
            }
            ZipError::FileNotFound => None,
        },
    };

    Ok((case, bus, gen, branch, gencost, dcline))
}

pub fn read_dir(
    dir_path: &PathBuf,
) -> Result<
    (
        Case,
        Vec<Bus>,
        Option<Vec<Gen>>,
        Option<Vec<Branch>>,
        Option<Vec<GenCost>>,
        Option<Vec<DCLine>>,
    ),
    String,
> {
    let case_path = dir_path.join(Path::new(CASE_FILE));
    let case = match File::open(case_path) {
        Ok(case_file) => read_case_file(case_file)?,
        Err(err) => {
            return Err(format!("case file error: {}", err));
        }
    };

    let bus_path = dir_path.join(Path::new(BUS_FILE));
    let bus = match File::open(bus_path) {
        Ok(bus_file) => read_bus_file(bus_file)?,
        Err(err) => {
            return Err(format!("bus file error: {}", err));
        }
    };

    let gen_path = dir_path.join(Path::new(GEN_FILE));
    let gen = if gen_path.exists() {
        match File::open(gen_path) {
            Ok(gen_file) => Some(read_gen_file(gen_file)?),
            Err(err) => {
                return Err(format!("gen file error: {}", err));
            }
        }
    } else {
        None
    };

    let branch_path = dir_path.join(Path::new(BRANCH_FILE));
    let branch = if branch_path.exists() {
        match File::open(branch_path) {
            Ok(branch_file) => Some(read_branch_file(branch_file)?),
            Err(err) => {
                return Err(format!("branch file error: {}", err));
            }
        }
    } else {
        None
    };

    let gencost_path = dir_path.join(Path::new(GENCOST_FILE));
    let gencost = if gencost_path.exists() {
        match File::open(gencost_path) {
            Ok(gencost_file) => Some(read_gencost_file(gencost_file)?),
            Err(err) => {
                return Err(format!("gencost file error: {}", err));
            }
        }
    } else {
        None
    };

    let dcline_path = dir_path.join(Path::new(DCLINE_FILE));
    let dcline = if dcline_path.exists() {
        match File::open(dcline_path) {
            Ok(dcline_file) => Some(read_dcline_file(dcline_file)?),
            Err(err) => {
                return Err(format!("dcline file error: {}", err));
            }
        }
    } else {
        None
    };

    Ok((case, bus, gen, branch, gencost, dcline))
}

fn read_case_file(file_reader: impl Read) -> Result<Case, String> {
    let mut reader = csv::Reader::from_reader(file_reader);
    let case: Case = match reader.deserialize().next() {
        Some(result) => match result {
            Ok(case) => case,
            Err(err) => {
                return Err(format!("case record parse error: {}", err));
            }
        },
        None => {
            return Err("case record must exist".to_string());
        }
    };
    Ok(case)
}

fn read_bus_file(file_reader: impl Read) -> Result<Vec<Bus>, String> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut bus = Vec::new();
    for result in csv_reader.deserialize() {
        bus.push(match result {
            Ok(bus) => bus,
            Err(err) => {
                return Err(format!("bus record parse error: {}", err));
            }
        });
    }
    Ok(bus)
}

fn read_gen_file(file_reader: impl Read) -> Result<Vec<Gen>, String> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut gen = Vec::new();
    for result in csv_reader.deserialize() {
        gen.push(match result {
            Ok(g) => g,
            Err(err) => {
                return Err(format!("gen record parse error: {}", err));
            }
        });
    }
    Ok(gen)
}

fn read_branch_file(file_reader: impl Read) -> Result<Vec<Branch>, String> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut branch = Vec::new();
    for result in csv_reader.deserialize() {
        branch.push(match result {
            Ok(br) => br,
            Err(err) => {
                return Err(format!("branch record parse error: {}", err));
            }
        });
    }
    Ok(branch)
}

fn read_gencost_file(file_reader: impl Read) -> Result<Vec<GenCost>, String> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut gencost = Vec::new();
    for result in csv_reader.records() {
        match result {
            Ok(record) => {
                gencost.push(GenCost::from_string_record(record)?);
                // match GenCost::from_string_record(record) {
                //     Ok(cost) => {cost}
                //     Err(err) => {}
                // }
            }
            Err(err) => {
                return Err(format!("gencost record parse error: {}", err));
            }
        }
    }
    Ok(gencost)
}

fn read_dcline_file(file_reader: impl Read) -> Result<Vec<DCLine>, String> {
    let mut csv_reader = csv::Reader::from_reader(file_reader);
    let mut dcline = Vec::new();
    for result in csv_reader.deserialize() {
        dcline.push(match result {
            Ok(ln) => ln,
            Err(err) => {
                return Err(format!("dcline record parse error: {}", err));
            }
        });
    }
    Ok(dcline)
}
