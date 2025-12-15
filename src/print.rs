use crate::binary::{Binary, Segment};
use crate::decoder::{Count, Detail, Feature, Item};
use std::env;
use std::error::Error;
use std::io::{self, Result as IoResult, StdoutLock, Write};

pub fn help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        PKG = env!("CARGO_PKG_NAME"),
        VER = env!("CARGO_PKG_VERSION"),
        BIN_NAME = (|| bin.as_ref()?.file_name()?.to_str())().unwrap_or(env!("CARGO_BIN_NAME")),
    );
}

pub fn error(error: &dyn Error) {
    eprintln!("Error: {error}");
}

pub fn file_path(path: &str) {
    println!("Reading '{path}'...");
}

pub fn binary(binary: &Binary) {
    println!("Format: {:?}", binary.format());
    println!("Architecture: {:?}", binary.architecture());
}

pub fn segment(segment: &Segment) {
    println!(
        "    {} => 0x{:x}, {} bytes",
        segment.name().unwrap_or_default(),
        segment.offset(),
        segment.size(),
    );
}

pub fn header(text: &str) {
    println!("{text}");
    println!("{:-<1$}", "", text.len());
}

pub fn cpuid(features: &[Count<Feature>]) {
    if features.iter().any(Count::is_cpuid) {
        println!("Warning: CPUID usage detected, features could switch in runtime.");
    }
}

pub fn stats_note() {
    println!("Note: instructions that belong to multiple feature sets make counters overlap.");
}

pub fn features(features: &[Count<Feature>]) -> IoResult<()> {
    let stdout = &mut io::stdout().lock();

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)
}

fn item_total(items: &[impl Item], stdout: &mut StdoutLock) -> IoResult<u64> {
    let total = items.iter().map(Item::count).sum();
    writeln!(stdout, "= {total}")?;
    writeln!(stdout)?;
    Ok(total)
}

fn item_value(
    item: &impl Item, stdout: &mut StdoutLock, total: u64, nlen: usize, tab: usize,
) -> IoResult<()> {
    let count = item.count();
    let ratio = (count as f64 / total as f64) * 100.0;
    writeln!(stdout, "{:tab$}{:nlen$} {count} ({ratio:.2}%)", "", item.name())
}

fn data_body(items: &[impl Item], stdout: &mut StdoutLock, total: u64, tab: usize) -> IoResult<()> {
    let nlen = items.iter().map(Item::name).map(str::len).max().unwrap_or(0);

    for item in items {
        item_value(item, stdout, total, nlen, tab)?;
    }

    writeln!(stdout)
}

pub fn stats(items: &[impl Item]) -> IoResult<()> {
    let stdout = &mut io::stdout().lock();
    let total = item_total(items, stdout)?;
    data_body(items, stdout, total, 0)
}

pub fn details(details: &[Detail]) -> IoResult<()> {
    let stdout = &mut io::stdout().lock();
    let total = item_total(details, stdout)?;

    for detail in details {
        item_value(detail, stdout, total, 0, 0)?;
        data_body(detail.mnemonics(), stdout, total, 4)?;
    }

    Ok(())
}
