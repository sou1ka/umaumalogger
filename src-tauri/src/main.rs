#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::api::dialog;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

use std::env;
use std::io;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::collections::HashMap;
use chrono::Utc;
use image::imageops::FilterType;
use image::ImageOutputFormat;
use std::io::Cursor;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn filelogging(filename: &str) -> String {
    format!("Filename {} is enabled.", filename)
}

#[tauri::command]
fn get_filelog_lastline(logno_str: &str, filename: &str) -> String {
    let path = &format!("{}\\out\\{}", get_currentpath(), filename);
    let mut ret = String::from("");
    let dat = fs::read_to_string(path).expect("not read file.");
    let dats:Vec<&str> = dat.trim().lines().collect();
    let linecount = dats.len();
    let logno:usize = logno_str.parse().unwrap();//println!("linecount {}, logno {}", linecount, logno);
    ret += &linecount.to_string();
    ret += "\n";
    if logno < linecount {
        for n in logno..linecount {
            ret += &dats[n].to_string();
            ret += "\n";
        }
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

#[tauri::command]
fn take_screenshot() -> String {
    let device = dxcapture::Device::new_from_window("umamusume".to_string());

    match device {
        Ok(d) => {
        //    let d = dxcapture::Device::new_from_window("umamusume".to_string()).expect("There is no 'umamusume' window.");
            let capture = dxcapture::Capture::new(&d).unwrap();
            let image = capture.wait_img_frame().expect("Failed to capture");
            let now = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let path = &format!("{}\\screenshot\\{}.png", get_currentpath(), now);
            let out = image.data.save(path);

            match out {
                Ok(o) => {
                    format!("{}.png", now)
                },
                Err(e) => {
                    format!("{}", e)
                }
            }
           
        },
        Err(e) => format!("There is no 'umamusume' window.")
    }    
}

#[tauri::command]
fn get_imagelist() -> String {
    let path: &str = &format!("{}\\screenshot\\", get_currentpath());
    let lists: Vec<String> = read_dir(path).unwrap();
    let mut arr: Vec<String> = Vec::new();

    for item in lists {
        let mut line: HashMap<&str, String> = HashMap::new();
        line.insert("filename", (&item).to_string());
//        line.insert("base64", get_imgbase64(&format!("{}{}", &path, &item)));
        let serialized: String = serde_json::to_string(&line).unwrap();
        arr.push(serialized);
    }

    format!("[{}]", arr.join(","))
}

fn main() {
    let exit = CustomMenuItem::new("exit".to_string(), "Exit");
    let version = CustomMenuItem::new("vers".to_string(), "Version");
    let filemenu = Submenu::new("File", Menu::new().add_item(exit));
    let helpmenu = Submenu::new("Help", Menu::new().add_item(version));
    let menu = Menu::new()
        .add_submenu(filemenu)
        .add_submenu(helpmenu);
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            filelogging,
            get_filelog_lastline,
            get_path,
            get_loglists,
            get_filelog,
            take_screenshot,
            get_imagelist
        ])
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "exit" => {
                    std::process::exit(0);
                }
                "vers" => {
                    dialog::message(Some(&event.window()), "UmaUmaLogger", "Version 0.2.4.20221123\n\rAuthor: sou1ka @sou1ka");
                }
                _ => {}
            }
          })
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

fn get_imgbase64(path: &str) -> String {
    let img = image::open(&path).expect(&path);
    let nwidth: u32 = 90; // thumbnail width
    let nheight: u32 = img.height();

    let resized_img = img.resize(nwidth, nheight, FilterType::Lanczos3);
    let mut buf: Vec<u8> = Vec::new();
    resized_img.write_to(&mut Cursor::new(&mut buf), ImageOutputFormat::Png).unwrap();
    let res_base64 = base64::encode(buf);
    
    return format!("data:image/png;base64,{}", res_base64);
}