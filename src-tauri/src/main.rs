// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::metronome::{Metronome, MetronomeConfig, TimeSignature};
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use log;
use tauri::AppHandle;
mod metronome;

static METRONOME_INSTANCE: Lazy<Mutex<Option<Metronome>>> = Lazy::new(|| Mutex::new(None));
static CONFIG: Lazy<Mutex<Option<Arc<Mutex<MetronomeConfig>>>>> =
    Lazy::new(|| Mutex::new(None));

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![start_metronome, stop_metronome, update_metronome])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

#[tauri::command]
fn start_metronome(app: AppHandle, bpm: u32, time_signature: TimeSignature, volume: f64) {
    let mut lock = METRONOME_INSTANCE.lock().unwrap();

    let config = Arc::new(Mutex::new(metronome::MetronomeConfig {
        bpm: bpm as f32,
        time_signature,
        volume: volume as f32,
    }));

    if let Some(old) = lock.take() {
        old.stop();
    }

    match metronome::Metronome::start(Arc::clone(&config), app) {
        Ok((metro, _, _, shared_config)) => {
            *lock = Some(metro);
            *CONFIG.lock().unwrap() = Some(shared_config);
        }
        Err(e) => {
            log::error!("Failed to start metronome: {:?}", e);
        }
    }
}

#[tauri::command]
fn stop_metronome() {
    let mut lock = METRONOME_INSTANCE.lock().unwrap();
    if let Some(metro) = lock.take() {
        metro.stop();
        log::info!("Stopped metronome");
    }
}

#[tauri::command]
fn update_metronome(bpm: u32, time_signature: TimeSignature, volume: f64) {
    if let Some(config) = CONFIG.lock().unwrap().as_ref() {
        metronome::update_config(config, bpm as f32, time_signature, volume as f32);
    }
}