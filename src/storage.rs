use ::csv::{ReaderBuilder, WriterBuilder};
use std::{
    error::Error,
    fmt, fs,
    io::{Read, Write},
    path::PathBuf,
};
use tracing::log::{debug, error, warn};

/*
 * The idea of storages
 *
 * To abstract file manipulations on business logic level file abstractions
 * has been used. Since currently the whole database storage is based on files,
 * we can have one common interface (Repository) for writing and reading
 * serialized data.
 *
 * Second level of abstraction are structures which implements correct
 * deserialization and are mimicking the database behavior (CSV files, TOML etc.).
 *
 * The third level of abstraction implements concrete functionalities dependant on
 * information stored in the databases.
 *
 * Example abstraction (from bottom to top)
 *  1. File storage  (read/write/append)
 *  2. TOML database (serialize/deserialize)
 *  3. User config   (field change, search)
 *
*/

#[derive(Debug)]
pub enum StorageError {
    FileAlreadyExists,
    FileNotExists,
    UnableToRestoreFile,
    UnableToBackupFile,
    UnableToWriteToFile,
    UnableToReadFromFile,
    UnableToOpenFile,
    UnableToSeek,
    CannotOverwriteFile,
    UnableToCreateFile,
    UnableToCreateDir,
    UnableToSerialize,
    UnableToDeserialize,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::FileAlreadyExists => write!(f, "File already exists"),
            StorageError::FileNotExists => write!(f, "File not exists"),
            StorageError::UnableToRestoreFile => write!(f, "Unable to restore file"),
            StorageError::UnableToBackupFile => write!(f, "Unable to backup file"),
            StorageError::UnableToWriteToFile => write!(f, "Unable to write to file"),
            StorageError::UnableToReadFromFile => write!(f, "Unable to read from file"),
            StorageError::UnableToSeek => write!(f, "Unable to seek"),
            StorageError::UnableToOpenFile => write!(f, "Unable to open file"),
            StorageError::CannotOverwriteFile => write!(f, "Cannot overwrite file"),
            StorageError::UnableToCreateFile => write!(f, "Unable to create file"),
            StorageError::UnableToCreateDir => write!(f, "Unable to create directory"),
            StorageError::UnableToSerialize => write!(f, "Unable to serialize"),
            StorageError::UnableToDeserialize => write!(f, "Unable to deserialize"),
        }
    }
}

impl Error for StorageError {}

pub trait Repository
where
    Self: Sized,
{
    type Error: Error + 'static;

    fn create(&self) -> Result<(), Self::Error>;
    fn exists(&self) -> bool;
    fn and_create(self) -> Result<Self, Self::Error> {
        self.create()?;
        Ok(self)
    }
}

pub trait BasicRepository: Repository
where
    Self: Sized,
{
    fn read(&mut self) -> Result<(), Self::Error>;
    fn write(&mut self) -> Result<(), Self::Error>;
    fn and_read(mut self) -> Result<Self, Self::Error> {
        self.read()?;
        Ok(self)
    }
}

pub trait BackupRepository: Repository {
    const BACKUP_EXT: &'static str = "bak";

    fn backup(&self) -> Result<(), Self::Error>;
    fn restore(&mut self) -> Result<(), Self::Error>;
}

pub trait AppendableRepository: Repository {
    type Input;
    fn append(&mut self, data: &Self::Input) -> Result<(), Self::Error>;
}

#[derive(Clone)]
pub struct FileStorage {
    content: Option<Vec<u8>>,
    file_path: PathBuf,
}

impl FileStorage {
    pub fn content(&self) -> &Vec<u8> {
        let Some(content) = &self.content else {
            error!("Trying to access not loaded content.");
            panic!("Trying to access not loaded content.");
        };

        content
    }

    pub fn content_set(&mut self, content: &Vec<u8>) {
        self.content = Some(content.to_vec());
    }
}

impl std::fmt::Debug for FileStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Content of file '{}'.",
            self.file_path.to_str().unwrap_or_default()
        )
    }
}

impl From<&PathBuf> for FileStorage {
    fn from(path: &PathBuf) -> Self {
        FileStorage {
            content: None,
            file_path: path.to_owned(),
        }
    }
}

impl Repository for FileStorage {
    type Error = StorageError;

    fn create(&self) -> Result<(), Self::Error> {
        if self.file_path.exists() {
            return Err(StorageError::FileAlreadyExists);
        }

        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|_| StorageError::UnableToCreateDir)?;
        }

        fs::File::create(&self.file_path).map_err(|_| StorageError::UnableToCreateFile)?;
        Ok(())
    }

    fn exists(&self) -> bool {
        self.file_path.exists()
    }
}

impl BasicRepository for FileStorage {
    fn read(&mut self) -> Result<(), Self::Error> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(&self.file_path)
            .map_err(|_| StorageError::UnableToOpenFile)?;

        let mut file_content = Vec::new();
        if let Err(err) = file.read_to_end(&mut file_content) {
            error!("Unable to read from file.");
            debug!("{}", err);
            return Err::<(), StorageError>(StorageError::UnableToReadFromFile);
        };

        self.content = Some(Vec::from(file_content));

        Ok(())
    }

    fn write(&mut self) -> Result<(), Self::Error> {
        if let Err(err) = fs::write(&self.file_path, self.content()) {
            error!("Cannot overwrite file.");
            debug!("{}", err);
            return Err(StorageError::CannotOverwriteFile);
        }

        Ok(())
    }
}

impl AppendableRepository for FileStorage {
    type Input = Vec<u8>;
    fn append(&mut self, data: &Self::Input) -> Result<(), Self::Error> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|_| StorageError::UnableToOpenFile)?;

        file.write(&data)
            .map_err(|_| StorageError::UnableToWriteToFile)?;

        match &mut self.content {
            Some(u) => u.extend(data),
            None => self.content = Some(data.clone()),
        };

        Ok(())
    }
}

impl FileStorage {
    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn to_string(&self) -> Result<String, Box<dyn Error>> {
        match &self.content {
            Some(c) => Ok(std::str::from_utf8(c)?.to_string()),
            None => Ok("".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct TomlFile<T> {
    content: Option<T>,
    file: FileStorage,
}

impl<T> TomlFile<T>
where
    T: Clone,
{
    pub fn content(&self) -> &T {
        let Some(content) = &self.content else {
            error!("Trying to access not loaded content.");
            panic!("Trying to access not loaded content.");
        };

        content
    }

    pub fn content_mut(&mut self) -> &mut T {
        let Some(content) = &mut self.content else {
            error!("Trying to access mutually not loaded content.");
            panic!("Trying to access mutually not loaded content.");
        };

        content
    }
}

impl<T> std::fmt::Debug for TomlFile<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Content of TOML file '{}'.",
            self.file.path().to_str().unwrap_or_default()
        )
    }
}

impl<T> From<&PathBuf> for TomlFile<T> {
    fn from(path: &PathBuf) -> Self {
        TomlFile {
            content: None,
            file: FileStorage::from(path),
        }
    }
}

impl<T> Repository for TomlFile<T> {
    type Error = StorageError;

    fn create(&self) -> Result<(), Self::Error> {
        self.file.create()
    }

    fn exists(&self) -> bool {
        self.file.exists()
    }
}

impl<T> BasicRepository for TomlFile<T>
where
    T: serde::de::DeserializeOwned + serde::ser::Serialize + Clone,
{
    fn read(&mut self) -> Result<(), Self::Error> {
        self.file.read()?;

        let Ok(ser_content) = self.file.to_string() else {
            return Err(StorageError::UnableToReadFromFile);
        };

        self.content = match toml::from_str::<T>(&ser_content) {
            Ok(c) => Some(c),
            Err(err) => {
                warn!(
                    "Error while deserializing '{}'.",
                    self.file.path().to_str().unwrap_or_default()
                );
                debug!("{}", err.to_string());
                return Err(StorageError::UnableToDeserialize);
            }
        };

        Ok(())
    }

    fn write(&mut self) -> Result<(), Self::Error> {
        let ser_content = match toml::to_string(self.content()) {
            Ok(s) => s,
            Err(err) => {
                warn!(
                    "Error while serializing '{}'.",
                    self.file.path().to_str().unwrap_or_default()
                );
                debug!("{}", err.to_string());
                return Err(StorageError::UnableToSerialize);
            }
        };

        self.file.content = Some(ser_content.into_bytes());

        self.file.write()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct CsvFile<T> {
    content: Option<Vec<T>>,
    file: FileStorage,
}

impl<T> std::fmt::Debug for CsvFile<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Content of CSV file '{}'.",
            self.file.path().to_str().unwrap_or_default()
        )
    }
}

impl<T> From<&PathBuf> for CsvFile<T> {
    fn from(path: &PathBuf) -> Self {
        CsvFile {
            content: None,
            file: FileStorage::from(path),
        }
    }
}

impl<T> Repository for CsvFile<T> {
    type Error = StorageError;

    fn create(&self) -> Result<(), Self::Error> {
        self.file.create()
    }

    fn exists(&self) -> bool {
        self.file.exists()
    }
}

impl<T> BackupRepository for CsvFile<T> {
    fn backup(&self) -> Result<(), Self::Error> {
        if let Err(err) = fs::copy(
            &self.file.path(),
            self.file.path().with_added_extension(Self::BACKUP_EXT),
        ) {
            debug!("{}", err);
            return Err(Self::Error::UnableToBackupFile);
        }

        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        if let Err(err) = fs::copy(
            self.file.path().with_added_extension(Self::BACKUP_EXT),
            &self.file.path(),
        ) {
            debug!("{}", err);
            return Err(Self::Error::UnableToRestoreFile);
        };

        self.file.read()?;

        Ok(())
    }
}

impl<T> BasicRepository for CsvFile<T>
where
    T: serde::de::DeserializeOwned + serde::ser::Serialize + Clone,
{
    fn read(&mut self) -> Result<(), Self::Error> {
        self.file.read()?;
        let Some(content) = &self.file.content else {
            self.content = None;
            return Ok(());
        };

        let mut reader = self.make_reader(content);
        let deserialized_records = reader.deserialize();

        let mut content = Vec::<T>::new();
        for record in deserialized_records {
            match record {
                Ok(r) => {
                    content.push(r);
                }

                Err(err) => {
                    warn!(
                        "Error while deserializing '{}'.",
                        self.file.path().to_str().unwrap_or_default()
                    );
                    debug!("{}", err.to_string());
                    return Err(StorageError::UnableToDeserialize);
                }
            }
        }

        self.content = Some(content);

        Ok(())
    }

    fn write(&mut self) -> Result<(), Self::Error> {
        self.backup()?;

        self.file.content = Some(self.serialize(self.content().as_slice())?);

        if let Err(err) = self.file.write() {
            self.restore()?;
            return Err(err);
        }

        Ok(())
    }
}

impl<T> AppendableRepository for CsvFile<T>
where
    T: serde::ser::Serialize + serde::de::DeserializeOwned + Clone,
{
    type Input = T;
    fn append(&mut self, data: &Self::Input) -> Result<(), Self::Error> {
        self.backup()?;

        // THE TRICK
        // When appending to the CSV file a serialized data,
        // it is important to ensure we are staring a new record
        // from the new line.
        // The tick here is to read the last byte if it contains
        // the new line character, if not add one before appending
        // new record.

        let Some(raw_content) = &self.file.content else {
            return Err(StorageError::UnableToSeek);
        };

        if raw_content.last().is_some_and(|last| *last != b'\n') {
            if let Err(err) = self.file.append(&vec![b'\n']) {
                self.restore()?;
                return Err::<(), StorageError>(err);
            };
        }

        let serialized_data = self.serialize(std::slice::from_ref(data))?;
        if let Err(err) = self.file.append(&serialized_data) {
            self.restore()?;
            return Err::<(), StorageError>(err);
        };

        match &mut self.content {
            Some(c) => c.push(data.clone()),
            None => self.content = Some(vec![data.clone()]),
        }

        Ok(())
    }
}

impl<T> CsvFile<T>
where
    T: serde::ser::Serialize + serde::de::DeserializeOwned + Clone,
{
    fn make_writer(&self, data: &Vec<u8>) -> ::csv::Writer<Vec<u8>> {
        WriterBuilder::new()
            .has_headers(false)
            .from_writer(data.to_vec())
    }

    fn make_reader<'a>(&self, data: &'a Vec<u8>) -> ::csv::Reader<&'a [u8]> {
        ReaderBuilder::new()
            .has_headers(false)
            .from_reader(data.as_slice())
    }

    fn serialize(&self, records: &[T]) -> Result<Vec<u8>, StorageError> {
        let mut writer = self.make_writer(&vec![]);

        // We MUST iterate over records, without this, records gets
        // concat to single record.
        for record in records.iter() {
            if let Err(err) = writer.serialize(record) {
                warn!(
                    "Cannot serialize '{}'.",
                    self.file.path().to_str().unwrap_or_default()
                );
                debug!("{}", err);
                return Err(StorageError::UnableToSerialize);
            }
        }

        match writer.into_inner() {
            Ok(serialized_records) => Ok(serialized_records),
            Err(err) => {
                warn!(
                    "Cannot serialize '{}'.",
                    self.file.path().to_str().unwrap_or_default()
                );
                debug!("{}", err);
                return Err(StorageError::UnableToSerialize);
            }
        }
    }

    pub fn from_json(&self, data: &str) -> Result<Vec<T>, Box<dyn Error>> {
        match serde_json::from_str(data) {
            Ok(c) => Ok(c),
            Err(err) => {
                error!("Unable to deserialize from json.");
                debug!("{}", err);
                Err(err.into())
            }
        }
    }

    pub fn content(&self) -> &Vec<T> {
        let Some(content) = &self.content else {
            error!("Trying to access not loaded content.");
            panic!("Trying to access not loaded content.");
        };

        content
    }

    pub fn content_mut(&mut self) -> &mut Vec<T> {
        let Some(content) = &mut self.content else {
            error!("Trying to access mutually not loaded content.");
            panic!("Trying to access mutually not loaded content.");
        };

        content
    }

    pub fn content_set(&mut self, content: &Vec<T>) {
        self.content = Some(content.to_vec());
    }
}
