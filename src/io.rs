use crate::binary::Segment;
use crate::decoder::{Decoder, Task};
use std::fmt::Arguments;
use std::fs::File as FsFile;
use std::io::{BufRead, BufReader, Result as IoResult, Seek, SeekFrom, StdoutLock, Write};

pub struct File {
    file: FsFile,
}

impl File {
    pub fn open(path: &str) -> IoResult<Self> {
        let file = FsFile::open(path)?;
        Ok(Self { file })
    }

    pub fn fs_file(&self) -> &FsFile {
        &self.file
    }

    pub fn is_dir(&self) -> IoResult<bool> {
        Ok(self.file.metadata()?.file_type().is_dir())
    }

    pub fn decode<T: Task>(
        &self, task: T, bitness: u32, segments: &[Segment],
    ) -> IoResult<T::Result> {
        let mut decoder = Decoder::new(bitness, task);

        for segment in segments {
            self.fs_file().seek(SeekFrom::Start(segment.offset()))?;

            let mut reader = BufReader::with_capacity(segment.size() as usize, self.fs_file());
            decoder.read(reader.fill_buf()?);
        }

        Ok(decoder.into_result())
    }
}

pub struct Stdout<'a> {
    lock: StdoutLock<'a>,
}

impl<'a> Stdout<'a> {
    pub fn new() -> Self {
        Self { lock: std::io::stdout().lock() }
    }

    pub fn write_fmt(&mut self, args: Arguments<'_>) -> IoResult<()> {
        self.lock.write_fmt(args)
    }
}
