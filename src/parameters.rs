use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{
    Deserialize,
    Serialize,
};

/// file format for a parameters file. so this only maps influence names to
/// values. this will be applied to `parameters` map in
/// [[`crate::pattern::Template`]], which will also check the constraints.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Parameters {
    #[serde(flatten)]
    pub parameters: BTreeMap<String, Decimal>,
}
