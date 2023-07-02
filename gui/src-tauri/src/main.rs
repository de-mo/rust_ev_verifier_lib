// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod directory;
mod verifications;

use std::{env::current_dir, path::PathBuf};

pub(crate) fn root_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    } else {
        current_dir().unwrap()
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![greet])
        .plugin(directory::init())
        .plugin(verifications::init())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {})
}
