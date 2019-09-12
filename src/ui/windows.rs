use std::process::Command;
use crate::bridge::windows::{show_notification, close_notification, WindowsMenuItem, show_context_menu};
use widestring::U16CString;
use std::{fs, thread, time};
use log::{info, debug};
use std::sync::Mutex;
use std::sync::Arc;
use std::fs::create_dir_all;
use std::os::raw::c_void;
use crate::ui::{MenuItem, MenuItemType};

const BMP_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.bmp");
const ICO_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.ico");

pub struct WindowsUIManager {
    id: Arc<Mutex<i32>>
}

impl super::UIManager for WindowsUIManager {
    fn notify(&self, message: &str) {
        let current_id: i32 = {
            let mut id = self.id.lock().unwrap();
            *id += 1;
            *id
        };

        // Setup a timeout to close the notification
        let id = Arc::clone(&self.id);
        thread::spawn(move || {
            for i in 1..10 {
                let duration = time::Duration::from_millis(200);
                thread::sleep(duration);

                let new_id = id.lock().unwrap();
                if *new_id != current_id {
                    debug!("Cancelling notification close event with id {}", current_id);
                    return;
                }
            }

            unsafe {
                close_notification();
            }
        });

        // Create and show a window notification
        unsafe {
            let message = U16CString::from_str(message).unwrap();
            show_notification(message.as_ptr());
        }

    }

    fn show_menu(&self, menu: Vec<MenuItem>) {
        let mut raw_menu = Vec::new();

        for item in menu.iter() {
            let text = U16CString::from_str(item.item_name.clone()).unwrap_or_default();
            let mut str_buff : [u16; 100] = [0; 100];
            unsafe {
                std::ptr::copy(text.as_ptr(), str_buff.as_mut_ptr(), text.len());
            }

            let menu_type = match item.item_type {
                MenuItemType::Button => {1},
                MenuItemType::Separator => {2},
            };

            let raw_item = WindowsMenuItem {
                item_id: item.item_id,
                item_type: menu_type,
                item_name: str_buff,
            };

            raw_menu.push(raw_item);
        }

        unsafe { show_context_menu(raw_menu.as_ptr(), raw_menu.len() as i32); }
    }
}

impl WindowsUIManager {
    pub fn new() -> WindowsUIManager {
        let id = Arc::new(Mutex::new(0));

        let manager = WindowsUIManager {
            id
        };

        manager
    }
}