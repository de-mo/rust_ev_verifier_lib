use anyhow::{anyhow, ensure, Context};
use chrono::prelude::*;
use enum_kinds::EnumKind;
use rust_ev_crypto_primitives::{sha256_stream, Argon2id, ByteArray, Decrypter};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};
use strum::{AsRefStr, EnumString};

/// Metadata containing the information of the zip dataset before and after extraction
#[derive(Debug, Clone)]
pub struct DatasetMetadata {
    pub source_path: PathBuf,
    pub decrypted_zip_path: PathBuf,
    pub extracted_dir_path: PathBuf,
    pub fingerprint: ByteArray,
}

/// Datasettype containing the information about metadata
#[derive(Debug, Clone, AsRefStr, EnumKind)]
#[enum_kind(DatasetTypeKind, derive(AsRefStr, EnumString))]
pub enum DatasetType {
    Context(DatasetMetadata),
    Setup(DatasetMetadata),
    Tally(DatasetMetadata),
}

impl DatasetMetadata {
    /// New [DatasetMetadata]
    pub fn new(
        source_path: &Path,
        decrypted_zip_path: &Path,
        extracted_dir_path: &Path,
        fingerprint: &ByteArray,
    ) -> Self {
        Self {
            source_path: source_path.to_path_buf(),
            decrypted_zip_path: decrypted_zip_path.to_path_buf(),
            extracted_dir_path: extracted_dir_path.to_path_buf(),
            fingerprint: fingerprint.clone(),
        }
    }
}

impl DatasetType {
    pub fn metadata(&self) -> &DatasetMetadata {
        match self {
            DatasetType::Context(m) => m,
            DatasetType::Setup(m) => m,
            DatasetType::Tally(m) => m,
        }
    }

    pub fn get_from_context_str_with_inputs(
        datasettype_str: &str,
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> anyhow::Result<Self> {
        match DatasetTypeKind::try_from(datasettype_str)? {
            DatasetTypeKind::Context => {
                Self::context(input, password, extract_dir, zip_temp_dir_path)
            }
            DatasetTypeKind::Setup => Self::setup(input, password, extract_dir, zip_temp_dir_path),
            DatasetTypeKind::Tally => Self::tally(input, password, extract_dir, zip_temp_dir_path),
        }
    }

    /// Extract the data as context datatype.
    ///
    /// Return [DatasetType] with the correct metadata or Error if something goes wrong
    pub fn context(
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> anyhow::Result<Self> {
        Ok(Self::Context(Self::process_dataset_operations(
            input,
            password,
            extract_dir,
            "context",
            zip_temp_dir_path,
        )?))
    }

    /// Extract the data as setup datatype.
    ///
    /// Return [DatasetType] with the correct metadata or Error if something goes wrong
    pub fn setup(
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> anyhow::Result<Self> {
        Ok(Self::Setup(Self::process_dataset_operations(
            input,
            password,
            extract_dir,
            "setup",
            zip_temp_dir_path,
        )?))
    }

    /// Extract the data as tally datatype.
    ///
    /// Return [DatasetType] with the correct metadata or Error if something goes wrong
    pub fn tally(
        input: &Path,
        password: &str,
        extract_dir: &Path,
        zip_temp_dir_path: &Path,
    ) -> anyhow::Result<Self> {
        Ok(Self::Tally(Self::process_dataset_operations(
            input,
            password,
            extract_dir,
            "tally",
            zip_temp_dir_path,
        )?))
    }

    fn fingerprint(input: &Path) -> anyhow::Result<ByteArray> {
        let f =
            std::fs::File::open(input).context("Opening file for calculation of fingerprint")?;
        let mut reader = std::io::BufReader::new(f);
        sha256_stream(&mut reader).context("Calculating fingerprint")
    }

    fn process_dataset_operations(
        input: &Path,
        password: &str,
        extract_dir: &Path,
        datasettype_str: &str,
        zip_temp_dir_path: &Path,
    ) -> anyhow::Result<DatasetMetadata> {
        ensure!(input.exists(), "Input file does not exist");
        ensure!(input.is_file(), "Input is not a file");
        ensure!(
            extract_dir.is_dir(),
            "The directory where the zip should be extracted does not exist"
        );
        ensure!(
            zip_temp_dir_path.is_dir(),
            "The directory where the zip should be decrypted does not exist"
        );
        let fingerprint = Self::fingerprint(input)?;
        let extract_dir_with_context = extract_dir.join(datasettype_str);
        let mut reader = EncryptedZipReader::new(
            input,
            password,
            &extract_dir_with_context,
            zip_temp_dir_path,
        )?;
        reader.unzip()?;
        Ok(DatasetMetadata::new(
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
    ) -> anyhow::Result<Self> {
        let f = File::open(file).context(format!(
            "File {}",
            file.file_name().unwrap().to_str().unwrap()
        ))?;
        let mut buf = BufReader::new(f);
        let mut salt_buf: Vec<u8> = vec![0; SALT_BYTE_LENGTH as usize];
        let mut nonce_buf: Vec<u8> = vec![0; NONCE_BYTE_LENGTH as usize];
        let bytes_red = buf.read(&mut salt_buf).context("Reading salt")?;
        if bytes_red != SALT_BYTE_LENGTH as usize {
            return Err(anyhow!("Number of bytes red not correct"));
        }
        let salt = ByteArray::from_bytes(&salt_buf);
        let bytes_red = buf.read(&mut nonce_buf).context("Reading nonce")?;
        if bytes_red != NONCE_BYTE_LENGTH as usize {
            return Err(anyhow!("Number of bytes red not correct"));
        }
        let nonce = ByteArray::from_bytes(&nonce_buf);
        let derive_key = Argon2id::new_standard()
            .get_argon2id(&ByteArray::from(password), &salt)
            .unwrap();
        Ok(Self {
            internal_reader: buf,
            decrypter: Decrypter::new(&nonce, &derive_key).context("Creating decrypter")?,
            target_dir: target_dir.to_path_buf(),
            temp_zip: Self::temp_zip_path(file, temp_zip_dir),
        })
    }

    fn decrypt_to_zip(&mut self) -> anyhow::Result<PathBuf> {
        let mut target = std::fs::File::create(&self.temp_zip).context("Creating Temp Zip")?;
        let buf = &mut self.internal_reader;
        loop {
            let mut temp_buffer = vec![0; ENCRYPTED_BLOCK_SIZE];
            let count = buf.read(&mut temp_buffer).context("Reading buffer")?;
            if count == 0 {
                break;
            }
            temp_buffer.truncate(count);
            let plaintext = self
                .decrypter
                .decrypt(&ByteArray::from_bytes(&temp_buffer))
                .context("Decrypting cipher")?;
            target
                .write_all(&plaintext.to_bytes())
                .context("Writing temp zip")?;
        }
        Ok(self.temp_zip.to_owned())
    }

    /// Decrypt and unzip the source file
    ///
    /// The method return the target directory
    pub fn unzip(&mut self) -> anyhow::Result<PathBuf> {
        if !self.temp_zip.exists() {
            self.decrypt_to_zip()?;
        }
        let f = std::fs::File::open(&self.temp_zip).context("Opening temp zip file")?;
        zip_extract::extract(&f, &self.target_dir, true).context("Error unzipping")?;
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
    use std::ffi::OsString;

    use super::*;
    use crate::config::test::{test_datasets_path, test_temp_dir_path, CONFIG_TEST};

    #[test]
    fn test_temp_zip_path() {
        let f = PathBuf::from("./toto/12345.zip");
        let res = EncryptedZipReader::temp_zip_path(&f, &PathBuf::from("/abc/xyz"));
        assert_eq!(res.extension(), Some(OsString::from("zip").as_os_str()));
        assert_eq!(res.parent(), Some(PathBuf::from("/abc/xyz").as_path()));
    }

    #[test]
    fn test_decrypt_zip() {
        let path = test_datasets_path().join("Dataset-context-NE_20231124_TT05-20240802_1158.zip");
        let mut zip_reader = EncryptedZipReader::new(
            &path,
            "LongPassword_Encryption1",
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
        let path = test_datasets_path().join("Dataset-context-NE_20231124_TT05-20240802_1158.zip");
        let subdir_time =
            test_temp_dir_path().join(Local::now().format("%Y%m%d-%H%M%S").to_string());
        let _ = std::fs::create_dir(&subdir_time);
        let target_dir = subdir_time.join("context");
        let mut zip_reader = EncryptedZipReader::new(
            &path,
            "LongPassword_Encryption1",
            &target_dir,
            &CONFIG_TEST.zip_temp_dir_path(),
        )
        .unwrap();
        let unzip_res = zip_reader.unzip();
        let path_res = unzip_res.unwrap();
        assert_eq!(path_res, target_dir)
    }
}
