use anyhow::Result;
use caseformat::dataset::Dataset;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Power flow simulation and optimization.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Input file or directory
    #[arg(required = true)]
    input: PathBuf,

    /// Output file or directory
    #[arg(short, long)]
    output: PathBuf,

    /// Pretty print JSON.
    #[arg(long, default_value_t = false)]
    pub pretty: bool,
}

fn main() {
    let cli = Cli::parse();

    match execute(&cli) {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }
    }
}

fn execute(cli: &Cli) -> Result<()> {
    let case_path = &cli.input;

    let is_case = match case_path.extension() {
        None => false,
        Some(os_str) => match os_str.to_str() {
            Some("case") | Some("zip") => true,
            _ => false,
        },
    };

    let (case, bus, gen, branch, gencost, dcline, readme, license) = if is_case {
        let file = File::open(case_path).expect("Unable to open input file");
        let reader = BufReader::new(file);
        caseformat::read_zip(reader)?
    } else {
        caseformat::read_dir(case_path)?
    };

    match cli.output.extension() {
        None => {
            caseformat::write_dir(
                &cli.output,
                &case,
                &bus,
                &gen,
                &branch,
                &gencost,
                &dcline,
                readme,
                license,
            )?;
        }
        Some(os_str) => match os_str.to_str() {
            Some("json") => {
                let file = File::create(&cli.output)?;
                let dataset = Dataset::new(&case, &bus, &gen, &branch);
                if cli.pretty {
                    serde_json::to_writer_pretty(file, &dataset)?;
                } else {
                    serde_json::to_writer(file, &dataset)?;
                }
            }
            Some("m") => {
                let file = File::create(&cli.output)?;
                caseformat::write_mpc(file, &case, &bus, &gen, &branch, &gencost, &dcline)?;
            }
            Some("case") | Some("zip") => {
                let file = File::create(&cli.output)?;
                caseformat::write_zip(
                    file, &case, &bus, &gen, &branch, &gencost, &dcline, readme, license,
                )?;
            }
            _ => {}
        },
    }

    Ok(())
}

// #[derive(serde::Serialize)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub struct Dataset {
//     pub casename: String,
//     pub base_mva: f64,
//
//     pub bus: Vec<caseformat::Bus>,
//     pub gen: Vec<caseformat::Gen>,
//     pub branch: Vec<caseformat::Branch>,
// }
