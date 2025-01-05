use chrono::Local;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    argon2::Argon2id,
    basic_crypto_functions::{sha256_stream, BasisCryptoError, Decrypter},
    ByteArray, EncodeTrait,
};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::{instrument, trace};

use crate::data_structures::dataset::DatasetTypeKind;

// Enum representing the direct trust errors
#[derive(Error, Debug)]
pub enum DatasetError {
    #[error("IO error {msg} -> caused by: {source}")]
    IO { msg: String, source: std::io::Error },
    #[error("Path doesn't exists {0}")]
    PathNotExist(PathBuf),
    #[error("Path is not a file {0}")]
    PathNotFile(PathBuf),
    #[error("Path is not a directory {0}")]
    PathIsNotDir(PathBuf),
    #[error("Crypto error {msg} -> caused by: {source}")]
    CryptoError {
        msg: String,
        source: BasisCryptoError,
    },
    #[error("Byte length error {0}")]
    ByteLengthError(String),
    #[error("Error Unzipping {file}: {source}")]
    Unzip {
        file: PathBuf,
        source: zip_extract::ZipExtractError,
    },
    #[error("Kind {0} delivered. Only context, setup and tally possible")]
    WrongKindStr(String),
}

/// Metadata containing the information of the zip dataset before and after extraction
#[derive(Debug, Clone)]
pub struct DatasetMetadata {
    dataset_kind: DatasetTypeKind,
    source_path: PathBuf,
    decrypted_zip_path: PathBuf,
    extracted_dir_path: PathBuf,
    fingerprint: ByteArray,
}

impl DatasetMetadata {
    /// New [DatasetMetadata]
    pub fn new(
        dataset_kind: DatasetTypeKind,
        source_path: &Path,
        decrypted_zip_path: &Path,
        extracted_dir_path: &Path,
        fingerprint: &ByteArray,
    ) -> Self {
        Self {
            dataset_kind,
            source_path: source_path.to_path_buf(),
            decrypted_zip_path: decrypted_zip_path.to_path_buf(),
            extracted_dir_path: extracted_dir_path.to_path_buf(),
            fingerprint: fingerprint.clone(),
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn decrypted_zip_path(&self) -> &Path {
        &self.decrypted_zip_path
    }
    pub fn extracted_dir_path(&self) -> &Path {
        &self.extracted_dir_path
    }
    pub fn fingerprint(&self) -> &ByteArray {
        &self.fingerprint
    }

    pub fn fingerprint_str(&self) -> String {
        self.fingerprint().base16_encode().unwrap()
    }

    pub fn kind(&self) -> DatasetTypeKind {
        self.dataset_kind
    }

    /// Extract the data as datatype given with the kind of the dataset type.
    ///
    /// Return [DatasetType] with the correct metadata or Error if something goes wrong
    pub fn extract_dataset_kind_with_inputs(
        kind: DatasetTypeKind,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, DatasetError> {
        Self::process_dataset_operations(kind, input, password, extract_dir, zip_temp_dir_path)
    }

    /// Extract the data as datatype give with the name of the dataset type.
    ///
    /// Return [DatasetType] with the correct metadata or Error if something goes wrong
    pub fn extract_dataset_str_with_inputs(
        datasettype_str: &str,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, DatasetError> {
        Self::extract_dataset_kind_with_inputs(
            DatasetTypeKind::try_from(datasettype_str)
                .map_err(|_| DatasetError::WrongKindStr(datasettype_str.to_string()))?,
            input,
            password,
            extract_dir,
            zip_temp_dir_path,
        )
    }

    fn calculate_fingerprint(input: &Path) -> Result<ByteArray, DatasetError> {
        let f = std::fs::File::open(input).map_err(|e| DatasetError::IO {
            msg: "Opening file for calculation of fingerprint".to_string(),
            source: e,
        })?;
        let mut reader = std::io::BufReader::new(f);
        sha256_stream(&mut reader).map_err(|e| DatasetError::CryptoError {
            msg: "Calculating fingerprint".to_string(),
            source: e,
        })
    }

    #[instrument(skip(password))]
    pub fn process_dataset_operations(
        datasetkind: DatasetTypeKind,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<DatasetMetadata, DatasetError> {
        if !input.exists() {
            return Err(DatasetError::PathNotExist(input.to_path_buf()));
        }
        if !input.is_file() {
            return Err(DatasetError::PathNotFile(input.to_path_buf()));
        }
        if !extract_dir.is_dir() {
            return Err(DatasetError::PathIsNotDir(extract_dir.to_path_buf()));
        }
        if !zip_temp_dir_path.is_dir() {
            return Err(DatasetError::PathIsNotDir(zip_temp_dir_path.to_path_buf()));
        }
        trace!("Start process_dataset_operations");
        let fingerprint = Self::calculate_fingerprint(input)?;
        let extract_dir_with_context = extract_dir.join(datasetkind.as_ref());
        let mut reader = EncryptedZipReader::new(
            input,
            password,
            &extract_dir_with_context,
            zip_temp_dir_path,
        )?;
        trace!("Zip decrypter");
        reader.unzip()?;
        trace!("unzip finished");
        Ok(DatasetMetadata::new(
            datasetkind,
            input,
            &extract_dir_with_context,
            &reader.temp_zip,
            &fingerprint,
        ))
    }
}

const SALT_BYTE_LENGTH: u8 = 16;
const NONCE_BYTE_LENGTH: u8 = 12;
const ENCRYPTED_BLOCK_SIZE: usize = 512;

/// Structure to decrypt the zip file and to extract the files
///
/// The zip will be first encrypted in the given location `target_dir`. The filename will extend with the word `encryption` and
/// the actual date time
///
/// In a second step, the extraction is done in the target directory (with strip away the topmost directory)
pub struct EncryptedZipReader {
    internal_reader: BufReader<File>,
    decrypter: Decrypter,
    target_dir: PathBuf,
    temp_zip: PathBuf,
}

impl EncryptedZipReader {
    /// New Reader of encrypted zip
    ///
    /// # parameters
    /// - File: path of the source file
    /// - password: The password for the decryption
    /// - target_dir: The target directory to extract the files. The extraction will strip away the topmost directory
    /// - temp_zip_dir: Location to the store the encrypted zip file
    pub fn new(
        file: &Path,
        password: &str,
        target_dir: &Path,
        temp_zip_dir: &Path,
    ) -> Result<Self, DatasetError> {
        let f = File::open(file).map_err(|e| DatasetError::IO {
            msg: format!("File {}", file.file_name().unwrap().to_str().unwrap()),
            source: e,
        })?;
        let mut buf = BufReader::new(f);
        let mut salt_buf: Vec<u8> = vec![0; SALT_BYTE_LENGTH as usize];
        let mut nonce_buf: Vec<u8> = vec![0; NONCE_BYTE_LENGTH as usize];
        let bytes_red = buf.read(&mut salt_buf).map_err(|e| DatasetError::IO {
            msg: "Reading salt".to_string(),
            source: e,
        })?;
        if bytes_red != SALT_BYTE_LENGTH as usize {
            return Err(DatasetError::ByteLengthError(format!(
                "size of bytes read {bytes_red} for salt wrong. Expected: {SALT_BYTE_LENGTH}"
            )));
        }
        let salt = ByteArray::from_bytes(&salt_buf);
        let bytes_red = buf.read(&mut nonce_buf).map_err(|e| DatasetError::IO {
            msg: "Reading nonce".to_string(),
            source: e,
        })?;
        if bytes_red != NONCE_BYTE_LENGTH as usize {
            return Err(DatasetError::ByteLengthError(format!(
                "size of bytes read {bytes_red} for nonce wrong. Expected: {NONCE_BYTE_LENGTH}"
            )));
        }
        let nonce = ByteArray::from_bytes(&nonce_buf);
        let derive_key = Argon2id::new_standard()
            .get_argon2id(&ByteArray::from(password), &salt)
            .unwrap();
        Ok(Self {
            internal_reader: buf,
            decrypter: Decrypter::new(&nonce, &derive_key).map_err(|e| {
                DatasetError::CryptoError {
                    msg: "Creating decrypter".to_string(),
                    source: e,
                }
            })?,
            target_dir: target_dir.to_path_buf(),
            temp_zip: Self::temp_zip_path(file, temp_zip_dir),
        })
    }

    fn decrypt_to_zip(&mut self) -> Result<PathBuf, DatasetError> {
        let mut target = std::fs::File::create(&self.temp_zip).map_err(|e| DatasetError::IO {
            msg: "Creating Temp Zip".to_string(),
            source: e,
        })?;
        let buf = &mut self.internal_reader;
        loop {
            let mut temp_buffer = vec![0; ENCRYPTED_BLOCK_SIZE];
            let count = buf.read(&mut temp_buffer).map_err(|e| DatasetError::IO {
                msg: "Reading buffer".to_string(),
                source: e,
            })?;
            if count == 0 {
                break;
            }
            temp_buffer.truncate(count);
            let plaintext = self
                .decrypter
                .decrypt(&ByteArray::from_bytes(&temp_buffer))
                .map_err(|e| DatasetError::CryptoError {
                    msg: "Decrypting cipher".to_string(),
                    source: e,
                })?;
            target
                .write_all(plaintext.to_bytes())
                .map_err(|e| DatasetError::IO {
                    msg: "Writing temp zip".to_string(),
                    source: e,
                })?;
        }
        Ok(self.temp_zip.to_owned())
    }

    /// Decrypt and unzip the source file
    ///
    /// The method return the target directory
    pub fn unzip(&mut self) -> Result<PathBuf, DatasetError> {
        if !self.temp_zip.exists() {
            self.decrypt_to_zip()?;
        }
        let f = std::fs::File::open(&self.temp_zip).map_err(|e| DatasetError::IO {
            msg: format!(
                "Opening temp zip file {}",
                self.temp_zip.file_name().unwrap().to_str().unwrap()
            ),
            source: e,
        })?;
        zip_extract::extract(&f, &self.target_dir, true).map_err(|e| DatasetError::Unzip {
            file: self.temp_zip.to_path_buf(),
            source: e,
        })?;
        Ok(self.target_dir.to_owned())
    }

    fn temp_zip_path(source: &Path, temp_zip_dir: &Path) -> PathBuf {
        let mut new_name = source.file_stem().unwrap().to_os_string();
        let now = Local::now().format("%Y%m%d-%H%M%S").to_string();
        new_name.push(format!(
            "-decrypted-{}.{}",
            now,
            source.extension().unwrap().to_str().unwrap()
        ));
        temp_zip_dir.join(new_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        test_datasets_context_zip_path, test_datasets_path, test_decrypt_zip_password,
        test_temp_dir_path, CONFIG_TEST,
    };
    use std::ffi::OsString;

    #[test]
    fn test_temp_zip_path() {
        let f = PathBuf::from("./toto/12345.zip");
        let res = EncryptedZipReader::temp_zip_path(&f, &PathBuf::from("/abc/xyz"));
        assert_eq!(res.extension(), Some(OsString::from("zip").as_os_str()));
        assert_eq!(res.parent(), Some(PathBuf::from("/abc/xyz").as_path()));
    }

    #[test]
    fn test_decrypt_zip() {
        let path = test_datasets_context_zip_path();
        let mut zip_reader = EncryptedZipReader::new(
            &path,
            test_decrypt_zip_password(),
            &test_datasets_path(),
            &CONFIG_TEST.zip_temp_dir_path(),
        )
        .unwrap();
        let decrypt_res = zip_reader.decrypt_to_zip();
        let res_path = decrypt_res.unwrap();
        assert_eq!(
            res_path.extension(),
            Some(OsString::from("zip").as_os_str())
        );
        assert_eq!(
            res_path.parent(),
            Some(CONFIG_TEST.zip_temp_dir_path().as_path())
        );
    }

    #[test]
    fn test_unzip() {
        let path = test_datasets_context_zip_path();
        let subdir_time =
            test_temp_dir_path().join(Local::now().format("%Y%m%d-%H%M%S").to_string());
        let _ = std::fs::create_dir(&subdir_time);
        let target_dir = subdir_time.join("context");
        let mut zip_reader = EncryptedZipReader::new(
            &path,
            test_decrypt_zip_password(),
            &target_dir,
            &CONFIG_TEST.zip_temp_dir_path(),
        )
        .unwrap();
        let unzip_res = zip_reader.unzip();
        let path_res = unzip_res.unwrap();
        assert_eq!(path_res, target_dir)
    }
}
