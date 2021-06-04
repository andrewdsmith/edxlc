use crate::game::StatusLevel;
use crate::x52pro::direct_output::DirectOutput;
use std::collections::HashMap;
use std::time::SystemTime;

pub const ALERT_FLASH_MILLISECONDS: u128 = 500;

const LED_CLUTCH_RED: u32 = 17;
const LED_CLUTCH_GREEN: u32 = 18;
const LED_FIRE: u32 = 0;
const LED_FIRE_A_RED: u32 = 1;
const LED_FIRE_A_GREEN: u32 = 2;
const LED_FIRE_B_RED: u32 = 3;
const LED_FIRE_B_GREEN: u32 = 4;
const LED_FIRE_D_RED: u32 = 5;
const LED_FIRE_D_GREEN: u32 = 6;
const LED_FIRE_E_RED: u32 = 7;
const LED_FIRE_E_GREEN: u32 = 8;
const LED_T1T2_RED: u32 = 9;
const LED_T1T2_GREEN: u32 = 10;
const LED_T3T4_RED: u32 = 11;
const LED_T3T4_GREEN: u32 = 12;
const LED_T5T6_RED: u32 = 13;
const LED_T5T6_GREEN: u32 = 14;

/// An instance of an interface to a Saitek X52 Pro Flight HOTAS flight
/// controller device.
pub struct Device {
    direct_output: DirectOutput,
    animated_light_status_levels: HashMap<Light, StatusLevel>,
    status_level_mapper: StatusLevelMapper,
}

impl Device {
    /// Returns a new instance of the device interface. Panics if the
    /// underlying `DirectOutput` instance cannot be loaded.
    pub fn new(status_level_mapper: StatusLevelMapper) -> Self {
        let mut direct_output = DirectOutput::load();
        direct_output.initialize();
        direct_output.enumerate();
        direct_output.add_page();

        Device {
            direct_output,
            animated_light_status_levels: HashMap::new(),
            status_level_mapper,
        }
    }

    /// Sets each input to specified status level. Repeated inputs with
    /// different status levels are handled by using the highest value. The Light
    /// for the input is looked up, as is the Light state for the status level.
    pub fn set_input_status_levels(&mut self, input_status_levels: Vec<(Input, StatusLevel)>) {
        // Build a hash of the highest status level keyed by light.
        let mut light_highest_status_levels = HashMap::new();

        for (input, status_level) in input_status_levels {
            let light = light_for_input(input);
            let light_status_level = light_highest_status_levels
                .entry(light)
                .or_insert(StatusLevel::Inactive);

            // Replace this with `and_modify` above?
            if status_level > *light_status_level {
                *light_status_level = status_level.clone();
            }
        }

        for (light, status_level) in &light_highest_status_levels {
            self.set_light_status_level(&light, &status_level);
        }

        light_highest_status_levels.retain(|_, &mut status_level| {
            self.status_level_mapper
                .status_level_is_animated(&status_level)
        });

        self.animated_light_status_levels = light_highest_status_levels;
    }

    /// Set the given Light to the specified status level.
    fn set_light_status_level(&self, light: &Light, status_level: &StatusLevel) {
        // Should cache these mappings in hash in the constructor so they can
        // be reused.
        let led_mapping = match light {
            Light::Clutch => LEDMapping::RedGreen(LED_CLUTCH_RED, LED_CLUTCH_GREEN),
            Light::Fire => LEDMapping::OnOff(LED_FIRE),
            Light::FireA => LEDMapping::RedGreen(LED_FIRE_A_RED, LED_FIRE_A_GREEN),
            Light::FireB => LEDMapping::RedGreen(LED_FIRE_B_RED, LED_FIRE_B_GREEN),
            Light::FireD => LEDMapping::RedGreen(LED_FIRE_D_RED, LED_FIRE_D_GREEN),
            Light::FireE => LEDMapping::RedGreen(LED_FIRE_E_RED, LED_FIRE_E_GREEN),
            Light::T1T2 => LEDMapping::RedGreen(LED_T1T2_RED, LED_T1T2_GREEN),
            Light::T3T4 => LEDMapping::RedGreen(LED_T3T4_RED, LED_T3T4_GREEN),
            Light::T5T6 => LEDMapping::RedGreen(LED_T5T6_RED, LED_T5T6_GREEN),
        };

        let light_state = self.status_level_mapper.light_state(status_level);
        led_mapping.set_leds_to_state(&self.direct_output, light_state);
    }

    /// Updates lights that have a state that is animated, e.g. flashing. This
    /// needs to be called frequently for proper animation.
    //
    // Ideally the device would manage its own threading for animation but
    // this would require state updates to be communicated asynchronously.
    pub fn update_animated_lights(&self) {
        for (light, status_level) in &self.animated_light_status_levels {
            self.set_light_status_level(light, status_level);
        }
    }
}

/// Supported input buttons or axes on the device.
#[derive(Debug, PartialEq)]
pub enum Input {
    Clutch,
    Fire,
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

/// Controllable lights on the device.
#[derive(Debug, Eq, Hash, PartialEq)]
enum Light {
    Clutch,
    Fire,
    FireA,
    FireB,
    FireD,
    FireE,
    T1T2,
    T3T4,
    T5T6,
}

/// Available states for lights on the device.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RedAmberGreenLightMode {
    Off,
    Red,
    Amber,
    Green,
    FlashingRedAmber,
}

/// Available final, unanimated states for lights on the device.
#[derive(Debug, PartialEq)]
pub enum RedAmberGreenLightState {
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

/// The state for a light of unknown type, holding the states for both boolean
/// and red/amber/green lights.
#[derive(Debug, PartialEq)]
struct LightState {
    pub boolean: BooleanLightState,
    pub red_amber_green: RedAmberGreenLightState,
}

impl LightState {
    /// Returns a new `LightState` instance.
    fn new(red_amber_green: RedAmberGreenLightState, boolean: BooleanLightState) -> Self {
        Self {
            red_amber_green,
            boolean,
        }
    }
}

/// Logical sets of LEDS ids the combine to provide different colours. This
/// will be extended with a `Single` type to support controls like the Fire
/// button and the throttle.
enum LEDMapping {
    OnOff(u32),
    RedGreen(u32, u32),
}

impl LEDMapping {
    /// Sets the mapped LEDS to the given state.
    fn set_leds_to_state(self, direct_output: &DirectOutput, light_state: LightState) {
        match self {
            Self::OnOff(led_id) => {
                let led_active = match light_state.boolean {
                    BooleanLightState::Off => false,
                    BooleanLightState::On => true,
                };

                direct_output.set_led(led_id, led_active);
            }
            Self::RedGreen(red_led_id, green_led_id) => {
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
    }
}

/// Returns the Light that corresponds to a given input. Note that in some cases,
/// specifically the T buttons, multiple inputs share an Light.
fn light_for_input(input: Input) -> Light {
    match input {
        Input::Clutch => Light::Clutch,
        Input::Fire => Light::Fire,
        Input::FireA => Light::FireA,
        Input::FireB => Light::FireB,
        Input::FireD => Light::FireD,
        Input::FireE => Light::FireE,
        Input::T1 => Light::T1T2,
        Input::T2 => Light::T1T2,
        Input::T3 => Light::T3T4,
        Input::T4 => Light::T3T4,
        Input::T5 => Light::T5T6,
        Input::T6 => Light::T5T6,
    }
}

/// An mapper that returns an `RedAmberGreenLightMode` for a given `StateLevel`. The mapping
/// depends on time to support animated (flashing) states.
pub struct StatusLevelMapper {
    inactive: RedAmberGreenLightMode,
    active: RedAmberGreenLightMode,
    blocked: RedAmberGreenLightMode,
    alert: RedAmberGreenLightMode,
    reference_time: SystemTime,
}

impl StatusLevelMapper {
    /// Returns a new instance the mapper.
    pub fn new(
        inactive: RedAmberGreenLightMode,
        active: RedAmberGreenLightMode,
        blocked: RedAmberGreenLightMode,
        alert: RedAmberGreenLightMode,
    ) -> Self {
        Self {
            inactive,
            active,
            blocked,
            alert,
            reference_time: SystemTime::now(),
        }
    }

    /// Returns the Light state that corrsponds to a given status level.
    //
    // Could take a closure here instead that passes in a hash mapping state
    // levels to Light states, meaning animated states need only be calculated
    // once.
    fn light_state(&self, status_level: &StatusLevel) -> LightState {
        let light_mode = self.light_mode_for_status_level(status_level);

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
                let millis = self.reference_time.elapsed().unwrap().as_millis();
                if (millis / ALERT_FLASH_MILLISECONDS) & 1 == 0 {
                    LightState::new(RedAmberGreenLightState::Red, BooleanLightState::On)
                } else {
                    LightState::new(RedAmberGreenLightState::Amber, BooleanLightState::Off)
                }
            }
        }
    }

    /// Returns true if the given status level is configured to an animated
    /// state.
    fn status_level_is_animated(&self, status_level: &StatusLevel) -> bool {
        self.light_mode_for_status_level(status_level) == RedAmberGreenLightMode::FlashingRedAmber
    }

    fn light_mode_for_status_level(&self, status_level: &StatusLevel) -> RedAmberGreenLightMode {
        match status_level {
            StatusLevel::Inactive => self.inactive,
            StatusLevel::Active => self.active,
            StatusLevel::Blocked => self.blocked,
            StatusLevel::Alert => self.alert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_to_light_permutations() {
        assert_light_for_input(Input::Clutch, Light::Clutch);
        assert_light_for_input(Input::Fire, Light::Fire);
        assert_light_for_input(Input::FireA, Light::FireA);
        assert_light_for_input(Input::FireB, Light::FireB);
        assert_light_for_input(Input::FireD, Light::FireD);
        assert_light_for_input(Input::FireE, Light::FireE);
        assert_light_for_input(Input::T1, Light::T1T2);
        assert_light_for_input(Input::T2, Light::T1T2);
        assert_light_for_input(Input::T3, Light::T3T4);
        assert_light_for_input(Input::T4, Light::T3T4);
        assert_light_for_input(Input::T5, Light::T5T6);
        assert_light_for_input(Input::T6, Light::T5T6);
    }

    fn assert_light_for_input(input: Input, light: Light) {
        assert_eq!(light_for_input(input), light);
    }

    #[test]
    fn status_level_for_light_state_permutations() {
        assert_light_state_for_status_level(StatusLevel::Inactive, RedAmberGreenLightState::Green);
        assert_light_state_for_status_level(StatusLevel::Active, RedAmberGreenLightState::Amber);
        assert_light_state_for_status_level(StatusLevel::Blocked, RedAmberGreenLightState::Red);
        assert_light_state_for_status_level(StatusLevel::Alert, RedAmberGreenLightState::Red);
    }

    fn assert_light_state_for_status_level(
        status_level: StatusLevel,
        rag_light_state: RedAmberGreenLightState,
    ) {
        let status_level_mapper = StatusLevelMapper::new(
            RedAmberGreenLightMode::Green,
            RedAmberGreenLightMode::Amber,
            RedAmberGreenLightMode::Red,
            RedAmberGreenLightMode::FlashingRedAmber,
        );
        let light_state = LightState::new(rag_light_state, BooleanLightState::On);
        assert_eq!(status_level_mapper.light_state(&status_level), light_state);
    }
}
