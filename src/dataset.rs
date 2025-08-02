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
use rust_ev_system_library::{
    chanel_security::stream::{get_stream_plaintext, StreamSymEncryptionError},
    rust_ev_crypto_primitives::prelude::{
        basic_crypto_functions::{sha256_stream, BasisCryptoError},
        ByteArray, EncodeTrait,
    },
};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::{instrument, trace};

#[derive(Error, Debug)]
#[error(transparent)]
/// Error with dataset
pub struct DatasetError(#[from] Box<DatasetErrorImpl>);

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
    #[error("Error extracting {file}")]
    Extract {
        file: PathBuf,
        source: zip_extract::ZipExtractError,
    },
    #[error("process_dataset_operations: Error crreating EncryptedZipReader")]
    ProcessNewEncryptedZipReader { source: Box<DatasetError> },
    #[error("process_dataset_operations: Error unzipping")]
    ProcessUnzip { source: Box<DatasetError> },
    #[error("Error streaming the file {path}")]
    GetStreamPlaintext {
        path: PathBuf,
        source: StreamSymEncryptionError,
    },
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
                .map_err(|_| DatasetErrorImpl::WrongKindStr(datasettype_str.to_string()))
                .map_err(|e| DatasetError(Box::new(e)))?,
            input,
            password,
            extract_dir,
            zip_temp_dir_path,
        )
    }

    fn calculate_fingerprint(input: &Path) -> Result<ByteArray, Box<DatasetErrorImpl>> {
        let f = std::fs::File::open(input).map_err(|e| DatasetErrorImpl::IOFingerprint {
            path: input.to_path_buf(),
            source: e,
        })?;
        let mut reader = std::io::BufReader::new(f);
        sha256_stream(&mut reader).map_err(|e| {
            Box::new(DatasetErrorImpl::Fingerprint {
                path: input.to_path_buf(),
                source: e,
            })
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
        .map_err(DatasetError)
    }

    fn process_dataset_operations_impl(
        datasetkind: DatasetTypeKind,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> Result<Self, Box<DatasetErrorImpl>> {
        if !input.exists() {
            return Err(Box::new(DatasetErrorImpl::PathNotExist(
                input.to_path_buf(),
            )));
        }
        if !input.is_file() {
            return Err(Box::new(DatasetErrorImpl::PathNotFile(input.to_path_buf())));
        }
        if !extract_dir.is_dir() {
            return Err(Box::new(DatasetErrorImpl::PathIsNotDir(
                extract_dir.to_path_buf(),
            )));
        }
        if !zip_temp_dir_path.is_dir() {
            return Err(Box::new(DatasetErrorImpl::PathIsNotDir(
                zip_temp_dir_path.to_path_buf(),
            )));
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

/// Structure to decrypt the zip file and to extract the files
///
/// The zip will be first encrypted in the given location `target_dir`. The filename will extend with the word `encryption` and
/// the actual date time
///
/// In a second step, the extraction is done in the target directory (with strip away the topmost directory)
pub struct EncryptedZipReader {
    internal_reader: BufReader<File>,
    password: String,
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
        Self::new_impl(file, password, target_dir, temp_zip_dir).map_err(DatasetError)
    }

    fn new_impl(
        file: &Path,
        password: &str,
        target_dir: &Path,
        temp_zip_dir: &Path,
    ) -> Result<Self, Box<DatasetErrorImpl>> {
        let f = File::open(file).map_err(|e| DatasetErrorImpl::IO {
            path: file.to_path_buf(),
            msg: "Opening file",
            source: e,
        })?;
        let buf = BufReader::new(f);
        Ok(Self {
            internal_reader: buf,
            password: password.to_string(),
            target_dir: target_dir.to_path_buf(),
            temp_zip: Self::temp_zip_path(file, temp_zip_dir),
        })
    }

    fn decrypt_to_zip(&mut self) -> Result<PathBuf, Box<DatasetErrorImpl>> {
        let target = std::fs::File::create(&self.temp_zip).map_err(|e| DatasetErrorImpl::IO {
            path: self.temp_zip.clone(),
            msg: "Creating Temp Zip",
            source: e,
        })?;
        let mut target_writer = BufWriter::new(target);
        get_stream_plaintext(
            &mut self.internal_reader,
            &self.password,
            &ByteArray::default(),
            &mut target_writer,
        )
        .map_err(|e| DatasetErrorImpl::GetStreamPlaintext {
            path: self.temp_zip.clone(),
            source: e,
        })?;
        Ok(self.temp_zip.to_owned())
    }

    /// Decrypt and unzip the source file
    ///
    /// The method return the target directory
    pub fn unzip(&mut self) -> Result<PathBuf, DatasetError> {
        self.unzip_impl().map_err(DatasetError)
    }

    fn unzip_impl(&mut self) -> Result<PathBuf, Box<DatasetErrorImpl>> {
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
