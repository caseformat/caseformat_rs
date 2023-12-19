use anyhow::Result;
use std::io::Write;

use crate::write::*;
use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};

#[macro_export]
macro_rules! write_row {
    ($w:expr, $rec:expr) => {{
        write!(
            $w,
            "\t{};\n",
            $rec.iter().map(|r| r).collect::<Vec<&str>>().join("\t")
        )?;
    }};
}

pub fn write_mpc<W: Write>(
    mut w: W,
    case: &Case,
    bus: &[Bus],
    gen: &[Gen],
    branch: &[Branch],
    gencost: &[GenCost],
    dcline: &[DCLine],
) -> Result<W> {
    write!(w, "function mpc = {}\n", case.name)?;
    write!(w, "\nmpc.version = '{}';\n", case.version)?;

    if !bus.is_empty() {
        let is_opf = bus.iter().any(|b| b.is_opf());
        let header = if !is_opf {
            BUS_HEADER.to_vec()
        } else {
            BUS_HEADER_OPF.to_vec()
        };
        write!(w, "\n%\t{}\n", header.join("\t"))?;
        write!(w, "mpc.bus = [\n")?;
        for b in bus {
            write_row!(w, b.to_string_record(is_opf));
        }
        write!(w, "];\n")?;
    }

    if !gen.is_empty() {
        let is_version_1 = gen.iter().any(|g| g.is_version_1());
        let is_opf = gen.iter().any(|g| g.is_opf());
        let header = if !is_opf && is_version_1 {
            GEN_HEADER.to_vec()
        } else if !is_opf {
            GEN_HEADER_2.to_vec()
        } else {
            GEN_HEADER_OPF.to_vec()
        };
        write!(w, "\n%\t{}\n", header.join("\t"))?;
        write!(w, "mpc.gen = [\n")?;
        for g in gen {
            write_row!(w, g.to_string_record(is_version_1, is_opf));
        }
        write!(w, "];\n")?;
    }

    if !branch.is_empty() {
        let is_pf = branch.iter().any(|br| br.is_pf());
        let is_opf = branch.iter().any(|br| br.is_opf());
        let header = if !is_opf && !is_pf {
            BRANCH_HEADER.to_vec()
        } else if !is_opf {
            BRANCH_HEADER_PF.to_vec()
        } else {
            BRANCH_HEADER_OPF.to_vec()
        };
        write!(w, "\n%\t{}\n", header.join("\t"))?;
        write!(w, "mpc.branch = [\n")?;
        for br in branch {
            write_row!(w, br.to_string_record(is_pf, is_opf));
        }
        write!(w, "];\n")?;
    }

    if !gencost.is_empty() {
        write!(w, "\n%\t{}\n", GENCOST_HEADER.join("\t"))?;
        write!(w, "mpc.gencost = [\n")?;
        for c in gencost {
            write_row!(w, c.to_string_record());
        }
        write!(w, "];\n")?;
    }

    if !dcline.is_empty() {
        let is_opf = dcline.iter().any(|br| br.is_opf());
        let header = if !is_opf {
            DCLINE_HEADER.to_vec()
        } else {
            DCLINE_HEADER_OPF.to_vec()
        };
        write!(w, "\n%\t{}\n", header.join("\t"))?;
        write!(w, "mpc.dcline = [\n")?;
        for br in dcline {
            write_row!(w, br.to_string_record(is_opf));
        }
        write!(w, "];\n")?;
    }

    Ok(w)
}
