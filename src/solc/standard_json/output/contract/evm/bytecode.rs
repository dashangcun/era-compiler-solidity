//!
//! The `solc --standard-json` output contract EVM bytecode.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` output contract EVM bytecode.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bytecode {
    /// The bytecode object.
    pub object: String,
}

impl Bytecode {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(object: String) -> Self {
        Self { object }
    }
}
