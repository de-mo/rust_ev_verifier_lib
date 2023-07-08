// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod directory;
mod verifications;

use lazy_static::lazy_static;
use log::{info, LevelFilter};
use rust_verifier_application::{init_logger, VerifierConfig};

lazy_static! {
    pub(crate) static ref CONFIG: VerifierConfig = {
        if cfg!(debug_assertions) {
            VerifierConfig::new("../..")
        } else {
            VerifierConfig::new(".")
        }
    };
}

fn main() {
    init_logger(&CONFIG, LevelFilter::Debug, cfg!(debug_assertions));
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .plugin(directory::init())
        .plugin(verifications::init())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| match event {
            tauri::RunEvent::Exit => info!("Verifier GUI closed"),
            tauri::RunEvent::Ready => info!("Start Verifier GUI"),
            _ => (),
        });
}
