use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str;
use std::str::FromStr;
use std::error::Error;
use std::fs;
use std::fs::File;
// use byteorder::{LittleEndian};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

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
        DType::DDA => read_dxa(&mut f, file_size),

        DType::DSB => read_dsb(&mut f, file_size),
        DType::DFB => read_dfb(&mut f, file_size),
        DType::DDB => read_ddb(&mut f, file_size),
    }
}


fn read_dxa<T: Read>(src: &mut T, size: usize) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut ret: Vec<f64> = Vec::with_capacity(size / *TEXT_BIN_FILE_SIZE_MEAN_RATE);
    for result in BufReader::new(src).lines() {
        let line = result?;
        let buf: f64 = line.parse::<f64>()?;
        ret.push(buf);
    }
    Ok(ret)
}

fn read_dsb<T: Read>(src: &mut T, size: usize) -> Result<Vec<f64>, Box<dyn Error>> {
    let byte_width = DType::DSB.byte_width();
    let mut buf: Vec<i16> = vec![0; size / byte_width as usize];
    let mut reader = BufReader::new(src);
    reader.read_i16_into::<LittleEndian>(&mut buf)?;
    Ok(buf.iter().map(|x| f64::from(*x)).collect())
}

fn read_dfb<T: Read>(src: &mut T, size: usize) -> Result<Vec<f64>, Box<dyn Error>> {
    let byte_width = DType::DFB.byte_width();
    let mut buf: Vec<f32> = vec![0.; size / byte_width as usize];
    let mut reader = BufReader::new(src);
    reader.read_f32_into::<LittleEndian>(&mut buf)?;
    Ok(buf.iter().map(|x| f64::from(*x)).collect())
}

fn read_ddb<T: Read>(src: &mut T, size: usize) -> Result<Vec<f64>, Box<dyn Error>> {
    let byte_width = DType::DDB.byte_width();
    let mut buf: Vec<f64> = vec![0.; size / byte_width as usize];
    let mut reader = BufReader::new(src);
    reader.read_f64_into::<LittleEndian>(&mut buf)?;
    Ok(buf.iter().map(|x| f64::from(*x)).collect())
}
//
// pub fn write_file(filename: &str, src: Vec<f64>) -> Result<(), Box<dyn Error>> {
//     let mut f = File::create(filename)?;
//     let file_size = f.metadata()?.len() as usize;
//     let dtype = DType::from_filename(filename)?;
//
//     match dtype {
//         DType::DSA |
//         DType::DFA |
//         DType::DDA => read_dxa(&mut f, file_size),
//
//         DType::DSB => read_dsb(&mut f, file_size),
//         DType::DFB => read_dfb(&mut f, file_size),
//         DType::DDB => read_ddb(&mut f, file_size),
//     }
// }
//
// fn write_dxa<T: Write>(dst: T, src: Vec<f64>, dtype: DType) -> Result<(), Box<dyn Error>> {
//     let mut writer = BufWriter::new(dst);
//     src.iter().map(|x| writer.write_fmt()
//     for result in BufWriter::new(dst).lines() {
//         let line = result?;
//         let buf: f64 = line.parse::<f64>()?;
//         ret.push(buf);
//     }
//     Ok(ret)
// }


fn f64s_to_i16s(src: &Vec<f64>) -> Vec<i16> {
    let amp = 1 << (16 - 1) - 1;
    let abs_src: Vec<f64> = src.iter().map(|x| x.clone().abs()).collect();
    let max = max_f64s(&abs_src);
    src.iter().map(|x| (x / max * amp as f64) as i16).collect()
}

fn max_f64s(src: &Vec<f64>) -> f64 {
    src.iter().fold(0.0 / 0.0, |m, v| v.max(m))
}

fn min_f64s(src: &Vec<f64>) -> f64 {
    src.iter().fold(0.0 / 0.0, |m, v| v.min(m))
}

fn max_f32s(src: &Vec<f32>) -> f32 {
    src.iter().fold(0.0 / 0.0, |m, v| v.max(m))
}

fn min_f32s(src: &Vec<f32>) -> f32 {
    src.iter().fold(0.0 / 0.0, |m, v| v.min(m))
}

#[cfg(test)]
mod tests {
    use crate::f64s_to_i16s;

    #[test]
    fn test_f64s_to_i16s() {
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        assert_eq!(f64s_to_i16s(&src), vec![16384, -6553, 13107, -9830]);
    }
}
