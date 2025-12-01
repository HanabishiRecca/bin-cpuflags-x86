use crate::types::{Arr, Str};
use object::{
    Architecture, BinaryFormat, File, Object, ObjectSection, ReadCache, ReadRef, Result, Section,
    SectionKind,
};

pub struct Segment {
    name: Option<Str>,
    offset: u64,
    size: u64,
}

impl Segment {
    pub fn new(name: Option<Str>, offset: u64, size: u64) -> Self {
        Self { name, offset, size }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

pub struct Binary {
    format: BinaryFormat,
    architecture: Architecture,
    segments: Arr<Segment>,
}

impl Binary {
    pub fn new(format: BinaryFormat, architecture: Architecture, segments: Arr<Segment>) -> Self {
        Self { format, architecture, segments }
    }

    pub fn format(&self) -> BinaryFormat {
        self.format
    }

    pub fn architecture(&self) -> Architecture {
        self.architecture
    }

    pub fn bitness(&self) -> Option<u32> {
        use Architecture::*;
        match self.architecture {
            X86_64 => Some(64),
            X86_64_X32 | I386 => Some(32),
            _ => None,
        }
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

fn map_section<'a>(section: Section<'a, 'a, impl ReadRef<'a>>) -> Option<Segment> {
    use SectionKind::*;
    match section.kind() {
        Text => section
            .file_range()
            .map(|(offset, size)| Segment::new(section.name().ok().map(Str::from), offset, size)),
        _ => None,
    }
}

pub fn parse(file: &std::fs::File) -> Result<Binary> {
    let cache = ReadCache::new(file);
    let binary = File::parse(&cache)?;

    Ok(Binary::new(
        binary.format(),
        binary.architecture(),
        binary.sections().filter_map(map_section).collect(),
    ))
}
