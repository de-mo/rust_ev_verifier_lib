// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use crate::data_structures::dataset::DatasetTypeKind;
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

#[derive(Error, Debug)]
#[error(transparent)]
/// Error with dataset
pub struct DatasetError(#[from] DatasetErrorImpl);

#[derive(Error, Debug)]
enum DatasetErrorImpl {
    #[error("Kind {0} delivered. Only context, setup and tally possible")]
    WrongKindStr(String),
    #[error("Error opening file {path} to calculate fingerprint")]
    IOFingerprint {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Error calculating fingerprint of {path}")]
    Fingerprint {
        path: PathBuf,
        source: BasisCryptoError,
    },
    #[error("Path doesn't exists {0}")]
    PathNotExist(PathBuf),
    #[error("Path is not a file {0}")]
    PathNotFile(PathBuf),
    #[error("Path is not a directory {0}")]
    PathIsNotDir(PathBuf),
    #[error("IO Error form {path}: {msg}")]
    IO {
        path: PathBuf,
        msg: &'static str,
        source: std::io::Error,
    },
    #[error("IO Error with the buffer: {msg}")]
    IOBuf {
        msg: &'static str,
        source: std::io::Error,
    },
    #[error("Byte length error {0}")]
    ByteLengthError(String),
    #[error("Error creating decrypter")]
    Decrypter { source: BasisCryptoError },
    #[error("Error deecrypting the zip file")]
    Decrypt { source: BasisCryptoError },
    #[error("Error extracting {file}")]
    Extract {
        file: PathBuf,
        source: zip_extract::ZipExtractError,
    },
    #[error("process_dataset_operations: Error crreating EncryptedZipReader")]
    ProcessNewEncryptedZipReader { source: Box<DatasetError> },
    #[error("process_dataset_operations: Error unzipping")]
    ProcessUnzip { source: Box<DatasetError> },
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

    /// Source path
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Path of the zip decrypted
    pub fn decrypted_zip_path(&self) -> &Path {
        &self.decrypted_zip_path
    }

    /// Path of the folder containing the extracted files
    pub fn extracted_dir_path(&self) -> &Path {
        &self.extracted_dir_path
    }

    /// Fingerprint of the source encrypted zip
    pub fn fingerprint(&self) -> &ByteArray {
        &self.fingerprint
    }

    /// Fingerprint of the source encrypted zip as string (16 coding)
    pub fn fingerprint_str(&self) -> String {
        self.fingerprint().base16_encode().unwrap()
    }

    /// Kind of the dataset
    pub fn kind(&self) -> DatasetTypeKind {
        self.dataset_kind
    }

    /// Extract the data as datatype given with the kind of the dataset type.
    ///
    /// Return [DatasetMetadata] with the correct metadata or Error if something goes wrong
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
    /// Return [DatasetMetadata] with the correct metadata or Error if something goes wrong
    pub fn extract_dataset_str_with_inputs(
        datasettype_str: &str,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, DatasetError> {
        Self::extract_dataset_kind_with_inputs(
            DatasetTypeKind::try_from(datasettype_str)
                .map_err(|_| DatasetErrorImpl::WrongKindStr(datasettype_str.to_string()))?,
            input,
            password,
            extract_dir,
            zip_temp_dir_path,
        )
    }

    fn calculate_fingerprint(input: &Path) -> Result<ByteArray, DatasetErrorImpl> {
        let f = std::fs::File::open(input).map_err(|e| DatasetErrorImpl::IOFingerprint {
            path: input.to_path_buf(),
            source: e,
        })?;
        let mut reader = std::io::BufReader::new(f);
        sha256_stream(&mut reader).map_err(|e| DatasetErrorImpl::Fingerprint {
            path: input.to_path_buf(),
            source: e,
        })
    }

    /// Process all the operations
    ///
    /// - Decrypt the zip defined by `input` to the `zip_temp_dir_path` using the password
    /// - Extract the decrypted zip to `extract_dir`
    ///
    /// Return [Self] with the correct metadata or Error if something goes wrong
    #[instrument(skip(password))]
    pub fn process_dataset_operations(
        datasetkind: DatasetTypeKind,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, DatasetError> {
        Self::process_dataset_operations_impl(
            datasetkind,
            input,
            password,
            extract_dir,
            zip_temp_dir_path,
        )
        .map_err(DatasetError::from)
    }

    fn process_dataset_operations_impl(
        datasetkind: DatasetTypeKind,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, DatasetErrorImpl> {
        if !input.exists() {
            return Err(DatasetErrorImpl::PathNotExist(input.to_path_buf()));
        }
        if !input.is_file() {
            return Err(DatasetErrorImpl::PathNotFile(input.to_path_buf()));
        }
        if !extract_dir.is_dir() {
            return Err(DatasetErrorImpl::PathIsNotDir(extract_dir.to_path_buf()));
        }
        if !zip_temp_dir_path.is_dir() {
            return Err(DatasetErrorImpl::PathIsNotDir(
                zip_temp_dir_path.to_path_buf(),
            ));
        }
        trace!("Start process_dataset_operations");
        let fingerprint = Self::calculate_fingerprint(input)?;
        let extract_dir_with_context = extract_dir.join(datasetkind.as_ref());
        let mut reader = EncryptedZipReader::new(
            input,
            password,
            &extract_dir_with_context,
            zip_temp_dir_path,
        )
        .map_err(|e| DatasetErrorImpl::ProcessNewEncryptedZipReader {
            source: Box::new(e),
        })?;
        trace!("Zip decrypter");
        reader.unzip().map_err(|e| DatasetErrorImpl::ProcessUnzip {
            source: Box::new(e),
        })?;
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
        Self::new_impl(file, password, target_dir, temp_zip_dir).map_err(DatasetError::from)
    }

    fn new_impl(
        file: &Path,
        password: &str,
        target_dir: &Path,
        temp_zip_dir: &Path,
    ) -> Result<Self, DatasetErrorImpl> {
        let f = File::open(file).map_err(|e| DatasetErrorImpl::IO {
            path: file.to_path_buf(),
            msg: "Opening file",
            source: e,
        })?;
        let mut buf = BufReader::new(f);
        let mut salt_buf: Vec<u8> = vec![0; SALT_BYTE_LENGTH as usize];
        let mut nonce_buf: Vec<u8> = vec![0; NONCE_BYTE_LENGTH as usize];
        let bytes_red = buf.read(&mut salt_buf).map_err(|e| DatasetErrorImpl::IO {
            path: file.to_path_buf(),
            msg: "Reading salt",
            source: e,
        })?;
        if bytes_red != SALT_BYTE_LENGTH as usize {
            return Err(DatasetErrorImpl::ByteLengthError(format!(
                "size of bytes read {bytes_red} for salt wrong. Expected: {SALT_BYTE_LENGTH}"
            )));
        }
        let salt = ByteArray::from_bytes(&salt_buf);
        let bytes_red = buf.read(&mut nonce_buf).map_err(|e| DatasetErrorImpl::IO {
            path: file.to_path_buf(),
            msg: "Reading nonce",
            source: e,
        })?;
        if bytes_red != NONCE_BYTE_LENGTH as usize {
            return Err(DatasetErrorImpl::ByteLengthError(format!(
                "size of bytes read {bytes_red} for nonce wrong. Expected: {NONCE_BYTE_LENGTH}"
            )));
        }
        let nonce = ByteArray::from_bytes(&nonce_buf);
        let derive_key = Argon2id::new_standard()
            .get_argon2id(&ByteArray::from(password), &salt)
            .unwrap();
        Ok(Self {
            internal_reader: buf,
            decrypter: Decrypter::new(&nonce, &derive_key)
                .map_err(|e| DatasetErrorImpl::Decrypter { source: e })?,
            target_dir: target_dir.to_path_buf(),
            temp_zip: Self::temp_zip_path(file, temp_zip_dir),
        })
    }

    fn decrypt_to_zip(&mut self) -> Result<PathBuf, DatasetErrorImpl> {
        let mut target =
            std::fs::File::create(&self.temp_zip).map_err(|e| DatasetErrorImpl::IO {
                path: self.temp_zip.clone(),
                msg: "Creating Temp Zip",
                source: e,
            })?;
        let buf = &mut self.internal_reader;
        loop {
            let mut temp_buffer = vec![0; ENCRYPTED_BLOCK_SIZE];
            let count = buf
                .read(&mut temp_buffer)
                .map_err(|e| DatasetErrorImpl::IOBuf {
                    msg: "Reading buffer",
                    source: e,
                })?;
            if count == 0 {
                break;
            }
            temp_buffer.truncate(count);
            let plaintext = self
                .decrypter
                .decrypt(&ByteArray::from_bytes(&temp_buffer))
                .map_err(|e| DatasetErrorImpl::Decrypt { source: e })?;
            target
                .write_all(plaintext.to_bytes())
                .map_err(|e| DatasetErrorImpl::IOBuf {
                    msg: "Writing temp zip",
                    source: e,
                })?;
        }
        Ok(self.temp_zip.to_owned())
    }

    /// Decrypt and unzip the source file
    ///
    /// The method return the target directory
    pub fn unzip(&mut self) -> Result<PathBuf, DatasetError> {
        self.unzip_impl().map_err(DatasetError::from)
    }

    fn unzip_impl(&mut self) -> Result<PathBuf, DatasetErrorImpl> {
        if !self.temp_zip.exists() {
            self.decrypt_to_zip()?;
        }
        let f = std::fs::File::open(&self.temp_zip).map_err(|e| DatasetErrorImpl::IO {
            path: self.temp_zip.clone(),
            msg: "Opening temp zip file {}",
            source: e,
        })?;
        zip_extract::extract(&f, &self.target_dir, true).map_err(|e| {
            DatasetErrorImpl::Extract {
                file: self.temp_zip.to_path_buf(),
                source: e,
            }
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
