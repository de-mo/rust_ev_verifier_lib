// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod directory;
mod verifications;

use lazy_static::lazy_static;
use rust_verifier_lib::config::Config;

lazy_static! {
    pub(crate) static ref CONFIG: Config = {
        if cfg!(debug_assertions) {
            Config::new("../..")
        } else {
            Config::new(".")
        }
    };
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .plugin(directory::init())
        .plugin(verifications::init())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {})
}
