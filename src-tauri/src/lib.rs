mod app_discovery;
mod search;
mod icon_extractor;

use app_discovery::{AppInfo, scan_windows_apps};
use icon_extractor::IconCache;
use search::search_apps;
use std::env;
use std::sync::Mutex;
use std::process::Command;

struct AppState {
    apps: Mutex<Vec<AppInfo>>,
    icon_cache: IconCache,
}

#[tauri::command]
fn get_installed_apps(state: tauri::State<'_, AppState>) -> Vec<AppInfo> {
    let mut apps = state.apps.lock().unwrap().clone();
    
    for app in &mut apps {
        if let Some(icon_data) = state.icon_cache.get_icon_data(&app.path) {
            app.icon_path = Some(icon_data); 
        }
    }
    
    apps.into_iter()
        .filter(|app| {
            let path = app.path.to_lowercase();
            !(path.contains("\\start menu\\") || 
              path.contains("\\startmenu\\") || 
              (path.contains("\\programs\\") && path.ends_with(".lnk")))
        })
        .collect()
}

#[tauri::command]
fn search_applications(query: String, state: tauri::State<'_, AppState>) -> Vec<AppInfo> {
    let apps = state.apps.lock().unwrap();
    let mut results = search_apps(&apps, &query);
    
    results.retain(|app| {
        let path = app.path.to_lowercase();
        !(path.contains("\\start menu\\") || 
          path.contains("\\startmenu\\") || 
          (path.contains("\\programs\\") && path.ends_with(".lnk")))
    });
    
    results
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

#[tauri::command]
fn get_app_icon(app_path: String, state: tauri::State<'_, AppState>) -> Option<String> {
    state.icon_cache.get_icon_data(&app_path)
}

pub fn run() {
    let apps = scan_windows_apps();
    
    let icon_cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => cache_dir.join("searchlight").join("icon_cache"),
        None => {
            let temp_dir = env::temp_dir().join("searchlight").join("icon_cache");
            println!("Using temp directory for icon cache: {:?}", temp_dir);
            temp_dir
        }
    };

    if let Err(e) = std::fs::create_dir_all(&icon_cache_dir) {
        println!("Warning: Failed to create icon cache directory: {}", e);
    }
    
    let icon_cache = IconCache::new();

    tauri::Builder::default()
        .manage(AppState {
            apps: Mutex::new(apps),
            icon_cache,
        })
        .setup(|app| {
            #[cfg(desktop)]
            use tauri::Manager;
        
            let app_handle = app.handle();
            let window = app.get_webview_window("main").unwrap();
            let window_label = "main"; 
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
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
                                        let _ = window.center();
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
            
            if let Some(window) = app.get_webview_window("main") {
                let app_handle_blur = app_handle.clone();
                
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(false) => {
                            let app_handle_thread = app_handle_blur.clone();
                            let window_label_thread = window_label.to_string();
                            
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                
                                if let Some(window) = app_handle_thread.get_webview_window(&window_label_thread) {
                                    let _ = window.hide();
                                }
                            });
                        },
                        _ => {}
                    }
                });
            } 
            Ok(())
        }) 
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            search_applications,
            launch_app,
            resize_window,
            get_app_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
