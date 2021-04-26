mod events;
mod game;
mod x52pro;

use events::Event;
use game::file::Status;
use game::{Attribute, Control, Controls, Ship, StatusLevel};
use hotwatch::Hotwatch;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use x52pro::device::{LEDState, LED};

const VERSION: &str = "1.4";
const ANIMATION_TICK_MILLISECONDS: u64 = x52pro::device::ALERT_FLASH_MILLISECONDS as u64;

pub fn run() {
    println!("EDXLC {}", VERSION);
    println!("Press Ctrl+C to exit");

    let mut x52pro = x52pro::Device::new();

    // Set LED red initially until the first update in status.
    x52pro.set_led_state(LED::T1T2, LEDState::Red);

    let bindings_file_path = game::file::bindings_file_path();
    println!("Bindings file path: {:?}", bindings_file_path);

    let controls = Controls::from_file(&bindings_file_path);
    println!("Controls: {:?}", controls);

    let status_file_path = game::file::status_file_path();
    println!("Status file path: {:?}", status_file_path);

    let initial_status =
        Status::from_file(&status_file_path).expect("Could not read current status");

    let mut ship = Ship::from_status(initial_status);
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
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

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(ANIMATION_TICK_MILLISECONDS));
        tx3.send(Event::AnimationTick)
            .expect("Could not send animation tick message");
    });

    for event in rx {
        match event {
            Event::Exit => break,
            Event::AnimationTick => {
                x52pro.update_animated_leds();
            }
            Event::StatusUpdate(status) => {
                if ship.update_status(status) {
                    // Here we build a hash of status levels for each input. An input may represent
                    // multiple ship statuses because it is bound to multiple controls. The hash
                    // value stores the highest precendence status level found for the input.

                    fn controls_for_status(status: &game::Status) -> Vec<Control> {
                        match status.attribute {
                            Attribute::CargoScoop => vec![Control::CargoScoop],
                            Attribute::ExternalLights => vec![Control::ExternalLights],
                            Attribute::FrameShiftDrive => vec![
                                Control::Hyperspace,
                                Control::HyperSuperCombination,
                                Control::Supercruise,
                            ],
                            Attribute::HeatSink => vec![Control::HeatSink],
                            Attribute::LandingGear => vec![Control::LandingGear],
                        }
                    }

                    let mut input_states = HashMap::new();

                    for status in ship.statuses() {
                        for control in controls_for_status(&status) {
                            let inputs = controls.inputs_for_control(control);

                            for input in inputs {
                                let input_status_level =
                                    input_states.entry(input).or_insert(StatusLevel::Inactive);

                                set_input_status_level_if_level(
                                    &status,
                                    input_status_level,
                                    StatusLevel::Active,
                                );
                                set_input_status_level_if_level(
                                    &status,
                                    input_status_level,
                                    StatusLevel::Blocked,
                                );
                                set_input_status_level_if_level(
                                    &status,
                                    input_status_level,
                                    StatusLevel::Alert,
                                );

                                fn set_input_status_level_if_level(
                                    status: &game::Status,
                                    input_status_level: &mut StatusLevel,
                                    level: StatusLevel,
                                ) {
                                    if status.level == level && *input_status_level != level {
                                        *input_status_level = level
                                    }
                                }
                            }
                        }
                    }

                    for (input, status_level) in input_states {
                        x52pro.set_input_status_level(input, status_level);
                    }
                } else {
                    println!("Status file updated but change not relevant");
                }
            }
        }
    }

    println!("Exiting");
}
