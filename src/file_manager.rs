use std::{
    fs::{OpenOptions, create_dir_all, read_dir, rename},
    io::{Bytes, Error, ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

#[derive(Debug)]
pub struct FileManager {
    db_dir_path: PathBuf,
    next_file_number: AtomicU64,
}

#[derive(Debug)]
pub enum Name {
    SSTable,
    WriteAheadLog,
    Manifest,
    Current,
    Lock,
}

impl FileManager {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        if path.exists() && path.is_file() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "db already exists as a file at that location",
            ));
        } else if path.exists() && path.is_dir() {
            let mut contents = read_dir(&path)?;
            if contents.next().is_some() {
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    "db already exists at that location",
                ));
            }
        } else {
            create_dir_all(&path)?;
        }

        initialize_db_files(&path)?;

        Ok(FileManager {
            db_dir_path: path,
            next_file_number: AtomicU64::new(2),
        })
    }
}

fn initialize_db_files(path: &PathBuf) -> Result<(), Error> {
    // will create initial manifest, MANIFEST-000001
    // acquire LOCK file
    // create current file pointing to manifest
    // return fm with path and counter = 2
    let lock_path = path.join("LOCK");
    let mut lf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)?;

    lf.write_all(std::process::id().to_string().as_bytes())?;
    lf.sync_all()?;

    let manifest_path = path.join("MANIFEST-000001");
    let mut mf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(manifest_path)?;

    mf.write_all(b"next_file_number: 2\n")?;
    mf.sync_all()?;

    let curtmp_path = path.join("CURRENT.tmp");
    let current_path = path.join("CURRENT");
    let mut cf = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&curtmp_path)?;

    cf.write_all(b"MANIFEST-000001\n")?;
    cf.sync_all()?;
    drop(cf);

    rename(&curtmp_path, &current_path)?;

    Ok(())
}
