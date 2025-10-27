enum EntryType {
    Put,
    Delete,
}

struct LogEntry {
    entry_type: EntryType,
    key: Vec<u8>,
    value: Vec<u8>,
}
