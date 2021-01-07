use std::io::prelude::*;
use std::error::Error;

enum DType {
    DSA,
    DFA,
    DDA,
    DSB,
    DFB,
    DDB,
}

impl DType {
    pub fn from_filename(filename: &str) -> Result<DTypeKind, &'static str> {
        let suffix = match filename.split(".").last() {
            Some(s) => s,
            None => return Err("invalid file name") // TODO improve filename
        };
    }
}


