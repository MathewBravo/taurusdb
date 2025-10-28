use std::{fs::File, io::Error};

use crate::logentry::{EntryType, LogEntry};

struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    pub fn new(file: File) -> Self {
        WriteAheadLog { file }
    }

    pub fn append() -> Result<(), Error> {
        todo!()
    }

    pub fn replay() -> Result<Vec<LogEntry>, Error> {
        todo!()
    }
}
