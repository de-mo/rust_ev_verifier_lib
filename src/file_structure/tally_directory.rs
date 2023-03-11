use std::path::{Path, PathBuf};

pub struct TallyDirectory {
    location: PathBuf,
}

impl TallyDirectory {
    pub fn new(data_location: &Path) -> TallyDirectory {
        let location = data_location.join("tally");
        TallyDirectory {
            location: location.to_path_buf(),
        }
    }
}
