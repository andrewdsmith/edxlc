use dirs;
use std::path::PathBuf;

pub fn run() {
    println!("Status file path: {:?}", status_file_path());
}

fn status_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous\Status.json"#)
}
