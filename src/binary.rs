use crate::types::{Arr, Str};
use object::{
    Architecture, BinaryFormat, File, Object, ObjectSection, ReadCache, ReadRef, Result, Section,
    SectionKind,
};

pub struct SectionInfo {
    name: Option<Str>,
    address: u64,
    size: u64,
    range: (u64, u64),
}

impl SectionInfo {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn range(&self) -> (u64, u64) {
        self.range
    }
}

pub struct BinaryInfo {
    format: BinaryFormat,
    arch: Architecture,
    sections: Arr<SectionInfo>,
}

impl BinaryInfo {
    pub fn format(&self) -> BinaryFormat {
        self.format
    }

    pub fn arch(&self) -> Architecture {
        self.arch
    }

    pub fn bitness(&self) -> Option<u32> {
        use Architecture::*;
        match self.arch {
            X86_64 => Some(64),
            X86_64_X32 | I386 => Some(32),
            _ => None,
        }
    }

    pub fn sections(&self) -> &[SectionInfo] {
        &self.sections
    }
}

fn map_section<'a>(section: Section<'a, 'a, impl ReadRef<'a>>) -> Option<SectionInfo> {
    match section.kind() {
        SectionKind::Text => section.file_range().map(|range| SectionInfo {
            name: section.name().ok().map(Str::from),
            address: section.address(),
            size: section.size(),
            range,
        }),
        _ => None,
    }
}

pub fn parse(file: &std::fs::File) -> Result<BinaryInfo> {
    let cache = ReadCache::new(file);
    let binary = File::parse(&cache)?;

    Ok(BinaryInfo {
        format: binary.format(),
        arch: binary.architecture(),
        sections: binary.sections().filter_map(map_section).collect(),
    })
}
