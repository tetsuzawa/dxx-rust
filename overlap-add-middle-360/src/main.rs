extern crate dxx;

use anyhow::{Error, Result};
use std::path::PathBuf;
use structopt::StructOpt;

/// overlap-add-middle calculates moving sounds through the angle
/// from the specified move_width and move_velocity.
#[derive(StructOpt, Debug)]
struct Opt {
    /// Directory of SUBJECT that has SLTF directory.
    /// i.e. `path/to/SUBJECTS/NAME`
    subject: PathBuf,

    /// Sound file that convolve the transfer function.
    /// Typically white noise is used.
    /// i.e. `path/to/wXXs.DSB`
    sound_file: PathBuf,

    /// Moving width [10^-1 deg].
    /// i.e. 0080
    move_width: u32,

    /// Moving velocity [10^-1 deg/sec].
    /// i.e. 0160
    move_velocity: u32,

    /// Angle placed in the middle [10^-1 deg].
    /// i.e. 0450
    angle: u32,

    /// Output path where convolved sound is placed.
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    overlap_add(opt)
}

fn overlap_add(opt: Opt) -> Result<()> {
    if !opt.subject.is_dir() {
        return Err(Error::msg("subject is not directory"));
    }
    let subject = match opt.subject.to_str() {
        Some(s) => s,
        None => return Err(Error::msg("subject is empty")),
    };

    if !opt.sound_file.is_file() {
        return Err(Error::msg("sound_file is not a file"));
    }
    let sound_file = match opt.sound_file.to_str() {
        Some(s) => s,
        None => return Err(Error::msg("sound_file is empty")),
    };

    let move_width = opt.move_width;
    let move_velocity = opt.move_velocity;
    let angle = opt.angle;

    if !opt.output.is_dir() {
        return Err(Error::msg("output is not directory"));
    }
    let output = match opt.output.to_str() {
        Some(s) => s,
        None => return Err(Error::msg("output is empty")),
    };

    // サンプリング周波数 [sample/sec]
    let sampling_freq = 48000;
    // 移動時間 [sec]
    let move_time: f64 = move_width as f64 / move_velocity as f64;
    // 移動時間 [sample]
    let move_samples: u32 = (move_time * sampling_freq as f64) as u32;

    // 0.1度動くのに必要なサンプル数
    // [sec]*[sample/sec] / [0.1deg] = [sample/0.1deg]
    let move_samples_per_deg: u32 = move_samples / move_width;

    // 音データの読み込み
    let sound = dxx::read_file(sound_file)?;

    let sltf_name = format!("{}/SLTF/SLTF_{}_{}.DDB", subject, 0, "L");
    let sltf = dxx::read_file(sltf_name.as_str())?;

    for direction in ["c", "cc"].iter() {
        for lr in ["L", "R"].iter() {
            let mut move_out: Vec<f64> = vec![0.; (move_samples + sltf.len() as u32 - 1) as usize];

            // 使用する角度の計算
            let clockwise = *direction == "c";
            let angles = calc_angles(&move_width, &angle, clockwise);

            for (i, conv_angle) in angles.iter().enumerate() {
                let i = i as u32;
                // SLTFの読み込み
                let sltf_name = format!("{}/SLTF/SLTF_{}_{}.DDB", subject, conv_angle, lr);
                let sltf = dxx::read_file(sltf_name.as_str())?;

                // 音データと伝達関数の畳込み
                let cut_sound = sound[(move_samples_per_deg * i) as usize
                    ..(move_samples_per_deg * (i + 1)) as usize]
                    .to_vec();
                let sound_sltf = linear_conv(cut_sound, sltf);
                // Overlap-Add
                for (j, v) in sound_sltf.iter().enumerate() {
                    let j = j as u32;
                    move_out[(move_samples_per_deg * i + j) as usize] += v
                }
            }

            let output_name = format!(
                "{}/move_judge_w{:>04}_mt{:>04}_{}_{:>04}_{}.DDB",
                output, move_width, move_velocity, direction, angle, lr
            );
            let output_len = move_out.len();
            dxx::write_file(output_name.as_str(), move_out)?;
            eprintln!("{}, length={}", output_name, output_len);
            eprintln!("angles={:?}", angles)
        }
    }
    Ok(())
}

fn calc_angles(move_width: &u32, angle: &u32, clockwise: bool) -> Vec<i32> {
    let move_width = *move_width as i32;
    let angle = *angle as i32;
    let start_angle = if clockwise {
        angle - move_width / 2
    } else {
        angle + move_width / 2 - 1
    };

    let start_angle = if start_angle < 0 {
        start_angle + 360
    } else {
        start_angle
    } % 360;

    let mut angles = Vec::with_capacity(move_width as usize);
    for i in 0..move_width {
        let mut data_angle = i % (move_width * 2);
        if data_angle > move_width {
            data_angle = move_width * 2 - data_angle
        }
        if !clockwise {
            data_angle = -1 * data_angle
        }
        if data_angle < 0 {
            data_angle += 360
        }
        angles.push((start_angle + data_angle) % 360);
    }
    angles
}

#[cfg(test)]
mod tests {
    use crate::calc_angles;

    #[test]
    fn calc_angles_0_c() {
        let angles = calc_angles(&10, &0, true);
        assert_eq!(angles, vec![355, 356, 357, 358, 359, 0, 1, 2, 3, 4])
    }
    #[test]
    fn calc_angles_0_cc() {
        let angles = calc_angles(&10, &0, false);
        assert_eq!(angles, vec![4, 3, 2, 1, 0, 359, 358, 357, 356, 355])
    }

    #[test]
    fn calc_angles_5_c() {
        let angles = calc_angles(&20, &5, true);
        assert_eq!(
            angles,
            vec![355, 356, 357, 358, 359, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        )
    }
    #[test]
    fn calc_angles_5_cc() {
        let angles = calc_angles(&20, &5, false);
        assert_eq!(
            angles,
            vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 359, 358, 357, 356, 355, 354, 353, 352, 351, 350]
        )
    }
}

fn linear_conv(x: Vec<f64>, y: Vec<f64>) -> Vec<f64> {
    let conv_len = x.len() + y.len() - 1;
    let mut ret: Vec<f64> = vec![0.; conv_len];
    for p in 0..x.len() {
        for n in p..y.len() + p {
            ret[n] += x[p] * y[n - p];
        }
    }
    ret
}
