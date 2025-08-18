#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{gtk::settings_win::AppSettings, logic::global::Global};
mod cli;
mod gtk;
mod logic;

const APPNAME: &str = "ShinCrypt";
const OLDAPPNAME: &str = "old_GHD_app";

pub mod memory {
  pub const KB: usize = 1024;
  pub const MB: usize = 1024 * 1024;

  // Derived constants
  pub const MB1: usize = 1 * MB;
  pub const MB4: usize = 4 * MB;
  pub const MB8: usize = 8 * MB;
  pub const MB16: usize = 16 * MB;
  pub const MB32: usize = 32 * MB;
  pub const MB64: usize = 64 * MB;
  pub const MB128: usize = 128 * MB;
  pub const MB256: usize = 256 * MB;
  pub const MB512: usize = 512 * MB;
  pub const GB1: usize = 1024 * MB;
}

#[derive(Clone, Default)]
pub struct AppState {
  pub settings: AppSettings,
  pub consts: AppConsts,
}

#[derive(Clone)]
pub struct AppConsts {
  pub app_name: String,
  pub file_name: String,
  pub version: String,
  pub author: String,
  pub repo_owner: String,
  pub github_repo: String,
  pub download_url: String,
  pub patreon_url: String,

  pub upad: u32,
  pub margin: i32,
  pub elevated: bool,
}

impl Default for AppConsts {
  fn default() -> Self {
    let app_name = String::from(APPNAME);
    let file_name = if cfg!(target_os = "windows") { format!("{}.exe", app_name) } else { app_name.clone() };
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let author = String::from("Game Hacking Dojo");
    let repo_owner = String::from("GameHackingDojo");
    let github_repo = format!("https://github.com/{}/{}", repo_owner, app_name);
    let download_url = format!("https://api.github.com/repos/{}/{}/releases/latest", repo_owner, app_name);
    let patreon_url = format!("https://www.patreon.com/c/{}", repo_owner);
    let elevated = if cfg!(windows) { Global::is_elevated().unwrap_or(false) } else { false };

    return Self {
      app_name: String::from(APPNAME),
      upad: 10,
      margin: 20,
      file_name,
      version,
      author,
      repo_owner,
      github_repo,
      download_url,
      patreon_url,
      elevated,
    };
  }
}

// const ICON_BYTES: &[u8] = if cfg!(target_os = "windows") { include_bytes!("../resources/icon.ico") } else { include_bytes!("../resources/icon.png") };

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let old_app = std::env::current_exe().unwrap().parent().unwrap().join(OLDAPPNAME);
  if old_app.exists() {
    Global::del_path(old_app).unwrap()
  }

  if !cli::cli() {
    unsafe { winapi::um::wincon::FreeConsole() };
    gtk::gtk_ui::gtk_ui();
  }

  Ok(())
}
