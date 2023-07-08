use rustdoc_denormalize::Crate;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let mut core = Crate::from_str(&fs::read_to_string("assets/core.json")?)?;
    let mut alloc = Crate::from_str(&fs::read_to_string("assets/alloc.json")?)?;
    let mut std = Crate::from_str(&fs::read_to_string("assets/std.json")?)?;

    core.append(&mut alloc);
    core.append(&mut std);

    println!("{}", core.to_table());
    Ok(())
}
