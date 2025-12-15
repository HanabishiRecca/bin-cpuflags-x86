use crate::binary::Segment;
use crate::decoder::{Decoder, Task};
use std::fs::File;
use std::io::{BufRead, BufReader, Result as IoResult, Seek, SeekFrom};

pub struct Reader {
    file: File,
}

impl Reader {
    pub fn open(path: &str) -> IoResult<Option<Self>> {
        let file = File::open(path)?;
        let file_type = file.metadata()?.file_type();
        Ok(file_type.is_file().then_some(Self { file }))
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn read<T: Task>(
        &self, task: T, bitness: u32, segments: &[Segment],
    ) -> IoResult<T::Result> {
        let mut decoder = Decoder::new(bitness, task);
        let mut file = &self.file;

        for segment in segments {
            file.seek(SeekFrom::Start(segment.offset()))?;

            let mut reader = BufReader::with_capacity(segment.size() as usize, file);
            decoder.read(reader.fill_buf()?);
        }

        Ok(decoder.into_result())
    }
}
