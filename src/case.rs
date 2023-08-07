use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Case {
    /// Case name.
    #[serde(rename = "CASENAME")]
    pub name: String,

    /// Case format version.
    pub version: String,

    /// System MVA base.
    #[validate(range(min = 1))]
    pub base_mva: f64,

    /// Total system cost (US dollars).
    pub f: Option<f64>,
}
