use dirs;
use hotwatch::blocking::{Flow, Hotwatch};
use hotwatch::Event;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub fn run() {
    let status_file_path = status_file_path();
    println!("Status file path: {:?}", status_file_path);

    let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_millis(100))
        .expect("File watcher failed to initialize");

    hotwatch
        .watch(status_file_path, |event: Event| {
            if let Event::Write(path) = event {
                println!("{:?} has changed", path);
                let status = fs::read_to_string(path).unwrap();
                println!("{}", status);
            }
            Flow::Continue
        })
        .expect("Failed to watch status file");

    hotwatch.run();
}

fn status_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous\Status.json"#)
}
