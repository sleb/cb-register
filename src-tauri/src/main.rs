// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::Mutex, thread};

use cb_register::{create_menu, ClipboardHistory, ClipboardRegister};
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardWatcher, ClipboardWatcherContext};
use tauri::{
    ActivationPolicy, AppHandle, GlobalShortcutManager, Manager, State, SystemTray,
    SystemTrayEvent, WindowEvent,
};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn register_shortcut(app_handle: AppHandle) {
    app_handle
        .global_shortcut_manager()
        .register("Super+Alt+V", move || {
            let window = app_handle
                .get_window("main")
                .expect("couldn't get main window");
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        })
        .unwrap();
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .on_window_event(|window_event| match window_event.event() {
            WindowEvent::Focused(focused) => {
                if !focused {
                    window_event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app: &mut tauri::App| {
            let handle = app.handle();
            let window = handle.get_window("main").unwrap();
            window.on_window_event(move |e| if let WindowEvent::Focused(false) = e {});

            register_shortcut(handle.clone());

            app.set_activation_policy(ActivationPolicy::Accessory);

            let history = Mutex::new(ClipboardHistory::new());
            app.manage(history);

            let cb_register = ClipboardRegister::new(handle.clone());
            let mut cb_watcher_ctx = ClipboardWatcherContext::new().unwrap();
            cb_watcher_ctx.add_handler(cb_register);

            thread::spawn(move || {
                cb_watcher_ctx.start_watch();
            });

            SystemTray::new()
                .with_menu(create_menu(&ClipboardHistory::new()))
                .build(app)
                .unwrap();
            Ok(())
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                _ => {
                    let history: State<Mutex<ClipboardHistory>> = app.state();
                    if let Ok(history) = history.lock() {
                        if let Some(item) = history.find_item(&id) {
                            let ctx = ClipboardContext::new().unwrap();
                            ctx.set_text(String::from(item)).unwrap();
                        }
                    };
                }
            },
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
