use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str;
use std::str::FromStr;
use std::error::Error;
use std::fs;
use std::fs::File;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

const TEXT_BIN_FILE_SIZE_MEAN_RATE: &'static usize = &13;
const DSX_AMP: i16 = i16::max_value();
const DFX_AMP: f32 = 10000.;
const DDX_AMP: f64 = 10000.;

pub enum DType {
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

pub fn write_file(filename: &str, src: Vec<f64>) -> Result<(), Box<dyn Error>> {
    let mut f = File::create(filename)?;
    let dtype = DType::from_filename(filename)?;

    match dtype {
        DType::DSA => write_dxa(&mut f, f64s_to_i16s(&src, DSX_AMP)),
        DType::DFA => write_dxa(&mut f, f64s_to_f32s(&src, DFX_AMP)),
        DType::DDA => write_dxa(&mut f, normalize_f64s(src, DDX_AMP)),

        DType::DSB => write_dsb(&mut f, f64s_to_i16s(&src, DSX_AMP)),
        DType::DFB => write_dfb(&mut f, f64s_to_f32s(&src, DFX_AMP)),
        DType::DDB => write_ddb(&mut f, normalize_f64s(src, DDX_AMP)),
    }
}

fn write_dxa<T: Write, U: std::fmt::Display>(dst: T, src: Vec<U>) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(dst);
    for x in src {
        writeln!(writer, "{}", x)?;
    }
    Ok(())
}

fn write_dsb<T: Write>(dst: T, src: Vec<i16>) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(dst);
    for x in src {
        writer.write_i16::<LittleEndian>(x)?;
    }
    Ok(())
}

fn write_dfb<T: Write>(dst: T, src: Vec<f32>) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(dst);
    for x in src {
        writer.write_f32::<LittleEndian>(x)?;
    }
    Ok(())
}

fn write_ddb<T: Write>(dst: T, src: Vec<f64>) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(dst);
    for x in src {
        writer.write_f64::<LittleEndian>(x)?;
    }
    Ok(())
}

fn normalize_f64s(src: Vec<f64>, amp: f64) -> Vec<f64> {
    let abs_src: Vec<f64> = src.iter().map(|x| x.clone().abs()).collect();
    let max = max_f64s(&abs_src);
    src.iter().map(|x| (x / max * amp)).collect()
}

fn f64s_to_i16s(src: &Vec<f64>, amp: i16) -> Vec<i16> {
    let abs_src: Vec<f64> = src.iter().map(|x| x.clone().abs()).collect();
    let max = max_f64s(&abs_src);
    src.iter().map(|x| (x / max * amp as f64) as i16).collect()
}

fn f64s_to_f32s(src: &Vec<f64>, amp: f32) -> Vec<f32> {
    let abs_src: Vec<f64> = src.iter().map(|x| x.clone().abs()).collect();
    let max = max_f64s(&abs_src);
    src.iter().map(|x| (x / max * amp as f64) as f32).collect()
}

fn max_f64s(src: &Vec<f64>) -> f64 {
    src.iter().fold(0.0 / 0.0, |m, v| v.max(m))
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_f64s_to_i16s() {
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        assert_eq!(f64s_to_i16s(&src, DSX_AMP), vec![32767, -13106, 26213, -19660]);
    }

    #[test]
    fn test_write_file() {
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DSA", src).unwrap();
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DFA", src).unwrap();
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DDA", src).unwrap();
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DSB", src).unwrap();
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DFB", src).unwrap();
        let src: Vec<f64> = vec![5., -2., 4., -3.];
        write_file("a.DDB", src).unwrap();
    }

    #[test]
    fn test_convert() {
        let data = read_file("sine.DSB").unwrap();
        write_file("sine.DSA", data).unwrap();
        let data = read_file("sine.DSB").unwrap();
        write_file("sine.DFA", data).unwrap();
        let data = read_file("sine.DSB").unwrap();
        write_file("sine.DFB", data).unwrap();
        let data = read_file("sine.DSB").unwrap();
        write_file("sine.DDA", data).unwrap();
        let data = read_file("sine.DSB").unwrap();
        write_file("sine.DDB", data).unwrap();
        let data = read_file("sine.DSA").unwrap();
        write_file("sine1.DSB", data).unwrap();
    }
}
