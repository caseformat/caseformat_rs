use std::path::Path;

use casecsv::{write_zip, Branch, Bus, Case, Gen, REF};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(2);
    }
}

fn run() -> anyhow::Result<()> {
    let (case, bus, gen, branch) = entsoe2()?;

    const _P_GRID: f64 = 0.763;
    const _Q_GRID: f64 = 1.209;

    let zip_path = Path::new("entsoe2.case");
    write_zip(
        &zip_path.to_path_buf(),
        &case,
        &bus,
        &gen,
        &branch,
        &Vec::default(),
        &Vec::default(),
        Some(String::from(
            "2-bus test case from \"Controller Tests in Test Grid Configurations\"
by ENTSO-E System Protection and Dynamics, Nov 2013.",
        )),
        None,
    )?;

    Ok(())
}

// A 2-bus test case from "Controller Tests in Test Grid Configurations"
// by ENTSO-E System Protection and Dynamics, Nov 2013.
fn entsoe2() -> anyhow::Result<(Case, Vec<Bus>, Vec<Gen>, Vec<Branch>)> {
    // System MVA base
    const SB: f64 = 100.0;

    let case = Case::new("entsoe2").base_mva(SB).build()?;

    // Note: Ref bus rated voltage differs from tfmr HV-side
    const VB: f64 = 380.0;
    const VB_HV: f64 = 419.0;
    const VB_LV: f64 = 21.0;

    // GRID //

    const GRID_L: (f64, f64) = (475.0, 76.0);
    const U_NGRID: f64 = 1.05;

    let ngrid = Bus::new(1)
        .bus_type(REF)
        .base_kv(VB)
        .pd(GRID_L.0)
        .qd(GRID_L.1)
        .build()?;
    let grid = Gen::new(1).vg(U_NGRID).mbase(SB).build()?;

    // GEN //

    let ngen = Bus::new(2)
        .pq()
        .base_kv(VB_LV)
        .vm(0.9917)
        .va(9.2327)
        .build()?;

    // PQ synchronous gen at 0.95pu
    const SG: f64 = 500.0;
    const PF: f64 = 0.95;
    let theta = f64::acos(PF);
    let pg = -SG * PF;
    let qg = -SG * f64::sin(theta);

    let gen = Gen::new(ngen.bus_i)
        .pg(pg)
        .qg(qg)
        .vg(U_NGRID)
        .mbase(SB)
        .build()?;

    // T-GEN //

    const SN: f64 = 500.0;
    const UR: f64 = 0.15; // Real-part of short-circuit impedance
    const UK: f64 = 16.0; // Percentage short-circuit impedance

    // The percentage impedance is relative to the nominal voltage (380^2/500).
    // Our transformer model assumes that the impedance is relative to the
    // rated voltage (419^2/500). Recall:
    //
    //                   Vold^2   Snew
    //     Znew = Zold * ------ * ----
    //                   Vnew^2   Sold

    // Convert to system base. Tfmr impedances are relative to HV-side (419kV).
    let c = f64::powi(VB_HV / VB, 2) * (SB / SN);
    let c2 = (f64::powi(VB_HV, 2) * SB) / (f64::powi(VB, 2) * SN); // alt
    assert_eq!(c, c2);
    let r_pu = (UR / 100.0) * c;
    let z_pu = (UK / 100.0) * c;
    let x_pu = f64::sqrt(z_pu.powi(2) - r_pu.powi(2));

    // Off-nominal rated voltage on the HV side of the transformer
    // requires adjustment to the tap ratio of the branch element.
    let tap = VB_HV / VB;

    let t_gen = Branch::new(ngrid.bus_i, ngen.bus_i)
        .r(r_pu)
        .x(x_pu)
        .tap(tap)
        .build()?;

    Ok((case, vec![ngrid, ngen], vec![grid, gen], vec![t_gen]))
}
