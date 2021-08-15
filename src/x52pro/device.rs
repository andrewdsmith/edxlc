use crate::game::StatusLevel;
use crate::x52pro::{direct_output::DirectOutput, LightModeToStateMapper, StatusLevelToModeMapper};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    lights: HashMap<Light, Box<dyn LightMapping>>,
    animated_lights: Vec<Light>,
    light_mode_to_state_mapper: LightModeToStateMapper,
}

impl Device {
    /// Returns a new instance of the device interface. Panics if the
    /// underlying `DirectOutput` instance cannot be loaded.
    pub fn new() -> Self {
        let mut direct_output = DirectOutput::load();
        direct_output.initialize();
        direct_output.enumerate();
        direct_output.add_page();

        let mut lights = HashMap::<Light, Box<dyn LightMapping>>::new();

        lights.insert(
            Light::Clutch,
            Box::new(RedGreenLightMapping::new(LED_CLUTCH_RED, LED_CLUTCH_GREEN)),
        );
        lights.insert(Light::Fire, Box::new(BinaryLightMapping::new(LED_FIRE)));
        lights.insert(
            Light::FireA,
            Box::new(RedGreenLightMapping::new(LED_FIRE_A_RED, LED_FIRE_A_GREEN)),
        );
        lights.insert(
            Light::FireB,
            Box::new(RedGreenLightMapping::new(LED_FIRE_B_RED, LED_FIRE_B_GREEN)),
        );
        lights.insert(
            Light::FireD,
            Box::new(RedGreenLightMapping::new(LED_FIRE_D_RED, LED_FIRE_D_GREEN)),
        );
        lights.insert(
            Light::FireE,
            Box::new(RedGreenLightMapping::new(LED_FIRE_E_RED, LED_FIRE_E_GREEN)),
        );
        lights.insert(
            Light::T1T2,
            Box::new(RedGreenLightMapping::new(LED_T1T2_RED, LED_T1T2_GREEN)),
        );
        lights.insert(
            Light::T3T4,
            Box::new(RedGreenLightMapping::new(LED_T3T4_RED, LED_T3T4_GREEN)),
        );
        lights.insert(
            Light::T5T6,
            Box::new(RedGreenLightMapping::new(LED_T5T6_RED, LED_T5T6_GREEN)),
        );

        Device {
            direct_output,
            lights,
            animated_lights: vec![],
            light_mode_to_state_mapper: LightModeToStateMapper::new(),
        }
    }

    /// Sets each input to specified status level. Repeated inputs with
    /// different status levels are handled by using the highest value. The Light
    /// for the input is looked up, as is the Light state for the status level.
    pub fn set_input_status_levels(
        &mut self,
        input_status_levels: Vec<(Input, StatusLevel)>,
        status_level_to_mode_mapper: &StatusLevelToModeMapper,
    ) {
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

        self.animated_lights.clear();

        for (light, status_level) in &light_highest_status_levels {
            let light_mode = status_level_to_mode_mapper.map(status_level);
            let light_mapping = self.lights.get_mut(light).expect("Can't find light");

            light_mapping.set_mode(light_mode);
            light_mapping.update_state(&self.direct_output, &self.light_mode_to_state_mapper);

            if light_mapping.is_animated() {
                self.animated_lights.push(*light);
            }
        }
    }

    /// Updates lights that have a state that is animated, e.g. flashing. This
    /// needs to be called frequently for proper animation.
    //
    // Ideally the device would manage its own threading for animation but
    // this would require state updates to be communicated asynchronously.
    pub fn update_animated_lights(&self) {
        for light in &self.animated_lights {
            let light_mapping = self.lights.get(light).expect("Can't find light");
            light_mapping.update_state(&self.direct_output, &self.light_mode_to_state_mapper);
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
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

/// Available modes for boolean lights on the device.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BooleanLightMode {
    Off,
    On,
    #[serde(rename = "flash")]
    Flashing,
}

impl BooleanLightMode {
    /// Returns true if the mode requires animation, i.e. changes over time.
    fn is_animated(&self) -> bool {
        match self {
            Self::Flashing => true,
            _ => false,
        }
    }
}

/// Available modes for red/amber/green lights on the device.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedAmberGreenLightMode {
    Off,
    Red,
    Amber,
    Green,
    #[serde(rename = "red-amber")]
    FlashingRedAmber,
}

impl RedAmberGreenLightMode {
    /// Returns true if the mode requires animation, i.e. changes over time.
    fn is_animated(&self) -> bool {
        match self {
            RedAmberGreenLightMode::FlashingRedAmber => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LightMode {
    boolean: BooleanLightMode,
    red_amber_green: RedAmberGreenLightMode,
}

impl LightMode {
    pub fn new(boolean: BooleanLightMode, red_amber_green: RedAmberGreenLightMode) -> Self {
        Self {
            boolean,
            red_amber_green,
        }
    }
}

/// Common methods for interacting with light mapped to one or more device LEDs.
trait LightMapping {
    /// Returns true if the light's currently set mode is animated.
    fn is_animated(&self) -> bool;

    /// Updates the light's mode.
    fn set_mode(&mut self, light_mode: LightMode);

    /// Updates the mapped LEDs using the given `DirectOutput` object and based
    /// on the current mode and the given `LightModeToStateMapper`.
    fn update_state(
        &self,
        direct_output: &DirectOutput,
        light_mode_to_state_mapper: &LightModeToStateMapper,
    );
}

/// The mapping of a light to a single device LED.
struct BinaryLightMapping {
    led_id: u32,
    light_mode: BooleanLightMode,
}

impl BinaryLightMapping {
    fn new(led_id: u32) -> Self {
        Self {
            led_id,
            light_mode: BooleanLightMode::Off,
        }
    }
}

impl LightMapping for BinaryLightMapping {
    fn is_animated(&self) -> bool {
        self.light_mode.is_animated()
    }

    fn set_mode(&mut self, light_mode: LightMode) {
        self.light_mode = light_mode.boolean;
    }

    fn update_state(
        &self,
        direct_output: &DirectOutput,
        light_mode_to_state_mapper: &LightModeToStateMapper,
    ) {
        light_mode_to_state_mapper.update_binary_light(
            direct_output,
            &self.light_mode,
            self.led_id,
        );
    }
}

/// The mapping of a light to a red-green pair of device LEDs.
struct RedGreenLightMapping {
    red_led_id: u32,
    green_led_id: u32,
    light_mode: RedAmberGreenLightMode,
}

impl RedGreenLightMapping {
    fn new(red_led_id: u32, green_led_id: u32) -> Self {
        Self {
            red_led_id,
            green_led_id,
            light_mode: RedAmberGreenLightMode::Off,
        }
    }
}

impl LightMapping for RedGreenLightMapping {
    fn is_animated(&self) -> bool {
        self.light_mode.is_animated()
    }

    fn set_mode(&mut self, light_mode: LightMode) {
        self.light_mode = light_mode.red_amber_green;
    }

    fn update_state(
        &self,
        direct_output: &DirectOutput,
        light_mode_to_state_mapper: &LightModeToStateMapper,
    ) {
        light_mode_to_state_mapper.update_red_amber_green_light(
            direct_output,
            &self.light_mode,
            self.red_led_id,
            self.green_led_id,
        );
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
}
