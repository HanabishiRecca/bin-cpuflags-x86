use crate::binary::{Binary, Segment};
use crate::cli::Output;
use crate::decoder::{Count, Detail, Feature, Item, Register};
use std::cell::Cell;
use std::env;
use std::error::Error;

pub fn help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        PKG = env!("CARGO_PKG_NAME"),
        VER = env!("CARGO_PKG_VERSION"),
        BIN_NAME = (|| bin.as_ref()?.file_name()?.to_str())().unwrap_or(env!("CARGO_BIN_NAME")),
    );
}

#[cold]
pub fn error(error: &dyn Error) {
    eprintln!("Error: {error}");
}

thread_local! {
    static OUTPUT: Cell<Output> = const { Cell::new(Output::Normal) };
}

pub fn set_output(value: Output) {
    OUTPUT.set(value);
}

macro_rules! output {
    ($value: expr) => {{
        use Output::*;
        OUTPUT.get() >= $value
    }};
}

macro_rules! only {
    ($value: expr) => {
        if !output!($value) {
            return;
        }
    };
}

pub fn file_path(path: &str) {
    only!(Verbose);
    println!("Reading '{path}'...");
}

pub fn binary(binary: &Binary) {
    only!(Normal);
    println!("Format: {}", binary.format());
    println!("Architecture: {}", binary.architecture());
}

fn segment(segment: &Segment) {
    println!(
        "    {} => 0x{:x}, {} bytes",
        segment.name().unwrap_or_default(),
        segment.offset(),
        segment.size(),
    );
}

pub fn segments(segments: &[Segment]) {
    only!(Verbose);
    println!("Text sections:");
    segments.iter().for_each(segment);
}

fn header(text: &str) {
    println!("{text}");
    println!("{:-<1$}", "", text.len());
}

fn stats_note() {
    println!("Note: instructions that belong to multiple feature sets make counters overlap.");
}

fn feature(feature: &Count<Feature>) {
    print!("{} ", feature.name());
}

pub fn features(features: &[Count<Feature>]) {
    if output!(Normal) {
        if features.iter().any(Count::is_cpuid) {
            println!("Warning: CPUID usage detected, features could switch in runtime.");
        }

        print!("Features: ");
    }

    features.iter().for_each(feature);
    println!();
}

fn item_total(items: &[impl Item]) -> u64 {
    let total = items.iter().map(Item::count).sum();
    println!("= {total}");
    println!();
    total
}

fn item_value(item: &impl Item, total: u64, nlen: usize, tab: usize) {
    let count = item.count();
    let ratio = (count as f64 / total as f64) * 100.0;
    println!("{:tab$}{:nlen$} {count} ({ratio:.2}%)", "", item.name());
}

fn data_body(items: &[impl Item], total: u64, tab: usize) {
    let nlen = items.iter().map(Item::name).map(str::len).max().unwrap_or(0);

    for item in items {
        item_value(item, total, nlen, tab);
    }

    println!();
}

fn items(items: &[impl Item]) {
    let total = item_total(items);
    data_body(items, total, 0);
}

pub fn stats(stats: &[Count<Feature>]) {
    if output!(Normal) {
        println!();
        stats_note();
    }

    items(stats);
}

pub fn details(details: &[Detail]) {
    if output!(Normal) {
        println!();
        header("Instructions");
        stats_note();
    }

    let total = item_total(details);

    for detail in details {
        item_value(detail, total, 0, 0);
        data_body(detail.mnemonics(), total, 4);
    }
}

pub fn registers(registers: &[Count<Register>]) {
    if output!(Normal) {
        println!();
        header("Registers");
    }

    items(registers);
}
