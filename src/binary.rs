use object::{self, Architecture, Object, ObjectSection, ReadCache, ReadRef, SectionKind};
use std::fs;

use crate::{
    check,
    cli::OutputMode,
    error::{AppError, R},
};

type Params = (Vec<(u64, u64)>, u32);

fn read_header<'a>(data: impl ReadRef<'a>, output_mode: OutputMode) -> R<Params> {
    let file = object::File::parse(data)?;
    let architecture = file.architecture();

    if output_mode > OutputMode::Quiet {
        println!("Format: {:?}", file.format());
        println!("Architecture: {architecture:?}");
    }

    check!(
        matches!(
            architecture,
            Architecture::X86_64 | Architecture::X86_64_X32 | Architecture::I386
        ),
        AppError::WrongArch,
    );

    if output_mode > OutputMode::Normal {
        println!("Text sections: ");
    }

    let sections = file
        .sections()
        .filter_map(|s| match s.kind() {
            SectionKind::Text => {
                if output_mode > OutputMode::Normal {
                    println!(
                        "    {} => 0x{:x}, {} bytes",
                        s.name().unwrap_or_default(),
                        s.address(),
                        s.size()
                    );
                }
                s.file_range()
            }
            _ => None,
        })
        .collect();

    let bitness = match architecture {
        Architecture::X86_64 => 64,
        _ => 32,
    };

    Ok((sections, bitness))
}

pub fn parse(file: &fs::File, output_mode: OutputMode) -> R<Params> {
    read_header(&ReadCache::new(file), output_mode)
}
