use std::path::{Path, PathBuf};

use crate::{
    data_structures::DatasetType as VariantDatasetType, file_structure::zip::EncryptedZipReader,
};
use anyhow::bail;

pub type DatasetType = VariantDatasetType<bool, bool, bool>;

impl TryFrom<&str> for DatasetType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "context" => Ok(DatasetType::Context(true)),
            "setup" => Ok(DatasetType::Setup(true)),
            "tally" => Ok(DatasetType::Tally(true)),
            _ => bail!("Uncorrect value"),
        }
    }
}

pub fn read_and_extract(
    input: &Path,
    password: &str,
    current_dir: &Path,
    dataset_type: DatasetType,
    zip_temp_dir_path: &Path,
) -> anyhow::Result<PathBuf> {
    let target_dir_with_context = current_dir.join(dataset_type.as_ref());
    let mut reader =
        EncryptedZipReader::new(input, password, &target_dir_with_context, zip_temp_dir_path)?;
    reader.unzip()?;
    Ok(target_dir_with_context)
}
