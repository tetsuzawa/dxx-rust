use std::io::prelude::*;
use std::io::{Cursor, BufReader};
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str;
use std::str::FromStr;
use std::error::Error;
use std::fs;
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};

const TEXT_BIN_FILE_SIZE_MEAN_RATE: &'static usize = &13;

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

pub fn read_file(filename: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut f = File::open(filename)?;
    let file_size = f.metadata()?.len() as usize;
    let dtype = DType::from_filename(filename)?;
    match dtype {
        DType::DSA |
        DType::DFA |
        DType::DDA => read_DXA(src, file_size),

        DType::DSB => read_DSB(src, file_size),
        DType::DFB => read_DFB(src, file_size),
        DType::DDB => read_DDB(src, file_size),
    }
}

fn read_DXA<T: Read>(src: T, size: usize) -> Result<Vec<f64>, Box<Error>> {
    let mut ret: Vec<f64> = Vec::with_capacity(size / *TEXT_BIN_FILE_SIZE_MEAN_RATE);
    for result in BufReader::new(src).lines() {
        let l = result?;
        let buf: f64 = From::from(l);
        ret.push(buf);
    }
    Ok(ret)
}

fn read_DSB<T: Read>(src: T, size: usize) -> Result<Vec<f64>, Box<Error>> {
    let byte_width = DType::DSB.byte_width();
    let mut buf: Vec<i16> = vec![0; size / byte_width];
    let mut rdr = Cursor::new(src);
    rdr.read_i16_into::<LittleEndian>(&mut buf);
    buf.iter().map(|x| f64::from(*x)).collect()
}

fn read_DFB<T: Read>(src: T, size: usize) -> Result<Vec<f64>, Box<Error>> {
    let byte_width = DType::DFB.byte_width();
    let mut buf: Vec<f32> = vec![0.; size / byte_width];
    let mut rdr = Cursor::new(src);
    rdr.read_f32_into::<LittleEndian>(&mut buf);
    buf.iter().map(|x| f64::from(*x)).collect()
}

fn read_DDB<T: Read>(src: T, size: usize) -> Result<Vec<f64>, Box<Error>> {
    let byte_width = DType::DDB.byte_width();
    let mut buf: Vec<f64> = vec![0.; size / byte_width];
    let mut rdr = Cursor::new(src);
    rdr.read_f64_into::<LittleEndian>(&mut buf);
    buf.iter().map(|x| f64::from(*x)).collect()
}

fn i16s_to_f64s(src: Vec<i16>) -> Vec<f64> {
    src.iter().map(|x| f64::from(*x)).collect()
}

