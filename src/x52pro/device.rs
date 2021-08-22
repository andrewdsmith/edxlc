use crate::game::StatusLevel;
use crate::x52pro::{direct_output::DirectOutput, LightModeToStateMapper, StatusLevelToModeMapper};
use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Controllable LEDs on the device. Assigned values correspond to the ids used
/// by DirectOutput.
#[derive(Copy, Clone)]
pub enum Led {
    Fire = 0,
    FireARed = 1,
    FireAGreen = 2,
    FireBRed = 3,
    FireBGreen = 4,
    FireDRed = 5,
    FireDGreen = 6,
    FireERed = 7,
    FireEGreen = 8,
    T1T2Red = 9,
    T1T2Green = 10,
    T3T4Red = 11,
    T3T4Green = 12,
    T5T6Red = 13,
    T5T6Green = 14,
    PoV2Red = 15,
    PoV2Green = 16,
    ClutchRed = 17,
    ClutchGreen = 18,
    Throttle = 19,
}

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
            Box::new(RedGreenLightMapping::new(Led::ClutchRed, Led::ClutchGreen)),
        );
        lights.insert(Light::Fire, Box::new(BinaryLightMapping::new(Led::Fire)));
        lights.insert(
            Light::FireA,
            Box::new(RedGreenLightMapping::new(Led::FireARed, Led::FireAGreen)),
        );
        lights.insert(
            Light::FireB,
            Box::new(RedGreenLightMapping::new(Led::FireBRed, Led::FireBGreen)),
        );
        lights.insert(
            Light::FireD,
            Box::new(RedGreenLightMapping::new(Led::FireDRed, Led::FireDGreen)),
        );
        lights.insert(
            Light::FireE,
            Box::new(RedGreenLightMapping::new(Led::FireERed, Led::FireEGreen)),
        );
        lights.insert(
            Light::PoV2,
            Box::new(RedGreenLightMapping::new(Led::PoV2Red, Led::PoV2Green)),
        );
        lights.insert(
            Light::T1T2,
            Box::new(RedGreenLightMapping::new(Led::T1T2Red, Led::T1T2Green)),
        );
        lights.insert(
            Light::T3T4,
            Box::new(RedGreenLightMapping::new(Led::T3T4Red, Led::T3T4Green)),
        );
        lights.insert(
            Light::T5T6,
            Box::new(RedGreenLightMapping::new(Led::T5T6Red, Led::T5T6Green)),
        );
        lights.insert(
            Light::Throttle,
            Box::new(BinaryLightMapping::new(Led::Throttle)),
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
        // A hash mapping every light to the highest status level encountered.
        // This creation could be moved into `new`.
        let mut light_highest_status_levels = HashMap::new();

        // Default all lights to inactive.
        for light in Light::into_enum_iter() {
            light_highest_status_levels.insert(light, StatusLevel::Inactive);
        }

        // Map given inputs to corresponding light and update the hash if the
        // status level is higher. It should be entirely safe to call `unwrap`
        // as we know we have an entry in the hash for every light.
        for (input, status_level) in input_status_levels {
            let light = light_for_input(input);
            let light_status_level = light_highest_status_levels.get_mut(&light).unwrap();

            if status_level > *light_status_level {
                *light_status_level = status_level.clone();
            }
        }

        // Update the list of lights that are in a mode that requires animation.
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
    PoV2Down,
    PoV2Left,
    PoV2Right,
    PoV2Up,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    ZAxis,
}

/// Controllable lights on the device, which have either one or two LEDs.
#[derive(Copy, Clone, Debug, Eq, Hash, IntoEnumIterator, PartialEq)]
enum Light {
    Clutch,
    Fire,
    FireA,
    FireB,
    FireD,
    FireE,
    PoV2,
    T1T2,
    T3T4,
    T5T6,
    Throttle,
}

/// Available modes for boolean lights on the device.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BooleanLightMode {
    Off,
    On,
    Flash,
}

impl BooleanLightMode {
    /// Returns true if the mode requires animation, i.e. changes over time.
    fn is_animated(&self) -> bool {
        match self {
            Self::Flash => true,
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
    RedAmber,
}

impl RedAmberGreenLightMode {
    /// Returns true if the mode requires animation, i.e. changes over time.
    fn is_animated(&self) -> bool {
        match self {
            RedAmberGreenLightMode::RedAmber => true,
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
    led_id: Led,
    light_mode: BooleanLightMode,
}

impl BinaryLightMapping {
    fn new(led_id: Led) -> Self {
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
    red_led_id: Led,
    green_led_id: Led,
    light_mode: RedAmberGreenLightMode,
}

impl RedGreenLightMapping {
    fn new(red_led_id: Led, green_led_id: Led) -> Self {
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
        Input::PoV2Down => Light::PoV2,
        Input::PoV2Left => Light::PoV2,
        Input::PoV2Right => Light::PoV2,
        Input::PoV2Up => Light::PoV2,
        Input::T1 => Light::T1T2,
        Input::T2 => Light::T1T2,
        Input::T3 => Light::T3T4,
        Input::T4 => Light::T3T4,
        Input::T5 => Light::T5T6,
        Input::T6 => Light::T5T6,
        Input::ZAxis => Light::Throttle,
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
        assert_light_for_input(Input::PoV2Up, Light::PoV2);
        assert_light_for_input(Input::PoV2Down, Light::PoV2);
        assert_light_for_input(Input::PoV2Left, Light::PoV2);
        assert_light_for_input(Input::PoV2Right, Light::PoV2);
        assert_light_for_input(Input::T1, Light::T1T2);
        assert_light_for_input(Input::T2, Light::T1T2);
        assert_light_for_input(Input::T3, Light::T3T4);
        assert_light_for_input(Input::T4, Light::T3T4);
        assert_light_for_input(Input::T5, Light::T5T6);
        assert_light_for_input(Input::T6, Light::T5T6);
        assert_light_for_input(Input::ZAxis, Light::Throttle);
    }

    fn assert_light_for_input(input: Input, light: Light) {
        assert_eq!(light_for_input(input), light);
    }
}
