mod file;

use file::Status;
use hotwatch::blocking::{Flow, Hotwatch};
use hotwatch::Event;
use std::time::Duration;

pub fn run() {
    let status_file_path = file::status_file_path();
    println!("Status file path: {:?}", status_file_path);

    let mut current_status =
        Status::from_file(&status_file_path).expect("Could not read current status");

    let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_millis(100))
        .expect("File watcher failed to initialize");

    hotwatch
        .watch(status_file_path, move |event: Event| {
            if let Event::Write(path) = event {
                println!("Status file written");
                if let Some(updated_status) = Status::from_file(&path) {
                    if updated_status != current_status {
                        println!(
                            "Status flags changed from {} to {}",
                            current_status.flags, updated_status.flags
                        );
                        current_status = updated_status;
                    }
                }
            }
            Flow::Continue
        })
        .expect("Failed to watch status file");

    hotwatch.run();
}
