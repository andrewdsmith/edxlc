use crate::x52pro::device::{BooleanLightMode, Led, RedAmberGreenLightMode};
use crate::x52pro::direct_output::DirectOutput;
use std::time::SystemTime;

pub const ALERT_FLASH_MILLISECONDS: u128 = 500;

/// Available final, unanimated states for lights on the device.
#[derive(Debug, PartialEq)]
enum RedAmberGreenLightState {
    Off,
    Red,
    Amber,
    Green,
}

/// Available states for a light on the device that can be either off or on.
#[derive(Debug, PartialEq)]
enum BooleanLightState {
    Off,
    On,
}

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

    /// Sets the given device LED to the correct state based on the given mode.
    pub fn update_binary_light(
        &self,
        direct_output: &DirectOutput,
        light_mode: &BooleanLightMode,
        led_id: Led,
    ) {
        let light_state = boolean_state_for_mode(light_mode, self.milliseconds_elapsed());

        // Could move this mapping onto the enum.
        let led_active = match light_state {
            BooleanLightState::Off => false,
            BooleanLightState::On => true,
        };

        direct_output.set_led(led_id as u32, led_active);
    }

    /// Sets the given device LEDs to the correct state based on the given mode.
    pub fn update_red_amber_green_light(
        &self,
        direct_output: &DirectOutput,
        light_mode: &RedAmberGreenLightMode,
        red_led_id: Led,
        green_led_id: Led,
    ) {
        let light_state = red_amber_green_state_for_mode(light_mode, self.milliseconds_elapsed());

        // Could move this mapping onto the enum.
        let (red_led_state, green_led_state) = match light_state {
            RedAmberGreenLightState::Off => (false, false),
            RedAmberGreenLightState::Red => (true, false),
            RedAmberGreenLightState::Amber => (true, true),
            RedAmberGreenLightState::Green => (false, true),
        };

        direct_output.set_led(red_led_id as u32, red_led_state);
        direct_output.set_led(green_led_id as u32, green_led_state);
    }

    /// Returns the number of milliseconds elapsed since the reference time.
    fn milliseconds_elapsed(&self) -> u128 {
        self.reference_time.elapsed().unwrap().as_millis()
    }
}

/// Returns the boolean light state that corrsponds to the given light mode at
/// the given time offset (in milliseconds).
fn boolean_state_for_mode(light_mode: &BooleanLightMode, milliseconds: u128) -> BooleanLightState {
    match light_mode {
        BooleanLightMode::Off => BooleanLightState::Off,
        BooleanLightMode::On => BooleanLightState::On,
        BooleanLightMode::Flash => {
            animated_state(milliseconds, BooleanLightState::On, BooleanLightState::Off)
        }
    }
}

/// Returns the red/amber/green light state that corrsponds to the given light
/// mode at the given time offset (in milliseconds).
fn red_amber_green_state_for_mode(
    light_mode: &RedAmberGreenLightMode,
    milliseconds: u128,
) -> RedAmberGreenLightState {
    use RedAmberGreenLightState::*;

    match light_mode {
        RedAmberGreenLightMode::Off => Off,
        RedAmberGreenLightMode::Red => Red,
        RedAmberGreenLightMode::Amber => Amber,
        RedAmberGreenLightMode::Green => Green,
        RedAmberGreenLightMode::RedAmber => animated_state(milliseconds, Red, Amber),
        RedAmberGreenLightMode::RedFlash => animated_state(milliseconds, Red, Off),
        RedAmberGreenLightMode::RedAmberFlash => animated_state(milliseconds, Red, Amber),
        RedAmberGreenLightMode::RedGreenFlash => animated_state(milliseconds, Red, Green),
        RedAmberGreenLightMode::AmberFlash => animated_state(milliseconds, Amber, Off),
        RedAmberGreenLightMode::AmberRedFlash => animated_state(milliseconds, Amber, Red),
        RedAmberGreenLightMode::AmberGreenFlash => animated_state(milliseconds, Amber, Green),
        RedAmberGreenLightMode::GreenFlash => animated_state(milliseconds, Green, Off),
        RedAmberGreenLightMode::GreenAmberFlash => animated_state(milliseconds, Green, Amber),
        RedAmberGreenLightMode::GreenRedFlash => animated_state(milliseconds, Green, Red),
    }
}

/// Returns either the first or second state based on the elapsed milliseconds
/// given as compared to the defined interval for animation.
fn animated_state<T>(milliseconds: u128, first_state: T, second_state: T) -> T {
    if (milliseconds / ALERT_FLASH_MILLISECONDS) & 1 == 0 {
        first_state
    } else {
        second_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_boolean_mapping(
        mode: BooleanLightMode,
        state_now: BooleanLightState,
        state_later: BooleanLightState,
    ) {
        assert_eq!(boolean_state_for_mode(&mode, 0), state_now);
        assert_eq!(
            boolean_state_for_mode(&mode, ALERT_FLASH_MILLISECONDS),
            state_later
        );
    }

    #[test]
    fn boolean_light_states_for_modes() {
        use BooleanLightState::*;

        assert_boolean_mapping(BooleanLightMode::Off, Off, Off);
        assert_boolean_mapping(BooleanLightMode::On, On, On);
        assert_boolean_mapping(BooleanLightMode::Flash, On, Off);
    }

    fn assert_rag_mapping(
        mode: RedAmberGreenLightMode,
        state_now: RedAmberGreenLightState,
        state_later: RedAmberGreenLightState,
    ) {
        assert_eq!(red_amber_green_state_for_mode(&mode, 0), state_now);
        assert_eq!(
            red_amber_green_state_for_mode(&mode, ALERT_FLASH_MILLISECONDS),
            state_later
        );
    }

    #[test]
    fn red_amber_green_states_for_modes() {
        use RedAmberGreenLightState::*;

        assert_rag_mapping(RedAmberGreenLightMode::Off, Off, Off);
        assert_rag_mapping(RedAmberGreenLightMode::Red, Red, Red);
        assert_rag_mapping(RedAmberGreenLightMode::Amber, Amber, Amber);
        assert_rag_mapping(RedAmberGreenLightMode::Green, Green, Green);
        assert_rag_mapping(RedAmberGreenLightMode::RedAmber, Red, Amber);
        assert_rag_mapping(RedAmberGreenLightMode::RedFlash, Red, Off);
        assert_rag_mapping(RedAmberGreenLightMode::RedAmberFlash, Red, Amber);
        assert_rag_mapping(RedAmberGreenLightMode::RedGreenFlash, Red, Green);
        assert_rag_mapping(RedAmberGreenLightMode::AmberFlash, Amber, Off);
        assert_rag_mapping(RedAmberGreenLightMode::AmberRedFlash, Amber, Red);
        assert_rag_mapping(RedAmberGreenLightMode::AmberGreenFlash, Amber, Green);
        assert_rag_mapping(RedAmberGreenLightMode::GreenFlash, Green, Off);
        assert_rag_mapping(RedAmberGreenLightMode::GreenAmberFlash, Green, Amber);
        assert_rag_mapping(RedAmberGreenLightMode::GreenRedFlash, Green, Red);
    }
}
