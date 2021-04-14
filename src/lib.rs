mod events;
mod game;
mod x52pro;

use events::Event;
use game::file::Status;
use game::Ship;
use hotwatch::Hotwatch;
use std::sync::mpsc;
use std::time::Duration;
use x52pro::device::{LEDState, LED};

const VERSION: &str = "1.2";

pub fn run() {
    println!("EDXLC {}", VERSION);
    println!("Press Ctrl+C to exit");

    let x52pro = x52pro::Device::new();

    // Set LED red initially until the first update in status.
    x52pro.set_led_state(LED::T1T2, LEDState::Red);

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
                    fn led_state(state: bool) -> LEDState {
                        match state {
                            true => LEDState::Amber,
                            false => LEDState::Green,
                        }
                    }

                    x52pro.set_led_state(LED::T1T2, led_state(ship.landing_gear_deployed()));
                    x52pro.set_led_state(LED::T3T4, led_state(ship.cargo_scoop_deployed()));
                    x52pro.set_led_state(LED::T5T6, led_state(ship.external_lights_on()));
                } else {
                    println!("Status file updated but change not relevant");
                }
            }
        }
    }

    println!("Exiting");
}
