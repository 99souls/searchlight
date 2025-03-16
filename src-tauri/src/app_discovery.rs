use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::RegKey;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub description: Option<String>,
}

pub fn scan_windows_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    
    scan_start_menu(&mut apps);
    scan_program_files(&mut apps);
    scan_registry(&mut apps);
    
    let mut unique_apps: HashMap<String, AppInfo> = HashMap::new();
    for app in apps {
        unique_apps.insert(app.path.clone(), app);
    }
    
    unique_apps.into_values().collect()
}

fn scan_start_menu(apps: &mut Vec<AppInfo>) {
  let start_menu_dirs = vec![
      std::path::PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"),
      
      if let Some(local_app_data) = dirs::data_local_dir() {
          local_app_data.join("Microsoft\\Windows\\Start Menu\\Programs")
      } else {
          std::path::PathBuf::new()
      },
  ];
  
  for dir in start_menu_dirs {
      if dir.exists() {
          for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
              if entry.path().extension().map_or(false, |ext| ext == "lnk") {
                  if let Some(app) = parse_shortcut(entry.path()) {
                      apps.push(app);
                  }
              }
          }
      }
  }
}

fn scan_program_files(apps: &mut Vec<AppInfo>) {
  let mut program_dirs = vec![
      std::path::PathBuf::from("C:\\Program Files"),
      std::path::PathBuf::from("C:\\Program Files (x86)"),
      std::path::PathBuf::from("C:\\ProgramData"),
  ];
  
  if let Ok(program_files) = std::env::var("ProgramFiles") {
      program_dirs.push(std::path::PathBuf::from(program_files));
  }
  
  if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
      program_dirs.push(std::path::PathBuf::from(program_files_x86));
  }
  
  if let Ok(program_data) = std::env::var("ProgramData") {
      program_dirs.push(std::path::PathBuf::from(program_data));
  }
  
  for dir in program_dirs {
      if dir.exists() {
          for entry in WalkDir::new(&dir)
              .max_depth(3) 
              .into_iter()
              .filter_map(|e| e.ok()) 
          {
              if entry.path().extension().map_or(false, |ext| ext == "exe") {
                  let path = entry.path().to_string_lossy().to_string();
                  let name = entry.path().file_stem()
                      .and_then(|n| n.to_str())
                      .unwrap_or("Unknown")
                      .to_string();
                  
                  apps.push(AppInfo {
                      name,
                      path,
                      icon_path: None,
                      description: None,
                  });
              }
          }
      }
  }
}

fn scan_registry(apps: &mut Vec<AppInfo>) {
  let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
  
  if let Ok(uninstall) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
      for key in uninstall.enum_keys().filter_map(|k| k.ok()) {
          if let Ok(app_key) = uninstall.open_subkey(&key) {
              if let (Ok(name), Ok(path)) = (
                  app_key.get_value::<String, _>("DisplayName"),
                  app_key.get_value::<String, _>("InstallLocation")
              ) {
                  if !path.is_empty() {
                      let exe_path = find_main_executable(&PathBuf::from(&path));
                      if let Some(exe_path) = exe_path {
                          apps.push(AppInfo {
                              name,
                              path: exe_path.to_string_lossy().to_string(),
                              icon_path: None,
                              description: app_key.get_value("DisplayName").ok(),
                          });
                      }
                  }
              }
          }
      }
  }
}


fn parse_shortcut(path: &std::path::Path) -> Option<AppInfo> {
    let file_name = path.file_stem()?.to_str()?;
    Some(AppInfo {
        name: file_name.to_string(),
        path: path.to_string_lossy().to_string(),
        icon_path: None,
        description: None,
    })
}

fn find_main_executable(dir: &PathBuf) -> Option<PathBuf> {
    for &name in &["app.exe", "main.exe", "launcher.exe"] {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    
    if dir.exists() && dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "exe") {
                    return Some(path);
                }
            }
        }
    }
    
    None
}
