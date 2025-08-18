use crate::{AppState, gtk::settings_win::{AppSettings, settings_ui}, logic::{encryption::ShinCrypt, global::{GTKhelper, Global}}, memory::MB64};
use gtk::prelude::*;
use gtk4 as gtk;
use parking_lot::RwLock;
use std::{path::PathBuf, sync::Arc};

// 1) Define the trait
pub trait MarginAll {
  /// Set all four margins (start, end, top, bottom) to the same value.
  fn set_margin_all(&self, margin: i32);
}

// 2) Implement it for every type that implements `IsA<Widget>`
impl<T: IsA<gtk::Widget>> MarginAll for T {
  fn set_margin_all(&self, margin: i32) {
    self.set_margin_start(margin);
    self.set_margin_end(margin);
    self.set_margin_top(margin);
    self.set_margin_bottom(margin);
  }
}

pub fn gtk_ui() -> gtk::glib::ExitCode {
  let application = gtk::Application::builder().flags(gtk::gio::ApplicationFlags::HANDLES_OPEN).build();
  let aps = Arc::new(RwLock::new(AppState::default()));
  let consts = aps.read().consts.clone();

  let setup_app = move |app: &gtk::Application, file_path: Option<std::path::PathBuf>| {
    if let Ok(settings) = AppSettings::import() {
      aps.write().settings = settings;
    }
    // dark mode
    gtk::Settings::default().expect("Failed to get settings").set_gtk_application_prefer_dark_theme(aps.read().settings.dark_mode);

    // Add CSS
    let provider = gtk::CssProvider::new();
    let css = std::str::from_utf8(include_bytes!("gtk.css")).unwrap();
    provider.load_from_string(css);
    gtk::style_context_add_provider_for_display(&gtk::gdk::Display::default().unwrap(), &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    // Main window
    let window = gtk::ApplicationWindow::new(app);
    window.set_title(Some(&consts.app_name));
    window.set_default_size(400, 160);
    window.set_resizable(true);

    let app_c = app.clone();

    window.connect_close_request(move |_| {
      app_c.quit();
      gtk::glib::Propagation::Proceed // Allow the window to close
    });

    // Grid container with spacing
    let grid = gtk::Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_margin_all(consts.margin);
    window.set_child(Some(&grid));

    let progress = gtk::ProgressBar::new();
    progress.set_fraction(0.0);

    grid.attach(&progress, 0, 0, 3, 1);

    // Row 0: Input + Browse
    let input = gtk::Entry::new();
    input.set_placeholder_text(Some("Path to file/directory"));
    input.set_hexpand(true);
    if let Some(path) = file_path {
      input.set_text(path.to_str().unwrap());
    }

    GTKhelper::drag_n_drop(&input);

    let window_c = window.clone();
    let input_c = input.clone();

    let browse_i_btn = gtk::Button::with_label("📁");
    browse_i_btn.set_tooltip_text(Some("Select file"));
    browse_i_btn.connect_clicked(move |_| {
      let file_dialog = gtk::FileDialog::new();
      file_dialog.set_title("Select a file");
      let input_c = input_c.clone();

      file_dialog.open(Some(&window_c), gtk::gio::Cancellable::NONE, move |result| match result {
        Ok(v) => input_c.set_text(v.path().unwrap_or_default().to_str().unwrap()),
        Err(err) => eprintln!("Error: {}", err),
      });
    });

    grid.attach(&input, 0, 1, 2, 1);
    grid.attach(&browse_i_btn, 2, 1, 1, 1);

    let same_dir = aps.read().settings.same_dir;

    // Row 0: Input + Browse
    let output = gtk::Entry::new();
    output.set_placeholder_text(Some("Path to output directory"));
    output.set_hexpand(true);
    output.set_sensitive(!same_dir);

    GTKhelper::drag_n_drop(&output);

    let window_c = window.clone();
    let output_c = output.clone();

    let browse_o_btn = gtk::Button::with_label("📂");
    browse_o_btn.set_tooltip_text(Some("Select directory"));
    browse_o_btn.connect_clicked(move |_| {
      let file_dialog = gtk::FileDialog::new();
      file_dialog.set_title("Select a directory");
      let output_c = output_c.clone();

      file_dialog.select_folder(Some(&window_c), gtk::gio::Cancellable::NONE, move |result| match result {
        Ok(v) => output_c.set_text(v.path().unwrap_or_default().to_str().unwrap()),
        Err(err) => eprintln!("Error: {}", err),
      });
    });

    browse_o_btn.set_sensitive(!same_dir);

    grid.attach(&output, 0, 2, 2, 1);
    grid.attach(&browse_o_btn, 2, 2, 1, 1);

    // Row 1: Password + Show/Hide
    let password = gtk::Entry::new();
    password.set_placeholder_text(Some("Password"));
    password.set_visibility(false);
    password.set_hexpand(true);

    GTKhelper::drag_n_drop(&password);

    let password_c = password.clone();
    let pw_toggle = gtk::Button::with_label("👁️");
    pw_toggle.set_tooltip_text(Some("Toggle password visbility"));
    pw_toggle.connect_clicked(move |_| {
      let visible = !gtk::prelude::EntryExt::is_visible(&password_c);
      gtk::prelude::EntryExt::set_visibility(&password_c, visible);
    });

    grid.attach(&password, 0, 3, 2, 1);
    grid.attach(&pw_toggle, 2, 3, 1, 1);

    let aps_c = aps.clone();
    let window_c = window.clone();

    let (bench_prog_s, bench_prog_r) = crossbeam::channel::unbounded::<f64>();

    // Row 2: (empty cell) + Settings button
    let settings_btn = gtk::Button::with_label("⚙️");
    settings_btn.set_tooltip_text(Some("Settings"));
    settings_btn.connect_clicked(move |_| {
      settings_ui(&window_c, aps_c.clone(), bench_prog_s.clone());
    });

    let encrypt_btn = gtk::Button::with_label("Encrypt 🔒");
    let decrypt_btn = gtk::Button::with_label("Decrypt 🔓");

    grid.attach(&encrypt_btn, 0, 4, 1, 1);
    grid.attach(&decrypt_btn, 1, 4, 1, 1);
    grid.attach(&settings_btn, 2, 4, 1, 1);

    let (e_res_tx, e_res_rx) = crossbeam::channel::unbounded::<(String, std::time::Duration)>();
    let (d_res_tx, d_res_rx) = crossbeam::channel::unbounded::<(String, std::time::Duration)>();
    let (progress_s, progress_r) = crossbeam::channel::unbounded::<f64>();

    let window_c = window.clone();
    let grid_c = grid.clone();
    let input_c = input.clone();
    let output_c = output.clone();
    let password_c = password.clone();
    let aps_c = aps.clone();
    let e_res_tx_c = e_res_tx.clone();
    let progress_s_c = progress_s.clone();

    encrypt_btn.connect_clicked(move |_| {
      let mut input_v = input_c.text().to_string();
      input_v.retain(|c| c != '"' && c != '\'');

      let mut output_v = output_c.text().to_string();
      output_v.retain(|c| c != '"' && c != '\'');

      let input_path = PathBuf::from(input_v.clone());
      let mut output_path = PathBuf::from(output_v.clone());
      let password_v = password_c.text().to_string();

      if input_v.is_empty() || password_v.is_empty() {
        GTKhelper::message_box(&window_c, "Error", "Fill in the required fields", None);
        return;
      }

      if !input_path.exists() {
        GTKhelper::message_box(&window_c, "Error", "Invalid input path", None);
        return;
      }

      if aps_c.read().settings.same_dir {
        output_path = input_path.parent().unwrap().to_path_buf()
      } else {
        if !output_path.exists() {
          if let Err(e) = std::fs::create_dir_all(&output_path) {
            GTKhelper::message_box(&window_c, "Error", format!("Failed to create directory:\n{}", e), None);
            return;
          }
        }
      }

      grid_c.set_sensitive(false);

      let input_path_c = input_path.clone();
      let output_path_c = output_path.clone();
      let e_res_tx_c_c = e_res_tx_c.clone();
      let progress_s_c_c = progress_s_c.clone();

      let shincrypt = ShinCrypt::new(input_path_c.clone(), output_path_c.clone(), password_v.clone(), Some(progress_s_c_c.clone()));

      std::thread::Builder::new()
        .stack_size(MB64)
        .spawn(move || {
          let time = std::time::Instant::now();
          match shincrypt.encrypt_file() {
            Ok(_) => e_res_tx_c_c.send(("Success".to_string(), time.elapsed())),
            Err(e) => e_res_tx_c_c.send((e, time.elapsed())),
          }
          .unwrap();
        })
        .unwrap();

      password_c.set_text("");
    });

    let window_c = window.clone();
    let grid_c = grid.clone();
    let input_c = input.clone();
    let output_c = output.clone();
    let password_c = password.clone();
    let aps_c = aps.clone();
    let d_res_tx_c = d_res_tx.clone();
    let progress_s_c = progress_s.clone();

    decrypt_btn.connect_clicked(move |_| {
      let mut input_v = input_c.text().to_string();
      input_v.retain(|c| c != '"' && c != '\'');

      let mut output_v = output_c.text().to_string();
      output_v.retain(|c| c != '"' && c != '\'');

      let input_path = PathBuf::from(input_v.clone());
      let mut output_path = PathBuf::from(output_v.clone());
      let password_v = password_c.text().to_string();

      if input_v.is_empty() || password_v.is_empty() {
        GTKhelper::message_box(&window_c, "Error", "Fill in the required fields", None);
        return;
      }

      if !input_path.exists() {
        GTKhelper::message_box(&window_c, "Error", "Invalid input path", None);
        return;
      }

      if aps_c.read().settings.same_dir {
        output_path = input_path.parent().unwrap().to_path_buf()
      } else {
        if !output_path.exists() {
          if let Err(e) = std::fs::create_dir_all(&output_path) {
            GTKhelper::message_box(&window_c, "Error", format!("Failed to create directory:\n{}", e), None);
            return;
          }
        }
      }

      grid_c.set_sensitive(false);

      let input_path_c = input_path.clone();
      let output_path_c = output_path.clone();
      let d_res_tx_c_c = d_res_tx_c.clone();
      let progress_s_c_c = progress_s_c.clone();

      let shincrypt = ShinCrypt::new(input_path_c.clone(), output_path_c.clone(), password_v.clone(), Some(progress_s_c_c.clone()));

      std::thread::Builder::new()
        .stack_size(MB64)
        .spawn(move || {
          let time = std::time::Instant::now();
          match shincrypt.decrypt_file() {
            Ok(_) => d_res_tx_c_c.send(("Success".to_string(), time.elapsed())),
            Err(e) => d_res_tx_c_c.send((e, time.elapsed())),
          }
          .unwrap();
        })
        .unwrap();

      password_c.set_text("");
    });

    let aps_c = aps.clone();
    let window_c = window.clone();
    let grid_c = grid.clone();
    let progress_c = progress.clone();
    let input_c = input.clone();
    let output_c = output.clone();
    let browse_o_btn_c = browse_o_btn.clone();

    // Use glib::source::idle_add to update GUI from main thread
    gtk::glib::source::idle_add_local(move || {
      {
        let same_dir = aps_c.read().settings.same_dir;
        // output_c.set_visible(!same_dir);
        // browse_o_btn_c.set_visible(!same_dir);
        output_c.set_sensitive(!same_dir);
        browse_o_btn_c.set_sensitive(!same_dir);
      }

      if let Ok(prog) = progress_r.try_recv() {
        progress_c.set_fraction(prog);
      }

      if let Ok(prog) = bench_prog_r.try_recv() {
        progress_c.set_fraction(prog);
      }

      let mut input_v = input_c.text().to_string();
      input_v.retain(|c| c != '"' && c != '\'');
      let input_path = std::path::PathBuf::from(input_v);

      if let Ok((msg, dur)) = e_res_rx.try_recv() {
        grid_c.set_sensitive(true);
        progress_c.set_fraction(0.0);
        if msg == "Success" {
          if aps_c.read().settings.remove_org {
            if let Err(e) = Global::del_path(input_path.clone()) {
              GTKhelper::message_box(&window_c, "Error", e, None);
            };
          }

          GTKhelper::message_box(&window_c, msg, format!("File encrypted in {}", Global::format_duration(dur)), None);
        } else {
          GTKhelper::message_box(&window_c, "Failed", msg, None);
        }
      }

      if let Ok((msg, dur)) = d_res_rx.try_recv() {
        grid_c.set_sensitive(true);
        progress_c.set_fraction(0.0);
        if msg == "Success" {
          if aps_c.read().settings.remove_org {
            if let Err(e) = Global::del_path(input_path.clone()) {
              GTKhelper::message_box(&window_c, "Error", e, None);
            };
          }

          GTKhelper::message_box(&window_c, msg, format!("File decrypted in {}", Global::format_duration(dur)), None);
        } else {
          GTKhelper::message_box(&window_c, "Failed", msg, None);
        }
      }

      gtk::glib::ControlFlow::Continue
    });

    // {
    //   let aps_c = aps.clone();
    //   let window_c = window.clone();

    //   let test_btn = gtk::Button::with_label("🧪☢️🧪");
    //   test_btn.set_tooltip_text(Some("TEST ONLY"));
    //   test_btn.connect_clicked(move |_| match Global::is_elevated() {
    //     Ok(v) => GTKhelper::message_box(&window_c, "Elevated", v.to_string(), None),
    //     Err(e) => GTKhelper::message_box(&window_c, "Function failed", e, None),
    //   });
    //   grid.attach(&test_btn, 0, 5, 3, 1);
    // }

    window.present();

    #[cfg(target_os = "windows")]
    GTKhelper::centre_to_screen(&window).unwrap();
  };

  let setup_app_c = setup_app.clone();
  application.connect_activate(move |app| {
    setup_app_c(app, None);
  });

  let setup_app_c = setup_app.clone();
  // Connect to the "open" signal to handle file opening
  application.connect_open(move |app, files, _hint| {
    // for file in files {
    //   if let Some(path) = file.path() {
    //     println!("Opening file: {}", path.display());
    //     // Handle the file here
    //   }
    // }
    let file_path = files.first().and_then(|f| f.path());
    setup_app_c(app, file_path);
  });

  application.run()
}
