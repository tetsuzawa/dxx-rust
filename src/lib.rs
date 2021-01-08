use std::io::prelude::*;
use std::io::Cursor;
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str;
use std::str::FromStr;
use std::error::Error;
use std::fs;
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};

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
    pub fn from_filename(filename: &str) -> Result<DType, &'static str> {
        let suffix = match filename.split(".").last() {
            Some(s) => s,
            None => return Err("invalid file name") // TODO improve filename
        };
        DType::from_str(suffix)
    }

    pub fn byte_width(&self) -> u32 {
        match *self {
            DType::DSA | DType::DSB => 2,
            DType::DFA | DType::DFB => 4,
            DType::DDA | DType::DDB => 8,
        }
    }

    pub fn bits_width(&self) -> u32 {
        match *self {
            DType::DSA | DType::DSB => 16,
            DType::DFA | DType::DFB => 32,
            DType::DDA | DType::DDB => 64,
        }
    }
}

pub fn len_file(filename: &str) -> Result<u64, Box<dyn Error>> {
    let meta = fs::metadata(filename)?;
    Ok(meta.len())
}

pub fn read(filename: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let dtype = DType::from_filename(filename)?;

    let mut f = File::open(filename)?;
    let file_size = f.metadata()?.len();
    let mut raw_data = Vec::with_capacity(file_size as usize);
    f.read_to_end(&mut raw_data)?;

    let samples = &file_size / dtype.byte_width() as u64;
    let mut data = Vec::with_capacity(samples as usize);

    let mut buf = Vec::with_capacity(dtype.byte_width() as usize);

    let mut rdr = Cursor::new(raw_data);
    match dtype {
        DType::DSB => {rdr.read_i16_into::<LittleEndian>(&mut data)},
        DType::DFB => rdr.read_f32_into::<LittleEndian>(&mut data),
        DType::DDB => rdr.read_f64_into::<LittleEndian>(&mut data),
        _ => Ok(()),//TODO
    };
}

