use crate::x52pro::device::{
    BooleanLightState, LightState, RedAmberGreenLightMode, RedAmberGreenLightState,
};
use crate::x52pro::direct_output::DirectOutput;
use std::time::SystemTime;

pub const ALERT_FLASH_MILLISECONDS: u128 = 500;

/// Maps light modes to light states. The returned states change over time
/// because certain modes are animated, i.e. flashing.
pub struct LightModeToStateMapper {
    reference_time: SystemTime,
}

impl LightModeToStateMapper {
    /// Returns a new instance the mapper.
    pub fn new() -> Self {
        Self {
            reference_time: SystemTime::now(),
        }
    }

    /// Returns the light state corrsponding to the given light mode at the
    /// current moment in time.
    //
    // Could take a closure here instead that provides a pre-computed hash with
    // states keyed by modes so that animated states need only be calculated
    // once.
    pub fn map(&self, light_mode: &RedAmberGreenLightMode) -> LightState {
        let milliseconds = self.reference_time.elapsed().unwrap().as_millis();
        light_state_for_mode(light_mode, milliseconds)
    }

    /// Sets the given device LED to the correct state based on the given mode.
    pub fn update_binary_light(
        &self,
        direct_output: &DirectOutput,
        light_mode: &RedAmberGreenLightMode,
        led_id: u32,
    ) {
        let light_state = self.map(&light_mode);
        let led_active = match light_state.boolean {
            BooleanLightState::Off => false,
            BooleanLightState::On => true,
        };

        direct_output.set_led(led_id, led_active);
    }

    /// Sets the given device LEDs to the correct state based on the given mode.
    pub fn update_red_amber_green_light(
        &self,
        direct_output: &DirectOutput,
        light_mode: &RedAmberGreenLightMode,
        red_led_id: u32,
        green_led_id: u32,
    ) {
        let light_state = self.map(light_mode);
        let (red_led_state, green_led_state) = match light_state.red_amber_green {
            RedAmberGreenLightState::Off => (false, false),
            RedAmberGreenLightState::Red => (true, false),
            RedAmberGreenLightState::Amber => (true, true),
            RedAmberGreenLightState::Green => (false, true),
        };

        direct_output.set_led(red_led_id, red_led_state);
        direct_output.set_led(green_led_id, green_led_state);
    }
}

/// Returns the light state that corrsponds to the given light mode at the
/// given time offset (in milliseconds).
fn light_state_for_mode(light_mode: &RedAmberGreenLightMode, milliseconds: u128) -> LightState {
    match light_mode {
        RedAmberGreenLightMode::Off => {
            LightState::new(RedAmberGreenLightState::Off, BooleanLightState::Off)
        }
        RedAmberGreenLightMode::Red => {
            LightState::new(RedAmberGreenLightState::Red, BooleanLightState::On)
        }
        RedAmberGreenLightMode::Amber => {
            LightState::new(RedAmberGreenLightState::Amber, BooleanLightState::On)
        }
        RedAmberGreenLightMode::Green => {
            LightState::new(RedAmberGreenLightState::Green, BooleanLightState::On)
        }
        RedAmberGreenLightMode::FlashingRedAmber => {
            if (milliseconds / ALERT_FLASH_MILLISECONDS) & 1 == 0 {
                LightState::new(RedAmberGreenLightState::Red, BooleanLightState::On)
            } else {
                LightState::new(RedAmberGreenLightState::Amber, BooleanLightState::Off)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_light_state_for_mode(
        light_mode: RedAmberGreenLightMode,
        milliseconds: u128,
        red_amber_green: RedAmberGreenLightState,
        boolean: BooleanLightState,
    ) {
        let light_state = LightState::new(red_amber_green, boolean);
        assert_eq!(light_state_for_mode(&light_mode, milliseconds), light_state);
    }

    #[test]
    fn light_state_for_mode_permutations() {
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Off,
            0,
            RedAmberGreenLightState::Off,
            BooleanLightState::Off,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Off,
            ALERT_FLASH_MILLISECONDS,
            RedAmberGreenLightState::Off,
            BooleanLightState::Off,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Red,
            0,
            RedAmberGreenLightState::Red,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Red,
            ALERT_FLASH_MILLISECONDS,
            RedAmberGreenLightState::Red,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Amber,
            0,
            RedAmberGreenLightState::Amber,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Amber,
            ALERT_FLASH_MILLISECONDS,
            RedAmberGreenLightState::Amber,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Green,
            0,
            RedAmberGreenLightState::Green,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::Green,
            ALERT_FLASH_MILLISECONDS,
            RedAmberGreenLightState::Green,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::FlashingRedAmber,
            0,
            RedAmberGreenLightState::Red,
            BooleanLightState::On,
        );
        assert_light_state_for_mode(
            RedAmberGreenLightMode::FlashingRedAmber,
            ALERT_FLASH_MILLISECONDS,
            RedAmberGreenLightState::Amber,
            BooleanLightState::Off,
        );
    }
}
