mod events;
mod game;
mod x52pro;

use events::Event;
use game::file::Status;
use game::Ship;
use hotwatch::Hotwatch;
use std::sync::mpsc;
use std::time::Duration;
use x52pro::DirectOutput;

const VERSION: &str = "1.1";

pub fn run() {
    println!("EDXLC {}", VERSION);
    println!("Press Ctrl+C to exit");

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
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_millis(100))
        .expect("File watcher failed to initialize");

    hotwatch
        .watch(status_file_path, move |event: hotwatch::Event| {
            if let hotwatch::Event::Write(path) = event {
                if let Some(status) = Status::from_file(&path) {
                    tx.send(Event::StatusUpdate(status))
                        .expect("Could not send status update message");
                }
            }
        })
        .expect("Failed to watch status file");

    ctrlc::set_handler(move || {
        println!("Received Ctrl+C");
        tx2.send(Event::Exit).expect("Could not send exit message");
    })
    .expect("Failed to set Ctrl+C handler");

    for event in rx {
        match event {
            Event::Exit => break,
            Event::StatusUpdate(status) => {
                if ship.update_status(status) {
                    println!("Landing gear deployed: {}", ship.landing_gear_deployed());

                    if ship.landing_gear_deployed() {
                        direct_output.set_led(9, true);
                        direct_output.set_led(10, true);
                    } else {
                        direct_output.set_led(9, false);
                        direct_output.set_led(10, true);
                    }

                    if ship.cargo_scoop_deployed() {
                        direct_output.set_led(11, true);
                        direct_output.set_led(12, true);
                    } else {
                        direct_output.set_led(11, false);
                        direct_output.set_led(12, true);
                    }
                } else {
                    println!("Status file updated but change not relevant");
                }
            }
        }
    }

    println!("Exiting");
}
