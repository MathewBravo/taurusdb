use std::{
    fs::{OpenOptions, create_dir_all, read_dir, rename},
    io::{Error, ErrorKind, Read, Write},
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

    pub fn open_existing(path: PathBuf) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "db directory not found"));
        }
        let cp = path.join("CURRENT");
        let mut cf = OpenOptions::new().read(true).create(false).open(&cp)?;

        let mut contents = String::new();
        cf.read_to_string(&mut contents)?;

        let manifest = Path::new(contents.trim());

        let next_file = get_next_file_num(manifest)?;

        Ok(FileManager {
            db_dir_path: path,
            next_file_number: next_file,
        })
    }
}

fn initialize_db_files(path: &PathBuf) -> Result<(), Error> {
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

fn get_next_file_num(manifest_path: &Path) -> Result<AtomicU64, Error> {
    let mut mf = OpenOptions::new()
        .read(true)
        .create(false)
        .open(manifest_path)?;

    let mut manifest_contents = String::new();

    mf.read_to_string(&mut manifest_contents)?;

    let line = manifest_contents
        .lines()
        .find_map(|line| {
            let next_line = line.strip_prefix("next_file_number=")?;
            next_line.parse::<u64>().ok()
        })
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "next_file_number not found or invalid",
            )
        })?;

    Ok(AtomicU64::from(line))
}
