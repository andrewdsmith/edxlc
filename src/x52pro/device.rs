use crate::game::StatusLevel;
use crate::x52pro::direct_output::DirectOutput;
use std::collections::HashMap;
use std::time::SystemTime;

pub const ALERT_FLASH_MILLISECONDS: u128 = 500;

/// An instance of an interface to a Saitek X52 Pro Flight HOTAS flight
/// controller device.
pub struct Device {
    direct_output: DirectOutput,
    led_status_levels: HashMap<LED, StatusLevel>,
    status_level_mapper: StatusLevelMapper,
}

impl Device {
    /// Returns a new instance of the device interface. Panics if the
    /// underlying `DirectOutput` instance cannot be loaded.
    pub fn new() -> Self {
        let mut direct_output = DirectOutput::load();
        direct_output.initialize();
        direct_output.enumerate();
        direct_output.add_page();

        Device {
            direct_output: direct_output,
            led_status_levels: HashMap::new(),
            status_level_mapper: StatusLevelMapper::new(),
        }
    }

    /// Sets each input to specified status level. Repeated inputs with
    /// different status levels are handled by using the highest value. The LED
    /// for the input is looked up, as is the LED state for the status level.
    pub fn set_input_status_levels(&mut self, input_status_levels: Vec<(Input, StatusLevel)>) {
        // Build a hash of the highest status level keyed by led.
        let mut led_highest_status_levels = HashMap::new();

        for (input, status_level) in input_status_levels {
            let led = led_for_input(input);
            let led_status_level = led_highest_status_levels
                .entry(led)
                .or_insert(StatusLevel::Inactive);

            // Replace this with `and_modify` above?
            if status_level > *led_status_level {
                *led_status_level = status_level.clone();
            }
        }

        for (led, status_level) in &led_highest_status_levels {
            self.set_led_status_level(&led, &status_level);
        }

        self.led_status_levels = led_highest_status_levels;
    }

    /// Set the given LED to the specified status level.
    fn set_led_status_level(&self, led: &LED, status_level: &StatusLevel) {
        let (red_led_id, green_led_id) = match led {
            LED::Clutch => (17, 18),
            LED::FireA => (1, 2),
            LED::FireB => (3, 4),
            LED::FireD => (5, 6),
            LED::FireE => (7, 8),
            LED::T1T2 => (9, 10),
            LED::T3T4 => (11, 12),
            LED::T5T6 => (13, 14),
        };

        let state = self.status_level_mapper.led_state(status_level);
        let (red_led_state, green_led_state) = match state {
            LEDState::Red => (true, false),
            LEDState::Amber => (true, true),
            LEDState::Green => (false, true),
            _ => panic!("Unable to set unmapped LED state"),
        };

        self.direct_output.set_led(red_led_id, red_led_state);
        self.direct_output.set_led(green_led_id, green_led_state);
    }

    /// Updates LEDs that have a state that is animated, e.g. flashing. This
    /// needs to be called frequently for proper animation.
    //
    // Ideally the device would manage its own threading for animation but
    // this would require state updates to be communicated asynchronously.
    pub fn update_animated_leds(&self) {
        for (led, status_level) in &self.led_status_levels {
            // Could query the mapper to see if the LED is in an animated state
            // and only update it if necessary here.
            self.set_led_status_level(led, status_level);
        }
    }
}

/// Supported input buttons or axes on the device.
#[derive(Debug, PartialEq)]
pub enum Input {
    Clutch,
    FireA,
    FireB,
    FireD,
    FireE,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
}

/// Controllable LEDs on the devive.
#[derive(Debug, Eq, Hash, PartialEq)]
enum LED {
    Clutch,
    FireA,
    FireB,
    FireD,
    FireE,
    T1T2,
    T3T4,
    T5T6,
}

/// Available states for LEDs on the device.
#[derive(Copy, Clone, Debug, PartialEq)]
enum LEDState {
    Red,
    Amber,
    Green,
    FlashingRedAmber,
}

/// Returns the LED that corresponds to a given input. Note that in some cases,
/// specifically the T buttons, multiple inputs share an LED.
fn led_for_input(input: Input) -> LED {
    match input {
        Input::Clutch => LED::Clutch,
        Input::FireA => LED::FireA,
        Input::FireB => LED::FireB,
        Input::FireD => LED::FireD,
        Input::FireE => LED::FireE,
        Input::T1 => LED::T1T2,
        Input::T2 => LED::T1T2,
        Input::T3 => LED::T3T4,
        Input::T4 => LED::T3T4,
        Input::T5 => LED::T5T6,
        Input::T6 => LED::T5T6,
    }
}

/// An mapper that returns an `LEDState` for a given `StateLevel`. The mapping
/// depends on time to support animated (flashing) states.
struct StatusLevelMapper {
    inactive: LEDState,
    active: LEDState,
    blocked: LEDState,
    alert: LEDState,
    reference_time: SystemTime,
}

impl StatusLevelMapper {
    /// Returns a new instance the mapper.
    pub fn new() -> Self {
        Self {
            inactive: LEDState::Green,
            active: LEDState::Amber,
            blocked: LEDState::Red,
            alert: LEDState::FlashingRedAmber,
            reference_time: SystemTime::now(),
        }
    }

    /// Returns the LED state that corrsponds to a given status level.
    //
    // Could take a closure here instead that passes in a hash mapping state
    // levels to LED states, meaning animated states need only be calculated
    // once.
    fn led_state(&self, status_level: &StatusLevel) -> LEDState {
        let led_state = match status_level {
            StatusLevel::Inactive => self.inactive,
            StatusLevel::Active => self.active,
            StatusLevel::Blocked => self.blocked,
            StatusLevel::Alert => self.alert,
        };

        if led_state == LEDState::FlashingRedAmber {
            let millis = self.reference_time.elapsed().unwrap().as_millis();
            if (millis / ALERT_FLASH_MILLISECONDS) & 1 == 0 {
                LEDState::Red
            } else {
                LEDState::Amber
            }
        } else {
            led_state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_to_led_permutations() {
        assert_led_for_input(Input::Clutch, LED::Clutch);
        assert_led_for_input(Input::FireA, LED::FireA);
        assert_led_for_input(Input::FireB, LED::FireB);
        assert_led_for_input(Input::FireD, LED::FireD);
        assert_led_for_input(Input::FireE, LED::FireE);
        assert_led_for_input(Input::T1, LED::T1T2);
        assert_led_for_input(Input::T2, LED::T1T2);
        assert_led_for_input(Input::T3, LED::T3T4);
        assert_led_for_input(Input::T4, LED::T3T4);
        assert_led_for_input(Input::T5, LED::T5T6);
        assert_led_for_input(Input::T6, LED::T5T6);
    }

    fn assert_led_for_input(input: Input, led: LED) {
        assert_eq!(led_for_input(input), led);
    }

    #[test]
    fn status_level_for_led_state_permutations() {
        assert_led_state_for_status_level(StatusLevel::Inactive, LEDState::Green);
        assert_led_state_for_status_level(StatusLevel::Active, LEDState::Amber);
        assert_led_state_for_status_level(StatusLevel::Blocked, LEDState::Red);
    }

    fn assert_led_state_for_status_level(status_level: StatusLevel, led_state: LEDState) {
        let status_level_mapper = StatusLevelMapper::new();
        assert_eq!(status_level_mapper.led_state(&status_level), led_state);
    }
}
