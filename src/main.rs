use rustdoc_denormalize::Crate;
use rustdoc_denormalize::Stability;
use std::fs;
use std::io;
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
enum Opts {
    /// Output a table
    Table,
    /// Output a CSV
    Csv,
}

fn main() -> io::Result<()> {
    let mut krate = Crate::from_str(&fs::read_to_string("assets/core.json")?)?;
    let mut alloc = Crate::from_str(&fs::read_to_string("assets/alloc.json")?)?;
    let mut std = Crate::from_str(&fs::read_to_string("assets/std.json")?)?;

    krate.append(&mut alloc);
    krate.append(&mut std);
    let table = krate.to_table();

    match Opts::from_args() {
        Opts::Table => print_table(table, krate),
        Opts::Csv => print_csv(krate),
    }
}

fn print_csv(krate: Crate) -> Result<(), io::Error> {
    let mut writer = csv::Writer::from_writer(io::stdout());
    krate
        .structs
        .into_iter()
        .for_each(|t| writer.serialize(t).unwrap());
    krate
        .enums
        .into_iter()
        .for_each(|t| writer.serialize(t).unwrap());
    krate
        .traits
        .into_iter()
        .for_each(|t| writer.serialize(t).unwrap());
    krate
        .functions
        .into_iter()
        .for_each(|t| writer.serialize(t).unwrap());
    Ok(())
}

fn print_table(table: cli_table::TableStruct, core: Crate) -> Result<(), io::Error> {
    println!("{}", table.display()?);

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
