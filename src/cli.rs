use crate::{logic::encryption::ShinCrypt, memory::MB64};
use clap::{Parser, Subcommand};
use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
  #[arg(short, long, alias = "--quiet", alias = "-q")]
  quiet: bool,

  #[arg(short, long, alias = "--remove", alias = "-r")]
  remove: bool,

  #[command(subcommand)]
  command: Option<Command>,

  file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Encrypt a file
  #[command(alias = "e", alias = "--encrypt", alias = "-e")] // allows  "encrypt" "e" "--encrypt" "-e"
  Encrypt { password: String, path: String, optional_path: Option<String> },

  /// Decrypt a file
  #[command(alias = "d", alias = "--decrypt", alias = "-d")] // allows  "decrypt" "d" "--decrypt" "-d"
  Decrypt { password: String, path: String, optional_path: Option<String> },
}

pub fn cli() -> bool {
  unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
  let args = Args::parse();

  let (tx, rx) = crossbeam::channel::unbounded::<f64>();

  // let in_path;

  match args.command {
    Some(Command::Encrypt { password, path, optional_path }) => {
      // in_path = std::path::PathBuf::from(path.clone());
      if let Some(mut shincrypt) = con_shincrypt(password, path, optional_path) {
        shincrypt.set_progres(Some(tx));
        std::thread::Builder::new().stack_size(MB64).spawn(move || shincrypt.encrypt_file().unwrap()).unwrap();
      }
    }
    Some(Command::Decrypt { password, path, optional_path }) => {
      // in_path = std::path::PathBuf::from(path.clone());
      if let Some(mut shincrypt) = con_shincrypt(password, path, optional_path) {
        shincrypt.set_progres(Some(tx));
        std::thread::Builder::new().stack_size(MB64).spawn(move || shincrypt.decrypt_file().unwrap()).unwrap();
      }
    }
    _ => return false,
  }

  if !args.quiet {
    for progress in rx {
      print!("\r{:width$}", "", width = 80); // clear line
      print!("\rProgress: {:.0}%", progress * 100.0);
      std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    println!("\n✅ Completed successfully.");
  }

  // if args.remove {
  //   Global::del_path(in_path).unwrap();
  // }

  true
}

///construct shincrypt
fn con_shincrypt(password: String, path: String, optional_path: Option<String>) -> Option<ShinCrypt> {
  let input_path = std::path::PathBuf::from(path.clone());
  if !input_path.exists() {
    eprintln!("Invalid path: {}", path);
    return None;
  }
  let mut output_dir = input_path.parent().unwrap().to_path_buf();

  if let Some(path) = optional_path {
    output_dir = std::path::PathBuf::from(path);
    // println!("Optional Path: {}", path);
  }

  Some(ShinCrypt::new(input_path, output_dir, password, None))
}
