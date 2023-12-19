use crate::read::read_zip;
use crate::write::write_zip;
use crate::{Branch, Bus, Case, DCLine, Gen, GenCost};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ReadResponse {
    case: Case,
    bus: Vec<Bus>,
    gen: Vec<Gen>,
    branch: Vec<Branch>,
    gencost: Vec<GenCost>,
    dcline: Vec<DCLine>,
    readme: Option<String>,
    license: Option<String>,
}

#[wasm_bindgen]
pub fn read_case_bytes(data: Vec<u8>) -> Result<ReadResponse, String> {
    let cursor = Cursor::new(data.as_slice());
    let (case, bus, gen, branch, gencost, dcline, readme, license) = read_zip(cursor).unwrap();
    Ok(ReadResponse {
        case,
        bus,
        gen,
        branch,
        gencost,
        dcline,
        readme,
        license,
    })
}

#[wasm_bindgen]
pub fn write_case_bytes(
    data: ReadResponse,
    // case: Case,
    // bus: Vec<Bus>,
    // gen: Vec<Gen>,
    // branch: Vec<Branch>,
    // gencost: Vec<GenCost>,
    // dcline: Vec<DCLine>,
    // readme: Option<String>,
    // license: Option<String>,
) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(vec![]);
    let cursor = write_zip(
        cursor,
        &data.case,
        &data.bus,
        &data.gen,
        &data.branch,
        &data.gencost,
        &data.dcline,
        data.readme,
        data.license,
    )
    .unwrap();
    Ok(cursor.into_inner())
    // Ok(vec![])
}
