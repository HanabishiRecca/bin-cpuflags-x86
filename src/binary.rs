use crate::types::{Arr, Str};
use object::{
    Architecture, BinaryFormat, File, Object, ObjectSection, ReadCache, ReadRef,
    Result as ObjResult, Section, SectionKind,
};
use std::fs::File as FsFile;

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

fn map_segment<'a>(section: Section<'a, 'a, impl ReadRef<'a>>) -> Option<Segment> {
    (section.kind() == SectionKind::Text).then_some(())?;
    let (offset, size) = section.file_range()?;
    let name = section.name().ok().map(Str::from);
    Some(Segment::new(name, offset, size))
}

impl Binary {
    fn new(format: BinaryFormat, architecture: Architecture, segments: Arr<Segment>) -> Self {
        Self { format, architecture, segments }
    }

    pub fn parse(file: &FsFile) -> ObjResult<Self> {
        let cache = ReadCache::new(file);
        let binary = File::parse(&cache)?;
        let segments = binary.sections().filter_map(map_segment).collect();
        Ok(Self::new(binary.format(), binary.architecture(), segments))
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

    pub fn into_segments(self) -> Arr<Segment> {
        self.segments
    }
}
