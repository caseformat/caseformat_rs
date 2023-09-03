use std::path::Path;
use validator::Validate;

use crate::read::{read_dir, read_zip};

#[test]
fn test_read_dir() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let case9_dir = manifest_path.join("casedata").join("case9");

    let (case, buses, gen, branch, gencost, dcline) = read_dir(&case9_dir).unwrap();

    assert!(case.validate().is_ok());
    assert_eq!(case.name, "case9");
    assert_eq!(case.version, "2");
    assert_eq!(case.base_mva, 100.0);
    assert!(case.f.is_none());

    assert_eq!(buses.len(), 9);
    assert!(buses.iter().all(|bus| bus.validate().is_ok()));
    assert!(!buses.iter().any(|bus| bus.is_opf()));
    assert_eq!(buses.iter().filter(|bus| bus.is_ref()).count(), 1);

    assert!(gen.is_some());
    if let Some(gen) = gen {
        assert_eq!(gen.len(), 3);
        assert!(gen.iter().all(|g| g.validate().is_ok()));
    }

    assert!(branch.is_some());
    if let Some(branch) = branch {
        assert_eq!(branch.len(), 9);
        assert!(branch.iter().all(|br| br.validate().is_ok()));
    }

    assert!(gencost.is_some());
    if let Some(gencost) = gencost {
        assert_eq!(gencost.len(), 3);
        assert!(gencost.iter().all(|c| c.validate().is_ok()));
    }

    assert!(dcline.is_none());
}

#[test]
fn test_read_zip() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let ieee14_zip = manifest_path.join("casedata").join("ieee14.case");

    let (case, buses, gen, branch, gencost, dcline) = read_zip(&ieee14_zip).unwrap();

    assert!(case.validate().is_ok());
    assert_eq!(case.name, "ieee14");
    assert_eq!(case.version, "2");
    assert_eq!(case.base_mva, 100.0);
    assert!(case.f.is_none());

    assert_eq!(buses.len(), 14);
    assert!(buses.iter().all(|bus| bus.validate().is_ok()));
    assert!(!buses.iter().any(|bus| bus.is_opf()));
    assert_eq!(buses.iter().filter(|bus| bus.is_ref()).count(), 1);

    assert!(gen.is_some());
    if let Some(gen) = gen {
        assert_eq!(gen.len(), 5);
        assert!(gen.iter().all(|g| g.validate().is_ok()));
    }

    assert!(branch.is_some());
    if let Some(branch) = branch {
        assert_eq!(branch.len(), 20);
        assert!(branch.iter().all(|br| br.validate().is_ok()));
    }

    assert!(gencost.is_some());
    if let Some(gencost) = gencost {
        assert_eq!(gencost.len(), 5);
        assert!(gencost.iter().all(|c| c.validate().is_ok()));
    }

    assert!(dcline.is_none());
}
