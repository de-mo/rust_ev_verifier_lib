use super::GetFileNameTrait;
use crate::data_structures::{VerifierData, VerifierDataType};
use anyhow::anyhow;
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
        if name.contains('*') {
            let r_entries = glob(path.as_os_str().to_str().unwrap());
            if let Ok(entries) = r_entries {
                let p = entries.last();
                if let Some(p_f) = p {
                    path = location.join(p_f.unwrap().file_name().unwrap());
                };
            }
        }
        File {
            path,
            data_type: data_type.clone(),
        }
    }

    #[allow(dead_code)]
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

    pub fn read_data(&self) -> anyhow::Result<String> {
        fs::read_to_string(self.get_path())
            .map_err(|e| anyhow!(e).context(format!("Cannot read file \"{}\"", self.to_str())))
    }

    pub fn get_data(&self) -> anyhow::Result<VerifierData> {
        if !self.exists() {
            return Err(anyhow!(format!(
                "File \"{}\" does not exists",
                self.to_str()
            )));
        }
        self.data_type.verifier_data_from_file(self).map_err(|e| {
            anyhow!(e).context(format!(
                "Content of the file \"{}\" is not valid",
                self.to_str()
            ))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{test_dataset_setup_path, test_dataset_tally_path};
    use crate::data_structures::{
        setup::VerifierSetupDataType, tally::VerifierTallyDataType, VerifierDataType,
        VerifierSetupDataTrait,
    };
    use std::path::PathBuf;

    fn get_location() -> PathBuf {
        test_dataset_setup_path().join("setup")
    }

    #[test]
    fn test_file() {
        let location = get_location();
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::ElectionEventContextPayload),
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
        let enc_data = data.election_event_context_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_macro() {
        let location = get_location();
        let f = create_file!(
            &location,
            Setup,
            VerifierSetupDataType::ElectionEventContextPayload
        );
        assert!(f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("encryptionParametersPayload.json")
        );
        let data = f.get_data().unwrap();
        assert!(data.is_setup());
        let enc_data = data.election_event_context_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_not_exist() {
        let location = get_location().join("toto");
        let f = File::new(
            &location,
            &VerifierDataType::Setup(VerifierSetupDataType::ElectionEventContextPayload),
            None,
        );
        assert!(!f.exists());
        assert_eq!(f.get_location(), location);
        assert_eq!(
            f.get_path(),
            location.join("electionEventContextPayload.json")
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
        let location = test_dataset_tally_path().join("tally");
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
