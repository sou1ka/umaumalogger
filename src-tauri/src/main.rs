#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::io;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::collections::HashMap;
use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn filelogging(filename: &str) -> String {
    format!("Filename {} is enabled.", filename)
}

#[tauri::command]
fn get_filelog_lastline(runtime_str: &str, filename: &str) -> String {
    let path = &format!("{}\\out\\{}", get_currentpath(), filename);
    let meta = fs::metadata(path).unwrap();
    let lastmodify: u64 = meta.last_write_time();
    let runtime: u64 = runtime_str.parse().unwrap();
    let mut ret = String::from("");

    if lastmodify > runtime {
        let dat = fs::read_to_string(path).unwrap();
        let dats:Vec<&str> = dat.trim().split("\n").collect();
        ret = dats[dats.len()-1].to_string();println!("{}", ret);
    }

    format!("{}", ret)
}

#[tauri::command]
fn get_loglists() -> String {
    let path = &format!("{}\\out\\", get_currentpath());
    let lists: Vec<String> = read_dir(path).unwrap();
    let mut arr: Vec<String> = Vec::new();

    for item in lists {
        let meta = fs::metadata(path.to_string() + &item).unwrap();
        let mut line: HashMap<&str, String> = HashMap::new();
        line.insert("filename", item);
        line.insert("create_timestamp", (meta.creation_time() / 10000000 - 11644473600).to_string());
        let serialized: String = serde_json::to_string(&line).unwrap();
        arr.push(serialized);
    }

    format!("[{}]", arr.join(","))
}

#[tauri::command]
fn get_path() -> String {
    format!("{}", get_currentpath())
}

#[tauri::command]
fn get_filelog(filename: &str) -> String {
    let path = &format!("{}\\out\\{}", get_currentpath(), filename);
    let dat = fs::read_to_string(path).unwrap();
    format!("{}", dat)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            filelogging,
            get_filelog_lastline,
            get_path,
            get_loglists,
            get_filelog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() {
                Some(entry.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect())
}

fn get_currentpath() -> String {
    let cd = env::current_dir().unwrap();

    return cd.to_string_lossy().to_string();
}
