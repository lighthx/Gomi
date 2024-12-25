use std::env;

pub fn get_data_dir() -> String {
    format!("{}", env::var("HOME").unwrap())
}

pub fn get_db_path() -> String {
    format!("{}/.gomi.db", get_data_dir())
}

pub const MENU_WINDOW_WIDTH: f32 = 300.0;
pub const MENU_WINDOW_HEIGHT: f32 = 300.0;
pub const SETTING_WINDOW_WIDTH: f32 = 1024.0;
pub const SETTING_WINDOW_HEIGHT: f32 = 768.0;
pub const LOG_DIR: &str = "/tmp";
pub const LOG_FILE: &str = "gomi.log";
