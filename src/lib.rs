use std::io::prelude::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str;
use std::str::FromStr;

enum DType {
    DSA,
    DFA,
    DDA,
    DSB,
    DFB,
    DDB,
}

impl Display for DType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DType::DSA => write!(f, "DSA"),
            DType::DFA => write!(f, "DFA"),
            DType::DDA => write!(f, "DDA"),
            DType::DSB => write!(f, "DSB"),
            DType::DFB => write!(f, "DFB"),
            DType::DDB => write!(f, "DDB"),
        }
    }
}

impl FromStr for DType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DSA" => Ok(DType::DSA),
            "DFA" => Ok(DType::DFA),
            "DDA" => Ok(DType::DDA),
            "DSB" => Ok(DType::DSB),
            "DFB" => Ok(DType::DFB),
            "DDB" => Ok(DType::DDB),
            _ => Err("invalid string")
        }
    }
}

impl DType {
    pub fn from_filename(filename: &str) -> Result<DTypeKind, &'static str> {
        let suffix = match filename.split(".").last() {
            Some(s) => s,
            None => return Err("invalid file name") // TODO improve filename
        };
        DType::from_str(suffix)
    }
}


