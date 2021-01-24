extern crate dxx;

use anyhow::Result;

fn main() -> Result<()>{
    let data = dxx::read_file("sine.DSB")?;
    dxx::write_file("sine.DSA", data)
}
