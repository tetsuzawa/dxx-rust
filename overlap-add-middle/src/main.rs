extern crate dxx;

use anyhow::Result;
use structopt::StructOpt;
use std::path::PathBuf;

/// overlap-add-middle calculates moving sounds through the angle
/// from the specified move_width and move_velocity.
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Directory of SUBJECT that has SLTF directory.
    /// i.e. `path/to/SUBJECTS/NAME`
    subject: PathBuf,

    /// Sound file that convolve the transfer function.
    /// Typically white noise is used.
    /// i.e. `path/to/wXXs.DSB`
    sound_file: PathBuf,

    /// Moving width [10^-1 deg].
    /// i.e. 080
    move_width: u32,

    /// Moving velocity [10^-1 deg/sec].
    /// i.e. 160
    move_velocity: u32,

    /// Angle placed in the middle [10^-1 deg].
    /// i.e. 450
    angle: u32,

    /// Output path the convolved sound is placed.
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
    Ok(())
    // let data = dxx::read_file("sine.DSB")?;
    // dxx::write_file("sine.DSA", data)
}
