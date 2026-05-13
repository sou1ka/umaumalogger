#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::api::{dialog, shell};
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent, WindowMenuEvent, Window};

use std::env;
use std::io;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use notify::{Watcher, RecursiveMode};
use chrono::Utc;

mod screenshot;

struct LogWatchState {
    watcher: Option<notify::RecommendedWatcher>,
    logno: Arc<Mutex<usize>>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn filelogging(filename: &str) -> String {
    format!("Filename {} is enabled.", filename)
}

#[tauri::command]
fn get_filelog_lastline(logno_str: &str, filename: &str) -> String {
    let path = &format!("{}\\out\\{}", get_currentpath(), filename);
    let mut ret = String::from("");

    if Path::new(&path).exists() {
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
    let now = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let path = &format!("{}\\screenshot\\{}.png", get_currentpath(), now);
    let res = screenshot::capture(&path);

    if res {
        format!("{}.png", now)
    } else {
        format!("")
    }
}

#[tauri::command]
fn get_imagelist(page: usize, num: usize) -> String {
    let path: &str = &format!("{}\\screenshot\\", get_currentpath());
    let mut lists: Vec<String> = read_dir(path).unwrap();
    lists.reverse();
    let size = lists.len();
    let mut arr: Vec<String> = Vec::new();
    let base: usize = 20;
    let limit: usize = if num == 0 { page * base } else { num };
    let start: usize = if num == 0 { limit - base } else { 0 };

    for i in start..limit {
        if size <= i.into() { break; }

        let item = &lists[i];
        let mut line: HashMap<&str, String> = HashMap::new();
        line.insert("filename", (&item).to_string());
        let serialized: String = serde_json::to_string(&line).unwrap();
        arr.push(serialized);
    }

    format!("[{}]", arr.join(","))
}

#[tauri::command]
fn get_eventvalue(musumename: &str, eventname: &str, force: bool) -> String {
    let path = env::temp_dir().to_str().unwrap().to_string() + "umalog\\scr_ikusei_event.txt";
    let mut ret: String = String::from("");
    let exists = Path::new(&path).exists();

    if force || exists {
        let mut now:u64 = 0;
        let mut modify:u64 = 0;

        if exists{
            now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()-1;
            let meta = fs::metadata(&path).unwrap();
            modify = meta.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        }

        if force || now <= modify {
            let mut event_name = eventname.to_string();

            if eventname == "" && exists {
                event_name = fs::read_to_string(path).unwrap();
            }

            if event_name != "" {
                let url = format!(
                    "http://www.plasmasphere.net/archives/umaumalogger/api/event.php?kwd={}&musumename={}",
                    urlencoding::encode(&event_name),
                    urlencoding::encode(musumename)
                );
                match reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build() {
                    Ok(client) => {
                        match client.get(&url).send() {
                            Ok(response) => {
                                if response.status() == 200 {
                                    if let Ok(text) = response.text() {
                                        ret = text;
                                    }
                                }
                            },
                            Err(_e) => {
                                ret = String::from("");
                            }
                        }
                    },
                    Err(_) => {
                        ret = String::from("");
                    }
                }
            }
        }
    }

    format!("{}", ret)
}

#[tauri::command]
fn start_logwatch(filename: String, state: tauri::State<'_, Mutex<LogWatchState>>, app_handle: tauri::AppHandle) {
    let mut s = state.lock().unwrap();
    s.watcher = None;
    *s.logno.lock().unwrap() = 0;

    let logno = Arc::clone(&s.logno);
    let app = app_handle.clone();
    let watch_dir = format!("{}\\out", get_currentpath());
    let filename_clone = filename.clone();

    let _ = std::fs::create_dir_all(&watch_dir);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                let target_matches = event.paths.iter().any(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == filename_clone)
                        .unwrap_or(false)
                });
                if target_matches && (event.kind.is_modify() || event.kind.is_create()) {
                    let current_logno = *logno.lock().unwrap();
                    println!("logwatch: file changed, current logno={}", current_logno);
                    let ret = get_filelog_lastline(&current_logno.to_string(), &filename_clone);
                    if let Some(newline_pos) = ret.find('\n') {
                        if let Ok(new_logno) = ret[..newline_pos].trim().parse::<usize>() {
                            *logno.lock().unwrap() = new_logno;
                        }
                    }
                    let _ = app.emit_all("logrefresh", &ret);
                }
            },
            Err(e) => println!("logwatch error: {:?}", e),
        }
    }).unwrap();

    match watcher.watch(Path::new(&watch_dir), RecursiveMode::NonRecursive) {
        Ok(_) => println!("logwatch: watching {}", watch_dir),
        Err(e) => {
            println!("logwatch: watch failed {:?}", e);
            return;
        }
    }

    s.watcher = Some(watcher);
}

#[tauri::command]
fn stop_logwatch(state: tauri::State<'_, Mutex<LogWatchState>>) {
    let mut s = state.lock().unwrap();
    s.watcher = None;
    *s.logno.lock().unwrap() = 0;
    println!("logwatch: stopped");
}

fn main() {
    let exit = CustomMenuItem::new("exit".to_string(), "終了");
    let check = CustomMenuItem::new("check".to_string(), "更新をチェック");
    let version = CustomMenuItem::new("vers".to_string(), "バージョン情報");
    let filemenu = Submenu::new("ファイル", Menu::new().add_item(exit.clone()));
    let helpmenu = Submenu::new("ヘルプ", Menu::new().add_item(check.clone()).add_native_item(MenuItem::Separator).add_item(version));
    let menu = Menu::new()
        .add_submenu(filemenu)
        .add_submenu(helpmenu);

    let shot = CustomMenuItem::new("shot".to_string(), "スクリーンショット");
    let show = CustomMenuItem::new("show".to_string(), "表示");
    let hide = CustomMenuItem::new("hide".to_string(), "隠す");
    let tray_menu = SystemTrayMenu::new()
        .add_item(shot)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(check.clone())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(exit.clone());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            filelogging,
            get_filelog_lastline,
            get_path,
            get_loglists,
            get_filelog,
            take_screenshot,
            get_imagelist,
            get_eventvalue,
            start_logwatch,
            stop_logwatch
        ])
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "exit" => {
                    std::process::exit(0);
                }
                "check" => {
                    check_version(event.window());
                }
                "vers" => {
                    let context = tauri::generate_context!();
                    let package = context.package_info();
                    let msg: String = format!("UmaUmaLogger\r\n\r\nVersion {}\n\rAuthor: sou1ka @sou1ka", package.version);
                    dialog::message(Some(&event.window()), &package.name, &msg);
                }
                _ => {}
            }
        })
        .setup(|app| {
            app.manage(Mutex::new(LogWatchState {
                watcher: None,
                logno: Arc::new(Mutex::new(0)),
            }));

            let app_handle = app.app_handle();
            std::thread::spawn(move || loop {
                let _ = app_handle.emit_all("eventrefresh", get_eventvalue("", "", false));
                std::thread::sleep(std::time::Duration::from_secs(1));
            });

            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.unminimize().unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
              match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.unminimize().unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "shot" => {
                    let _ = &take_screenshot();
                }
                "check" => {
                    check_version(&app.get_window("main").unwrap());
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "exit" => {
                    std::process::exit(0);
                }
                _ => {}
              }
            }
            _ => {}
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
//    return String::from("R:\\test");
}


fn check_version(win: &tauri::Window) {
    let mut checker = String::from("");

    match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build() {
        Ok(client) => {
            match client.get("http://www.plasmasphere.net/archives/umaumalogger/api/version.txt").send() {
                Ok(response) => {
                    if let Ok(text) = response.text() {
                        checker = text;
                    }
                },
                Err(_e) => {
                    checker = "0".to_string();
                }
            }
        },
        Err(_) => {
            checker = "0".to_string();
        }
    }

    let context = tauri::generate_context!();
    let package = context.package_info();
    let ver = package.version.to_string();

    if checker == "0" {
        dialog::message(Some(&win), &package.name, "更新チェックに失敗しました。");

    } else if ver != checker {
        let scope = win.shell_scope();
        dialog::ask(Some(&win), &package.name, "新しいバージョンがあります。ダウンロードしますか？", move |answer| {
            if answer {
                shell::open(&scope, "http://www.plasmasphere.net/archives/umaumalogger/", None).unwrap();
            }
        });

    } else {
        dialog::message(Some(&win), &package.name, "最新バージョンです。");
    }
}
