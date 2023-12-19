//! Power flow data in CSV format.
//!
//! Based on the [MATPOWER](https://matpower.org) case file format.

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

mod branch;
mod bus;
mod case;
mod dcline;
mod gen;
mod gencost;

mod read;
mod write;

mod mpc;

pub mod validate;

#[cfg(feature = "dataset")]
pub mod dataset;

#[cfg(test)]
mod test;

pub use branch::Branch;
pub use bus::Bus;
pub use bus::{NONE, PQ, PV, REF};
pub use case::Case;
pub use dcline::DCLine;
pub use gen::Gen;
pub use gencost::GenCost;
pub use gencost::{POLYNOMIAL, PW_LINEAR};
pub use mpc::write_mpc;
pub use read::{read_dir, read_zip};
pub use write::{write_dir, write_zip};

#[cfg(feature = "dataset")]
pub mod soa {
    pub use crate::branch::{BranchRef, BranchRefMut, BranchSlice, BranchSliceMut, BranchVec};
    pub use crate::bus::{BusRef, BusRefMut, BusSlice, BusSliceMut, BusVec};
    pub use crate::gen::{GenRef, GenRefMut, GenSlice, GenSliceMut, GenVec};
}

/// Out-of-service status.
pub const OUT_OF_SERVICE: usize = 0;
/// In-service status.
pub const IN_SERVICE: usize = 1;

pub mod builder {
    pub use crate::branch::{BranchBuilder, BranchBuilderError};
    pub use crate::bus::{BusBuilder, BusBuilderError};
    pub use crate::case::{CaseBuilder, CaseBuilderError};
    pub use crate::dcline::{DCLineBuilder, DCLineBuilderError};
    pub use crate::gen::{GenBuilder, GenBuilderError};
    pub use crate::gencost::{GenCostBuilder, GenCostBuilderError};
}

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// CaseCSV Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
fn pycasecsv(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(read_zip, m)?)?;
    m.add_class::<Case>()?;
    m.add_class::<Bus>()?;
    m.add_class::<Gen>()?;
    m.add_class::<Branch>()?;
    Ok(())
}
