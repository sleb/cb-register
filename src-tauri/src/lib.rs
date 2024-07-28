use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Mutex,
};

use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler, ContentFormat};
use tauri::{AppHandle, CustomMenuItem, Manager, State, SystemTrayMenu, SystemTrayMenuItem};
use unicode_segmentation::UnicodeSegmentation;

pub struct ClipboardRegister {
    ctx: ClipboardContext,
    app_handle: AppHandle,
}

impl ClipboardRegister {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            ctx: ClipboardContext::new().unwrap(),
            app_handle,
        }
    }
}

impl ClipboardHandler for ClipboardRegister {
    fn on_clipboard_change(&mut self) {
        if self.ctx.has(ContentFormat::Text) {
            let history: State<Mutex<ClipboardHistory>> = self.app_handle.state();
            if let Ok(mut history) = history.lock() {
                history.add_item(self.ctx.get_text().unwrap());

                self.app_handle
                    .tray_handle()
                    .set_menu(create_menu(&history))
                    .unwrap();
            };
        }
    }
}

pub struct ClipboardHistory {
    inner: HashMap<String, String>,
}

impl ClipboardHistory {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn add_item(&mut self, item: String) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();
        self.inner.insert(hash.to_string(), item);
        hash
    }

    pub fn find_item(&self, hash: &str) -> Option<&String> {
        self.inner.get(hash)
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }
}

pub fn create_menu_items_for_history<'a>(
    history: &'a ClipboardHistory,
) -> impl Iterator<Item = CustomMenuItem> + 'a {
    history.iter().map(|(hash, item)| {
        CustomMenuItem::new(
            hash,
            format!("{}...", item.graphemes(true).take(25).collect::<String>()),
        )
    })
}

pub fn create_menu(history: &ClipboardHistory) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new();

    for item in create_menu_items_for_history(history) {
        menu = menu.add_item(item);
    }

    menu.add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"))
}
