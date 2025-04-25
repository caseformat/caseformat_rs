pub mod entsoe2;

use crate::{read_tar, Branch, Bus, Case, DCLine, Gen, GenCost};

#[cfg(feature = "gs4")]
pub fn gs4() -> (
    Case,
    Vec<Bus>,
    Vec<Gen>,
    Vec<Branch>,
    Vec<GenCost>,
    Vec<DCLine>,
    Option<String>,
    Option<String>,
) {
    let bytes = include_bytes!("../../casedata/gs4-v0.1.0.case");
    read_tar(bytes.as_slice()).unwrap()
}

#[cfg(feature = "ieee14")]
pub fn ieee14() -> (
    Case,
    Vec<Bus>,
    Vec<Gen>,
    Vec<Branch>,
    Vec<GenCost>,
    Vec<DCLine>,
    Option<String>,
    Option<String>,
) {
    read_tar(include_bytes!("../../casedata/ieee14-v0.1.0.case").as_slice()).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::testcase::gs4;

    #[cfg(feature = "gs4")]
    #[test]
    fn test_gs4() {
        let case = gs4();
        assert_eq!(case.1.len(), 4);
    }
}
