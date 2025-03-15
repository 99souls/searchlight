#[tauri::command]
fn resize_window(window: tauri::Window, height: f64) {
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: 640,
        height: height as u32,
    }));
}

pub fn run() {
    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![resize_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
