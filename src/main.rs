use rustdoc_denormalize::Crate;
use rustdoc_denormalize::Stability;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let mut core = Crate::from_str(&fs::read_to_string("assets/core.json")?)?;
    let mut alloc = Crate::from_str(&fs::read_to_string("assets/alloc.json")?)?;
    let mut std = Crate::from_str(&fs::read_to_string("assets/std.json")?)?;

    core.append(&mut alloc);
    core.append(&mut std);

    println!("{}", core.to_table());
    let trait_count = core.traits.len();
    let unstable_trait_count = core
        .traits
        .iter()
        .filter(|t| matches!(t.stability, Stability::Unstable))
        .count();
    let stable_trait_count = trait_count - unstable_trait_count;
    println!("trait count: {trait_count}, stable: {stable_trait_count}, unstable: {unstable_trait_count}");
    Ok(())
}
