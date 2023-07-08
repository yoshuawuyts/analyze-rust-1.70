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

    let stats = Stats::from_iter(core.traits.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {stats:?}", "traits");

    let stats = Stats::from_iter(core.functions.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {stats:?}", "functions");

    let stats = Stats::from_iter(core.structs.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {stats:?}", "structs");

    let stats = Stats::from_iter(core.enums.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {stats:?}", "enums");
    Ok(())
}

struct Stats {
    total: usize,
    stable: usize,
    unstable: usize,
    generics: usize,
}

impl std::fmt::Debug for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total: {: >4}, stable: {: >4}, unstable: {: >4}, generics: {: >4}",
            &self.total, &self.stable, &self.unstable, &self.generics
        )
    }
}

impl Stats {
    fn from_iter(iter: impl Iterator<Item = (Stability, bool)>) -> Self {
        let mut this = Self {
            total: 0,
            stable: 0,
            unstable: 0,
            generics: 0,
        };
        for (stability, has_generics) in iter {
            this.total += 1;
            match stability {
                Stability::Stable => this.stable += 1,
                Stability::Unstable => this.unstable += 1,
            }
            if has_generics {
                this.generics += 1;
            }
        }
        this
    }
}
