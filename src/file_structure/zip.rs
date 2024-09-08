use crate::config;
use anyhow::{anyhow, Context};
use chrono::prelude::*;
use rust_ev_crypto_primitives::{Argon2id, ByteArray, Decrypter};
use std::{
    ffi::OsString,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

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
        new_name.push("-decrypted-");
        new_name.push(now);
        new_name.push(".");
        new_name.push(source.extension().unwrap());
        temp_zip_dir.join(new_name)
    }
}

#[cfg(test)]
mod test {
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
        std::fs::create_dir(&subdir_time);
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
