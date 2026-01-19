use std::error::Error;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

const FILENAME: &'static str = "data";

pub struct Datastore {
    file: File,
}

impl Datastore {
    pub fn new() -> Datastore {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .open(FILENAME)
            .expect(&format!("cannot open file {}", FILENAME));

        Datastore { file }
    }

    pub fn get_last_published_timestamp(&mut self) -> Option<i64> {
        let mut buf = [0u8; 8];

        self.file.seek(SeekFrom::Start(0)).ok()?;
        self.file.read_exact(&mut buf).ok()?;

        Some(i64::from_le_bytes(buf))
    }

    pub fn set_last_published_timestamp(&mut self, timestamp: i64) -> Result<(), Box<dyn Error>> {
        self.file.seek(SeekFrom::Start(0))?;
        let raw = timestamp.to_le_bytes();
        self.file.write(&raw)?;
        self.file.flush()?;

        Ok(())
    }
}

