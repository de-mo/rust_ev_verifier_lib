use super::{FileStructureError, FileStructureErrorType, GetFileNameTrait};
use crate::data_structures::{VerifierData, VerifierDataType};
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct File {
    path: PathBuf,
    data_type: VerifierDataType,
}

macro_rules! create_file {
    ($l: expr, $p: ident, $s: expr) => {
        File::new(&$l, &VerifierDataType::$p($s), None)
    };
    ($l: expr, $p: ident, $s: expr, $n: expr) => {
        File::new(&$l, &VerifierDataType::$p($s), Some($n))
    };
}
pub(crate) use create_file;

impl File {
    pub fn new(location: &Path, data_type: &VerifierDataType, file_nb: Option<usize>) -> Self {
        let name = data_type.get_file_name(file_nb);
        let mut path = location.join(&name);
        if (&name).contains("*") {
            let entries = glob(path.as_os_str().to_str().unwrap());
            if entries.is_ok() {
                let p = entries.unwrap().last();
                if p.is_some() {
                    path = location.join(&p.unwrap().unwrap().file_name().unwrap());
                };
            }
        }
        File {
            path,
            data_type: data_type.clone(),
        }
    }

    pub fn get_location(&self) -> PathBuf {
        self.path.parent().unwrap().to_path_buf()
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub fn to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub fn read_data(&self) -> Result<String, FileStructureError> {
        fs::read_to_string(&self.get_path()).map_err(|e| {
            create_verifier_error!(
                FileStructureErrorType::FileError,
                format!("Cannot read file \"{}\"", self.to_str()),
                e
            )
        })
    }

    pub fn get_data(&self) -> Result<VerifierData, FileStructureError> {
        if !self.exists() {
            return create_result_with_error!(
                FileStructureErrorType::FileError,
                format!("File \"{}\" does not exists", self.to_str())
            );
        }
        self.data_type.verifier_data_from_file(&self).map_err(|e| {
            create_verifier_error!(
                FileStructureErrorType::DataError,
                format!("Content of the file \"{}\" is not valid", self.to_str()),
                e
            )
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data_structures::{
        setup::VerifierSetupDataType, tally::VerifierTallyDataType, VerifierDataType,
        VerifierSetupDataTrait,
    };
    use std::path::{Path, PathBuf};

    fn get_location() -> PathBuf {
        Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
    }

    #[test]
    fn test_file() {
        let location = get_location();
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::EncryptionParametersPayload),
            None,
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("encryptionParametersPayload.json")
        );
        let data = f.get_data().unwrap();
        assert!(data.is_setup());
        let enc_data = data.encryption_parameters_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_macro() {
        let location = get_location();
        let f = create_file!(
            &location,
            Setup,
            VerifierSetupDataType::EncryptionParametersPayload
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("encryptionParametersPayload.json")
        );
        let data = f.get_data().unwrap();
        assert!(data.is_setup());
        let enc_data = data.encryption_parameters_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_not_exist() {
        let location = get_location().join("toto");
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::EncryptionParametersPayload),
            None,
        );
        assert!(!f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("encryptionParametersPayload.json")
        );
        let data = f.get_data();
        assert!(data.is_err())
    }

    #[test]
    fn test_file_with_nb() {
        let location = get_location();
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
            Some(2),
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("controlComponentPublicKeysPayload.2.json")
        );
        let data = f.get_data().unwrap();
        assert!(data.is_setup());
    }

    #[test]
    fn test_file_with_astrerix() {
        let location = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally");
        let f = File::new(
            &location,
            &VerifierDataType::Tally(VerifierTallyDataType::ECH0110),
            None,
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(f.get_path(), location.join("eCH-0110_Post_E2E_DEV.xml"));
    }

    #[test]
    fn test_file_with_nb_not_exist() {
        let location = get_location();
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
            Some(6),
        );
        assert!(!f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("controlComponentPublicKeysPayload.6.json")
        );
        let data = f.get_data();
        assert!(data.is_err());
    }

    #[test]
    fn test_file_with_nb_macro() {
        let location = get_location();
        let f = create_file!(
            &location,
            Setup,
            VerifierSetupDataType::ControlComponentPublicKeysPayload,
            2
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("controlComponentPublicKeysPayload.2.json")
        );
        let data = f.get_data().unwrap();
        assert!(data.is_setup());
    }
}
