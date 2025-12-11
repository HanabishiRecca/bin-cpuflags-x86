use crate::binary::{Binary, Segment};
use crate::decoder::{Counter, DetailCounter, FeatureCounter};
use crate::io::Stdout;
use std::error::Error;
use std::io::Result as IoResult;

pub fn help() {
    let bin = std::env::current_exe().ok();
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
    let len = text.len();
    println!("{text:^len$}");
    println!("{:-<len$}", "");
}

pub fn cpuid(features: &[FeatureCounter]) {
    if features.iter().any(FeatureCounter::is_cpuid) {
        println!("Warning: CPUID usage detected, features could switch in runtime.");
    }
}

pub fn stats_note() {
    println!("Note: instructions that belong to multiple feature sets make counters overlap.");
}

pub fn features(features: &[FeatureCounter]) -> IoResult<()> {
    let stdout = &mut Stdout::new();

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)
}

fn counters_total(counters: &[impl Counter], stdout: &mut Stdout) -> IoResult<u64> {
    let total = counters.iter().map(Counter::count).sum();
    writeln!(stdout, "= {total}")?;
    writeln!(stdout)?;
    Ok(total)
}

fn counter_value(
    counter: &impl Counter, stdout: &mut Stdout, total: u64, nlen: usize, tab: usize,
) -> IoResult<()> {
    let count = counter.count();
    let ratio = (count as f64 / total as f64) * 100.0;
    writeln!(stdout, "{:tab$}{:nlen$} {count} ({ratio:.2}%)", "", counter.name())
}

fn counters_body(
    counters: &[impl Counter], stdout: &mut Stdout, total: u64, tab: usize,
) -> IoResult<()> {
    let nlen = counters.iter().map(Counter::name).map(str::len).max().unwrap_or(0);

    for counter in counters {
        counter_value(counter, stdout, total, nlen, tab)?;
    }

    writeln!(stdout)
}

pub fn stats(stats: &[impl Counter]) -> IoResult<()> {
    let stdout = &mut Stdout::new();
    let total = counters_total(stats, stdout)?;
    counters_body(stats, stdout, total, 0)
}

pub fn details(details: &[DetailCounter]) -> IoResult<()> {
    let stdout = &mut Stdout::new();
    let total = counters_total(details, stdout)?;

    for detail in details {
        counter_value(detail, stdout, total, 0, 0)?;
        counters_body(detail.mnemonics(), stdout, total, 4)?;
    }

    Ok(())
}
