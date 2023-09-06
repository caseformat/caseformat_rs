use anyhow::Result;
use std::path::PathBuf;
use std::{fs, fs::File, io::Write};
use zip::write::FileOptions;

use crate::read::{BRANCH_FILE, BUS_FILE, CASE_FILE, DCLINE_FILE, GENCOST_FILE, GEN_FILE};
use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

const README_FILE: &str = "README";

pub fn write_zip(
    zip_path: &PathBuf,
    case: &Case,
    bus: &[Bus],
    gen: &[Gen],
    branch: &[Branch],
    gencost: &[GenCost],
    dcline: &[DCLine],
    readme: Option<&str>,
) -> Result<()> {
    let file = File::create(zip_path)?;
    let mut ar = zip::ZipWriter::new(file);

    let options = FileOptions::default().unix_permissions(0o664);

    ar.start_file(CASE_FILE, options)?;
    let mut w = csv::Writer::from_writer(Vec::default());
    w.serialize(case)?;
    w.flush()?;
    ar.write_all(&w.into_inner()?)?;

    if !bus.is_empty() {
        ar.start_file(BUS_FILE, options)?;
        let mut w = csv::Writer::from_writer(Vec::default());
        for r in bus {
            w.serialize(r)?;
        }
        w.flush()?;
        ar.write_all(&w.into_inner()?)?;
    }
    if !gen.is_empty() {
        ar.start_file(GEN_FILE, options)?;
        let mut w = csv::Writer::from_writer(Vec::default());
        for r in gen {
            w.serialize(r)?;
        }
        w.flush()?;
        ar.write_all(&w.into_inner()?)?;
    }
    if !branch.is_empty() {
        ar.start_file(BRANCH_FILE, options)?;
        let mut w = csv::Writer::from_writer(Vec::default());
        for r in branch {
            w.serialize(r)?;
        }
        w.flush()?;
        ar.write_all(&w.into_inner()?)?;
    }
    if !gencost.is_empty() {
        ar.start_file(GENCOST_FILE, options)?;
        let mut w = csv::Writer::from_writer(Vec::default());
        for r in gencost {
            w.write_record(&r.to_string_record())?;
        }
        w.flush()?;
        ar.write_all(&w.into_inner()?)?;
    }
    if !dcline.is_empty() {
        ar.start_file(DCLINE_FILE, options)?;
        let mut w = csv::Writer::from_writer(Vec::default());
        for r in dcline {
            w.serialize(r)?;
        }
        w.flush()?;
        ar.write_all(&w.into_inner()?)?;
    }

    if let Some(readme) = readme {
        ar.start_file(README_FILE, options)?;
        ar.write_all(readme.as_bytes())?;
    }

    Ok(())
}

pub fn write_dir(
    dir_path: &PathBuf,
    case: &Case,
    bus: &[Bus],
    gen: &[Gen],
    branch: &[Branch],
    gencost: &[GenCost],
    dcline: &[DCLine],
    readme: Option<&str>,
) -> Result<()> {
    let case_file = File::create(dir_path.join(CASE_FILE))?;
    let mut w = csv::Writer::from_writer(case_file);
    w.serialize(case)?;
    w.flush()?;

    if !bus.is_empty() {
        let file = File::create(dir_path.join(BUS_FILE))?;
        let mut w = csv::Writer::from_writer(file);
        for r in bus {
            w.serialize(r)?;
        }
        w.flush()?;
    }
    if !gen.is_empty() {
        let file = File::create(dir_path.join(GEN_FILE))?;
        let mut w = csv::Writer::from_writer(file);
        for r in gen {
            w.serialize(r)?;
        }
        w.flush()?;
    }
    if !branch.is_empty() {
        let file = File::create(dir_path.join(BRANCH_FILE))?;
        let mut w = csv::Writer::from_writer(file);
        for r in branch {
            w.serialize(r)?;
        }
        w.flush()?;
    }
    if !gencost.is_empty() {
        let file = File::create(dir_path.join(GENCOST_FILE))?;
        let mut w = csv::Writer::from_writer(file);
        for r in gencost {
            w.write_record(&r.to_string_record())?;
        }
        w.flush()?;
    }
    if !dcline.is_empty() {
        let file = File::create(dir_path.join(DCLINE_FILE))?;
        let mut w = csv::Writer::from_writer(file);
        for r in dcline {
            w.serialize(r)?;
        }
        w.flush()?;
    }
    if let Some(readme) = readme {
        fs::write(dir_path.join(README_FILE), readme)?;
    }

    Ok(())
}
