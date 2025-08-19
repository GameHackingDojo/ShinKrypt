#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{gtk::settings_win::AppSettings, logic::global::Global};
mod cli;
mod gtk;
mod logic;

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
  pub authors: Vec<String>,
  pub author_ghd: String,
  pub author_ken: String,
  pub install_dir: std::path::PathBuf,
  pub repo_owner: String,
  pub github_repo: String,
  pub download_url: String,
  pub patreon_url: String,
  pub reg_keys: Vec<String>,

  pub upad: u32,
  pub margin: i32,
  pub elevated: bool,
}

impl Default for AppConsts {
  fn default() -> Self {
    let app_name = String::from(env!("CARGO_PKG_NAME"));
    let file_name = if cfg!(target_os = "windows") { format!("{}.exe", app_name) } else { app_name.clone() };
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let authors: Vec<String> = env!("CARGO_PKG_AUTHORS").split(':').map(str::to_string).collect();
    let author_ghd = String::from(authors[0].trim());
    let author_ken = String::from(authors[1].trim());
    let install_dir = if cfg!(target_os = "windows") { std::path::PathBuf::from(format!(r"C:\Program Files\{}\{}", author_ghd, app_name)) } else { std::path::PathBuf::new() };
    let repo_owner = String::from("GameHackingDojo");
    let github_repo = String::from(env!("CARGO_PKG_REPOSITORY"));
    let download_url = format!("https://api.github.com/repos/{}/{}/releases/latest", repo_owner, app_name);
    let patreon_url = format!("https://www.patreon.com/c/{}", repo_owner);
    let elevated = if cfg!(target_os = "windows") { Global::is_elevated().unwrap_or(false) } else { false };
    let reg_keys = vec![format!(r#"*\shell\{}"#, app_name), format!(r#"Directory\shell\{}"#, app_name)];

    return Self {
      app_name,
      upad: 10,
      margin: 20,
      file_name,
      version,
      authors,
      author_ghd,
      author_ken,
      install_dir,
      repo_owner,
      github_repo,
      download_url,
      patreon_url,
      elevated,
      reg_keys,
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
