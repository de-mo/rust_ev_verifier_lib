use super::RunnerError;
use rust_ev_verifier_lib::{
    dataset::DatasetMetadata, verification::VerificationPeriod, DatasetTypeKind, VerifierConfig,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::{info, instrument};

#[derive(Debug, Clone)]
pub struct ExtractDataSetResults {
    metadata_hm: HashMap<DatasetTypeKind, DatasetMetadata>,
    location: PathBuf,
}

impl ExtractDataSetResults {
    pub fn location(&self) -> &Path {
        &self.location
    }

    pub fn dataset_metadata(&self, kind: &DatasetTypeKind) -> Option<&DatasetMetadata> {
        self.metadata_hm.get(kind)
    }

    #[instrument(skip(password, config))]
    pub fn extract_datasets(
        period: VerificationPeriod,
        context_zip_file: &Path,
        setup_zip_file: Option<&Path>,
        tally_zip_file: Option<&Path>,
        password: &str,
        config: &'static VerifierConfig,
    ) -> Result<Self, RunnerError> {
        let dataset_root_path = config.create_dataset_dir_path();
        let mut hm = HashMap::new();
        match period {
            VerificationPeriod::Setup => {
                if setup_zip_file.is_none() {
                    return Err(RunnerError::FileMissing(
                        "For setup, the dataset for setup must be delivered".to_string(),
                    ));
                } else {
                    let md = DatasetMetadata::extract_dataset_kind_with_inputs(
                        DatasetTypeKind::Setup,
                        setup_zip_file.unwrap(),
                        password,
                        &dataset_root_path,
                        &config.zip_temp_dir_path(),
                    )
                    .map_err(|e| RunnerError::Dataset {
                        msg: "Extracting setup".to_string(),
                        source: e,
                    })?;
                    info!(
                        "Setup extracted by {} (fingerprint: {}",
                        md.extracted_dir_path().to_str().unwrap(),
                        md.fingerprint_str()
                    );
                    hm.insert(DatasetTypeKind::Setup, md);
                }
            }
            VerificationPeriod::Tally => {
                if tally_zip_file.is_none() {
                    return Err(RunnerError::FileMissing(
                        "For tally, the dataset for tally must be delivered".to_string(),
                    ));
                } else {
                    let md = DatasetMetadata::extract_dataset_kind_with_inputs(
                        DatasetTypeKind::Tally,
                        tally_zip_file.unwrap(),
                        password,
                        &dataset_root_path,
                        &config.zip_temp_dir_path(),
                    )
                    .map_err(|e| RunnerError::Dataset {
                        msg: "Extracting tally".to_string(),
                        source: e,
                    })?;
                    info!(
                        "Tally extracted by {} (fingerprint: {}",
                        md.extracted_dir_path().to_str().unwrap(),
                        md.fingerprint_str()
                    );
                    hm.insert(DatasetTypeKind::Tally, md);
                }
            }
        }
        let md = DatasetMetadata::extract_dataset_kind_with_inputs(
            DatasetTypeKind::Context,
            context_zip_file,
            password,
            &dataset_root_path,
            &config.zip_temp_dir_path(),
        )
        .map_err(|e| RunnerError::Dataset {
            msg: "Extracting context".to_string(),
            source: e,
        })?;
        info!(
            "Context extracted by {} (fingerprint: {}",
            md.extracted_dir_path().to_str().unwrap(),
            md.fingerprint_str()
        );
        hm.insert(DatasetTypeKind::Context, md);
        Ok(Self {
            metadata_hm: hm,
            location: dataset_root_path,
        })
    }
}
