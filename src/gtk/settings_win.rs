use crate::{AppState, gtk::{about_win::about_win, gtk_ui::MarginAll}, logic::{encryption::ShinCrypt, global::{GTKhelper, Global}}, memory::MB64};
use gtk::prelude::*;
use gtk4 as gtk;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SETTINGS_FILE: &str = "settings.ron";

#[derive(Clone, Serialize, Deserialize)]
pub struct AppSettings {
  pub dark_mode: bool,
  pub remove_org: bool,
  pub same_dir: bool,
}

impl Default for AppSettings {
  fn default() -> Self { Self { dark_mode: true, remove_org: false, same_dir: false } }
}

impl AppSettings {
  fn get_settings_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> { Ok(std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join(SETTINGS_FILE)) }

  /// Save settings to a RON file
  pub fn export(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let path = Self::get_settings_path().unwrap();
    let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).map_err(|e| format!("Failed to serialize settings to RON, error: {}", e))?;

    std::fs::write(path.clone(), ron_string).map_err(|e| format!("Failed to write settings to {:?}, error: {}", path, e))?;

    Ok(path)
  }

  /// Load settings from a RON file
  pub fn import() -> Result<Self, Box<dyn std::error::Error>> {
    let path = Self::get_settings_path().unwrap();

    if !path.exists() {
      return Err("Failed to import settings files doesn't exist".into());
    }

    let file_content = std::fs::read_to_string(path.clone()).map_err(|e| format!("Failed to read settings from {:?}, error: {}", path.clone(), e))?;

    let settings = ron::from_str(&file_content).map_err(|e| format!("Failed to parse RON from {:?}, error: {}", path, e))?;

    Ok(settings)
  }
}

/// Show the settings dialog as a child of `parent_win`.
pub fn settings_ui(window: &gtk::ApplicationWindow, aps: Arc<RwLock<AppState>>, bench_prog: crossbeam::channel::Sender<f64>) {
  let consts = aps.read().consts.clone();

  // Build window
  let settings_win = gtk::ApplicationWindow::builder().transient_for(window).modal(true).resizable(true).title("Settings").default_width(300).default_height(150).build();

  let grid = gtk::Grid::new();
  grid.set_row_spacing(consts.upad);
  grid.set_column_spacing(consts.upad);
  grid.set_margin_all(consts.margin);

  let box_cb = gtk::Box::new(gtk::Orientation::Vertical, 0);

  let grid_cb = gtk::Grid::new();
  grid_cb.set_row_spacing(consts.upad);
  grid_cb.set_column_spacing(consts.upad);

  // Dark mode checkbox
  {
    let aps_c = aps.clone();
    let dark_mode_cb = gtk4::CheckButton::with_label("Dark mode");
    dark_mode_cb.set_active(aps_c.read().settings.dark_mode);
    dark_mode_cb.connect_toggled(move |cb| {
      aps_c.write().settings.dark_mode = cb.is_active();
      gtk4::Settings::default().expect("Failed to get Settings").set_gtk_application_prefer_dark_theme(cb.is_active());
      aps_c.read().settings.export().unwrap();
    });
    grid_cb.attach(&dark_mode_cb, 0, 0, 1, 1);
  }

  // Remove source file checkbox
  {
    let aps_c = aps.clone();
    let remove_cb = gtk4::CheckButton::with_label("Remove source file");
    remove_cb.set_active(aps_c.read().settings.remove_org);
    remove_cb.connect_toggled(move |cb| {
      aps_c.write().settings.remove_org = cb.is_active();
      aps_c.read().settings.export().unwrap();
    });
    grid_cb.attach(&remove_cb, 0, 1, 1, 1);
  }

  // Smae directory output checkbox
  {
    let aps_c = aps.clone();
    let same_dir_cb = gtk4::CheckButton::with_label("Same directory output");
    same_dir_cb.set_active(aps_c.read().settings.same_dir);
    same_dir_cb.connect_toggled(move |cb| {
      aps_c.write().settings.same_dir = cb.is_active();
      aps_c.read().settings.export().unwrap();
    });
    grid_cb.attach(&same_dir_cb, 0, 2, 1, 1);
  }

  box_cb.append(&grid_cb);

  let box_btn = gtk::Box::new(gtk::Orientation::Vertical, 0);

  let grid_btn = gtk::Grid::new();
  grid_btn.set_row_spacing(consts.upad);
  grid_btn.set_column_spacing(consts.upad);

  let (b_res_s, b_res_r) = crossbeam::channel::unbounded::<(std::time::Duration, std::time::Duration)>();
  let window_c = window.clone();
  let bench_prog_c = bench_prog.clone();

  {
    let benchmark_btn = gtk4::Button::with_label("Benchmark 🚝");
    benchmark_btn.set_hexpand(true);
    // benchmark_btn.set_sensitive(false);
    benchmark_btn.set_tooltip_text(Some("Test your machine"));
    benchmark_btn.connect_clicked(move |_| {
      GTKhelper::message_box(&window_c, "Please wait", "Benchmarking has started, the progress bar will be filled two times then the results will appear once the test is done.", None);

      let b_res_s_c = b_res_s.clone();
      let mut shin_crypt = ShinCrypt::default();
      shin_crypt.set_progres(Some(bench_prog_c.clone()));

      match std::thread::Builder::new().stack_size(MB64).spawn(move || shin_crypt.benchmark(b_res_s_c)) {
        Ok(_) => (),
        Err(e) => GTKhelper::message_box(&window_c, "Error", e.to_string(), None),
      };
    });

    let width = if cfg!(windows) { 1 } else { 2 };
    grid_btn.attach(&benchmark_btn, 0, 0, width, 1);
  }

  #[cfg(target_os = "windows")]
  {
    let window_c = window.clone();
    let aps_c = aps.clone();

    let tooltip = if consts.elevated { "Install to C:\\ProgramFiles and add to context menu" } else { "Available when run as admin" };

    let install_btn = gtk4::Button::with_label("Install ⬇️️");
    install_btn.set_hexpand(true);
    install_btn.set_sensitive(consts.elevated);
    install_btn.set_tooltip_text(Some(tooltip));
    install_btn.connect_clicked(move |_| {
      install(&window_c, aps_c.clone());
    });
    grid_btn.attach(&install_btn, 1, 0, 1, 1);
  }

  {
    let window_c = window.clone();
    let aps_c = aps.clone();

    let about_btn = gtk4::Button::with_label("About ℹ️");
    about_btn.set_hexpand(true);
    about_btn.connect_clicked(move |_| {
      about_win(&window_c, aps_c.clone());
    });
    grid_btn.attach(&about_btn, 0, 1, 2, 1);
  }

  box_btn.append(&grid_btn);

  let window_c = window.clone();

  // Use glib::source::idle_add to update GUI from main thread
  gtk::glib::source::idle_add_local(move || {
    if let Ok((e_time, d_time)) = b_res_r.try_recv() {
      GTKhelper::message_box(&window_c, "Done", format!("Encrypted 1GB:\n\nTime: {}\nSpeed: {:.2} MB/s\n\nDecrypted 1GB:\n\nTime: {}\nSpeed: {:.2} MB/s\n", Global::format_duration(e_time), Global::calculate_speed(1.0, e_time), Global::format_duration(d_time), Global::calculate_speed(1.0, d_time)), None)
    };

    gtk::glib::ControlFlow::Continue
  });

  grid.attach(&box_cb, 0, 0, 1, 1);
  grid.attach(&box_btn, 0, 1, 1, 1);

  settings_win.set_child(Some(&grid));
  settings_win.present();
}

pub fn install(window: &gtk4::ApplicationWindow, aps: Arc<RwLock<AppState>>) {
  let consts = aps.read().consts.clone();

  let exe_source = std::path::PathBuf::from(&consts.file_name);
  let exe_target_dir = std::path::PathBuf::from(format!(r"C:\Program Files\{}\{}", consts.author_ghd, consts.app_name));
  let exe_target = exe_target_dir.join(&consts.file_name);

  // Ensure target dir exists
  std::fs::create_dir_all(&exe_target_dir).unwrap();
  std::fs::copy(&exe_source, &exe_target).unwrap();

  let settings_src = std::path::PathBuf::from(SETTINGS_FILE);

  if settings_src.exists() {
    std::fs::copy(settings_src, exe_target_dir.join(SETTINGS_FILE)).unwrap();
  }

  // let command_name = &app_name;

  // // Add registry keys
  // let hkcr = winreg::RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT);

  // // Create the parent key: *\shell\<command_name>
  // let (file_key, _) = hkcr.create_subkey(format!(r#"*\shell\{}"#, command_name)).unwrap();

  // // Set the icon here
  // file_key.set_value("Icon", &format!(r#""{}""#, exe_target.display())).unwrap();

  // // Now create the command subkey
  // let (command_key, _) = file_key.create_subkey("command").unwrap();
  // command_key.set_value("", &format!(r#""{}" "%1""#, exe_target.display())).unwrap();

  // // Create the parent key: *\shell\<command_name>
  // let (dir_key, _) = hkcr.create_subkey(format!(r#"Directory\shell\{}"#, command_name)).unwrap();

  // // Set the icon here
  // dir_key.set_value("Icon", &format!(r#""{}""#, exe_target.display())).unwrap();

  // // Now create the command subkey
  // let (command_key, _) = dir_key.create_subkey("command").unwrap();
  // command_key.set_value("", &format!(r#""{}" "%1""#, exe_target.display())).unwrap();

  let reg_path = format!(r#"*\shell\{}"#, consts.app_name);

  add_ctx_option(&reg_path, &exe_target);

  let reg_path = format!(r#"Directory\shell\{}"#, consts.app_name);

  add_ctx_option(&reg_path, &exe_target);

  GTKhelper::message_box(window, "Success", format!("Application installed successfully to:\n{}", exe_target.display()), None);
}

fn add_ctx_option(reg_path: &str, exe_target: impl AsRef<std::path::Path>) {
  // Add registry keys
  let hkcr = winreg::RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT);

  // Create the parent key: *\shell\<command_name>
  let (dir_key, _) = hkcr.create_subkey(reg_path).unwrap();

  // Set the icon here
  dir_key.set_value("Icon", &format!(r#""{}""#, exe_target.as_ref().display())).unwrap();

  // Now create the command subkey
  let (command_key, _) = dir_key.create_subkey("command").unwrap();
  command_key.set_value("", &format!(r#""{}" "%1""#, exe_target.as_ref().display())).unwrap();
}
