mod app_discovery;
mod search;

use app_discovery::{AppInfo, scan_windows_apps};
use search::search_apps;
use std::sync::Mutex;
use std::process::Command;

struct AppState {
    apps: Mutex<Vec<AppInfo>>,
}

#[tauri::command]
fn get_installed_apps(state: tauri::State<AppState>) -> Vec<AppInfo> {
    state.apps.lock().unwrap().clone()
}

#[tauri::command]
fn search_applications(query: String, state: tauri::State<AppState>) -> Vec<AppInfo> {
    let apps = state.apps.lock().unwrap();
    search_apps(&apps, &query)
}

#[tauri::command]
fn launch_app(app_path: String) -> Result<(), String> {
    match Command::new(&app_path).spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to launch application: {}", e)),
    }
}

#[tauri::command]
fn resize_window(window: tauri::Window, height: f64) {
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: 640,
        height: height as u32,
    }));
}

pub fn run() {
    let apps = scan_windows_apps();

    tauri::Builder::default()
        .manage(AppState {
            apps: Mutex::new(apps),
        })
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                let defined_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().with_handler(move |_app, shortcut, event| {
                        if shortcut == &defined_shortcut {
                            match event.state() {
                              ShortcutState::Pressed => {
                                match window.is_visible() {
                                    Ok(true) => {
                                        let _ = window.hide();
                                    }
                                    Ok(false) => {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                    Err(error) => {
                                        println!("Err: {}", error );
                                    }
                                }
                              },
                              ShortcutState::Released => { return }
                            }
                        }
                    })
                    .build(),
                )?;

                app.global_shortcut().register(defined_shortcut)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            search_applications,
            launch_app,resize_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
