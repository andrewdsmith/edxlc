mod game;
mod x52pro;

use game::file::Status;
use game::Ship;
use hotwatch::blocking::{Flow, Hotwatch};
use hotwatch::Event;
use std::time::Duration;
use x52pro::DirectOutput;

pub fn run() {
    let mut direct_output = DirectOutput::load();
    direct_output.initialize();
    direct_output.enumerate();
    direct_output.add_page();

    // Set LED red initially until the first update in status.
    direct_output.set_led(9, true);
    direct_output.set_led(10, false);

    let status_file_path = game::file::status_file_path();
    println!("Status file path: {:?}", status_file_path);

    let initial_status =
        Status::from_file(&status_file_path).expect("Could not read current status");

    let mut ship = Ship::from_status(initial_status);

    let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_millis(100))
        .expect("File watcher failed to initialize");

    hotwatch
        .watch(status_file_path, move |event: Event| {
            if let Event::Write(path) = event {
                if let Some(status) = Status::from_file(&path) {
                    if ship.update_status(status) {
                        println!("Landing gear deployed: {}", ship.landing_gear_deployed());

                        if ship.landing_gear_deployed() {
                            direct_output.set_led(9, true);
                            direct_output.set_led(10, true);
                        } else {
                            direct_output.set_led(9, false);
                            direct_output.set_led(10, true);
                        }
                    } else {
                        println!("Status file updated but change not relevant")
                    }
                }
            }
            Flow::Continue
        })
        .expect("Failed to watch status file");

    hotwatch.run();
}
