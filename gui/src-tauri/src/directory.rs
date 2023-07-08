use log::{error, info};
use rust_verifier_application::is_directory_tally;
use std::path::Path;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[derive(Clone, serde::Serialize)]
struct DirectoryDataPayload {
    path: String,
    is_tally: bool,
}

impl DirectoryDataPayload {
    fn new(path: &Path) -> Result<Self, String> {
        match is_directory_tally(path) {
            Ok(b) => {
                info!("Directory loaded: {}", path.to_str().unwrap());
                Ok(DirectoryDataPayload {
                    path: path.as_os_str().to_str().unwrap().to_string(),
                    is_tally: b,
                })
            }
            Err(e) => {
                error!(
                    "Error loading the directory {}: {}",
                    path.to_str().unwrap(),
                    e
                );
                Err(e.to_string())
            }
        }
    }
}

#[tauri::command]
async fn choose_directory() -> Result<Option<DirectoryDataPayload>, String> {
    match FileDialogBuilder::new().pick_folder() {
        Some(d) => DirectoryDataPayload::new(&d).map(Some),
        None => Ok(None),
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("directory")
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![choose_directory])
        .build()
}
