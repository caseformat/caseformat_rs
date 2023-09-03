//! Power flow data in CSV format.
//!
//! Based on the [MATPOWER](https://matpower.org) case file format.

mod branch;
mod bus;
mod case;
mod dcline;
mod gen;
mod gencost;

pub mod read;
pub mod validate;

#[cfg(test)]
mod test;

pub use branch::Branch;
pub use bus::{Bus, PQ, PV, REF};
pub use case::Case;
pub use dcline::DCLine;
pub use gen::Gen;
pub use gencost::{GenCost, POLYNOMIAL, PW_LINEAR};

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
