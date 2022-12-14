#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::api::{dialog, shell};
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, Manager};

use std::env;
use std::io;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::collections::HashMap;
use std::time::SystemTime;
use chrono::Utc;
use easy_http_request::DefaultHttpRequest;

mod screenshot;

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
fn get_imagelist() -> String {
    let path: &str = &format!("{}\\screenshot\\", get_currentpath());
    let lists: Vec<String> = read_dir(path).unwrap();
    let mut arr: Vec<String> = Vec::new();

    for item in lists {
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
                let response = DefaultHttpRequest::get_from_url_str("http://www.plasmasphere.net/archives/umaumalogger/api/event.php?kwd=".to_owned() + &urlencoding::encode(&event_name)+ "&musumename=" + &urlencoding::encode(musumename)).unwrap().send();
                match response {
                    Ok(r) => {
                        if r.status_code == 200 {
                            ret = String::from_utf8(r.body).unwrap();
                        }
                    },
                    Err(_e) => {
                        ret = String::from("");
                    }
                }
            }
        }
    }

    format!("{}", ret)
}

fn main() {
    let exit = CustomMenuItem::new("exit".to_string(), "??????");
    let check = CustomMenuItem::new("check".to_string(), "?????????????????????");
    let version = CustomMenuItem::new("vers".to_string(), "?????????????????????");
    let filemenu = Submenu::new("????????????", Menu::new().add_item(exit));
    let helpmenu = Submenu::new("?????????", Menu::new().add_item(check).add_native_item(MenuItem::Separator).add_item(version));
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
            get_imagelist,
            get_eventvalue
        ])
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "exit" => {
                    std::process::exit(0);
                }
                "check" => {
                    let checker = check_version();
                    let context = tauri::generate_context!();
                    let package = context.package_info();
                    let ver = package.version.to_string();

                    if checker == "0" {
                        dialog::message(Some(&event.window()), &package.name, "??????????????????????????????????????????");

                    } else if ver != checker {
                        let scope = event.window().shell_scope();
                        dialog::ask(Some(&event.window()), &package.name, "???????????????????????????????????????????????????????????????????????????", move |answer| {
                            if answer {
                                shell::open(&scope, "http://www.plasmasphere.net/archives/umaumalogger/", None).unwrap();
                            }
                        });

                    } else {
                        dialog::message(Some(&event.window()), &package.name, "??????????????????????????????");
                    }
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
            let app_handle = app.app_handle();
            std::thread::spawn(move || loop {
                app_handle.emit_all("eventrefresh", get_eventvalue("", "", false)).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            });

            let app_handle = app.app_handle();
            app.listen_global("logcheck", move |event| {
                let arg: HashMap<&str, &str> = serde_json::from_str(event.payload().unwrap()).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            //    println!("logcheck {:?}", arg);
                app_handle.emit_all("logrefresh", get_filelog_lastline(arg.get("lognoStr").unwrap(), arg.get("filename").unwrap())).unwrap();
            });
            //app.manage(EventIdValue(eid.to_owned().to_string()));
            //app.listen_global("logcheck-stop", move |event| {
            //    println!("logcheck-stop eid: {}", eid);
            //    main_window.unlisten(eid);
            //});
            
            Ok(())
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

fn check_version() -> String {
    let response = DefaultHttpRequest::get_from_url_str("http://www.plasmasphere.net/archives/umaumalogger/api/version.txt").unwrap().send();
    match response {
        Ok(r) => {
            return String::from_utf8(r.body).unwrap();
        },
        Err(_e) => {
            return "0".to_string();
        }
    }
}

