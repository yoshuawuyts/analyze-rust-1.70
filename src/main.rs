use rustdoc_denormalize::Crate;
use rustdoc_denormalize::Item;
use rustdoc_denormalize::Stability;
use std::fs;
use std::io;
use structopt::StructOpt;

mod analyze;

#[derive(structopt::StructOpt)]
enum Opts {
    /// Output a table
    Table,
    /// Output a CSV
    Csv,
    /// Generate an analysis
    Stats,
}

fn main() -> io::Result<()> {
    let mut krate = Crate::from_str(&fs::read_to_string("assets/core.json")?)?;
    let mut alloc = Crate::from_str(&fs::read_to_string("assets/alloc.json")?)?;
    let mut std = Crate::from_str(&fs::read_to_string("assets/std.json")?)?;

    krate.append(&mut alloc);
    krate.append(&mut std);
    let table = krate.to_table();

    match Opts::from_args() {
        Opts::Table => print_table(table),
        Opts::Csv => print_csv(krate),
        Opts::Stats => print_stats(krate),
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
    krate
        .impls
        .into_iter()
        .for_each(|t| writer.serialize(t).unwrap());
    Ok(())
}

fn print_table(table: cli_table::TableStruct) -> Result<(), io::Error> {
    println!("{}", table.display()?);
    Ok(())
}

fn print_stats(krate: Crate) -> Result<(), io::Error> {
    let trait_stats = Stats::from_iter(krate.traits.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {trait_stats:?}", "traits");

    let fn_stats = Stats::from_iter(
        krate
            .functions
            .iter()
            .map(|t| (t.stability, t.has_generics)),
    );
    println!("{: <10} {fn_stats:?}", "functions");

    let struct_stats =
        Stats::from_iter(krate.structs.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {struct_stats:?}", "structs");

    let enum_stats = Stats::from_iter(krate.enums.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {enum_stats:?}", "enums");

    let impl_stats = Stats::from_iter(krate.impls.iter().map(|t| (t.stability, t.has_generics)));
    println!("{: <10} {impl_stats:?}", "impls");

    let adt_stats = struct_stats.clone() + enum_stats.clone();
    println!("{: <10} {adt_stats:?}", "ADTs");

    println!("\n------\n");

    println!(
        "traits per ADT: {:.1}",
        impl_stats.stable as f32 / adt_stats.stable as f32
    );

    count_const_stats("functions", &krate.functions, &fn_stats);
    count_const_stats("structs", &krate.structs, &struct_stats);
    count_const_stats("traits", &krate.traits, &trait_stats);
    count_const_stats("enums", &krate.enums, &enum_stats);
    count_const_stats("impls", &krate.impls, &impl_stats);

    println!("\n------\n");

    count_async_stats("functions", &krate.functions, &fn_stats, |item| {
        !item.has_generics
    });
    count_async_stats("structs", &krate.structs, &struct_stats, |_item| false);
    count_async_stats("traits", &krate.traits, &trait_stats, |_item| false);
    count_async_stats("enums", &krate.enums, &enum_stats, |_item| false);
    count_async_stats("impls", &krate.impls, &impl_stats, |_item| false);

    println!("\n------\n");
    Ok(())
}

fn count_const_stats(name: &str, items: &[Item], stats: &Stats) {
    let (const_count, excluded) = analyze::count_const_items(items);
    count_stats(name, "const", stats, excluded, const_count);
}

fn count_async_stats(
    name: &str,
    items: &[Item],
    stats: &Stats,
    should_exclude: impl FnMut(&&Item) -> bool,
) {
    let (async_count, excluded) = analyze::count_async_items(items, should_exclude);
    count_stats(name, "async", stats, excluded, async_count);
}

fn count_stats(name: &str, kind: &str, stats: &Stats, excluded: usize, const_count: usize) {
    let const_maximum = stats.stable - excluded;
    let const_max_ratio = (const_maximum as f64 / stats.stable as f64) * 100.0;
    let const_ratio = (const_count as f64 / const_maximum as f64) * 100.0;
    println!("potential {kind} {name}: {const_maximum} ({const_max_ratio:.1}%)");
    println!("currently {kind} {name}: {const_count} ({const_ratio:.1}%)",);
}

#[derive(Clone)]
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

impl std::ops::Add for Stats {
    type Output = Stats;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.total += rhs.total;
        self.stable += rhs.stable;
        self.unstable += rhs.unstable;
        self.generics += rhs.generics;
        self
    }
}
