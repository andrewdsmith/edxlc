use crate::x52pro::direct_output::DirectOutput;

/// An instance of an interface to a Saitek X52 Pro Flight HOTAS flight
/// controller device.
pub struct Device {
    direct_output: DirectOutput,
}

impl Device {
    /// Returns a new instance the the device interface. Panics if the
    /// underlying DirectOutput instance cannot be loaded.
    pub fn new() -> Self {
        let mut direct_output = DirectOutput::load();
        direct_output.initialize();
        direct_output.enumerate();
        direct_output.add_page();

        Device {
            direct_output: direct_output,
        }
    }

    /// Set the given LED to the specified state.
    pub fn set_led_state(&self, led: LED, state: LEDState) {
        let (red_led_id, green_led_id) = match led {
            LED::T1T2 => (9, 10),
            LED::T3T4 => (11, 12),
            LED::T5T6 => (13, 14),
        };

        let (red_led_state, green_led_state) = match state {
            LEDState::Red => (true, false),
            LEDState::Amber => (true, true),
            LEDState::Green => (false, true),
        };

        self.direct_output.set_led(red_led_id, red_led_state);
        self.direct_output.set_led(green_led_id, green_led_state);
    }
}

/// Supported input buttons or axes on the device.
#[derive(Debug, PartialEq)]
pub enum Input {
    T2,
    T4,
    T6,
}

/// Controllable LEDs on the devive.
#[derive(Debug, PartialEq)]
pub enum LED {
    T1T2,
    T3T4,
    T5T6,
}

/// Available states for LEDs on the device.
pub enum LEDState {
    Red,
    Amber,
    Green,
}

/// Returns the LED that corresponds to a given input. Note that in some cases,
/// specifically the T buttons, multiple inputs share an LED.
pub fn led_for_input(input: Input) -> LED {
    match input {
        Input::T2 => LED::T1T2,
        Input::T4 => LED::T3T4,
        Input::T6 => LED::T5T6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_maps_input_to_led() {
        assert_eq!(led_for_input(Input::T2), LED::T1T2);
        assert_eq!(led_for_input(Input::T4), LED::T3T4);
        assert_eq!(led_for_input(Input::T6), LED::T5T6);
    }
}
