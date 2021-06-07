use crate::x52pro::{device::RedAmberGreenLightMode, StatusLevelToModeMapper};
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIG_FILENAME: &str = "edxlc.toml";

const CONFIG_OFF: &str = "off";
const CONFIG_RED: &str = "red";
const CONFIG_AMBER: &str = "amber";
const CONFIG_GREEN: &str = "green";
const CONFIG_RED_AMBER: &str = "red-amber";

/// Raw configuration string values as read from a configuration file.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    inactive: String,
    active: String,
    blocked: String,
    alert: String,
}

impl Config {
    /// Returns a new instance constructed by loading the configuration file
    /// present in the current working directory. Panics if the TOML cannot be
    /// parsed.
    pub fn from_file() -> Self {
        let toml = fs::read_to_string(CONFIG_FILENAME).expect("Could not read configuration file");
        Self::from_toml(&toml)
    }

    /// Returns a new instance constructed from the referenced TOML `String`.
    /// Panics if the TOML cannot be parsed.
    fn from_toml(toml: &String) -> Self {
        toml::from_str(&toml).expect("Could not serialize default configuration")
    }

    /// Returns a `StatusLevelToModeMapper` configured from the mapped raw
    /// string values held by the instance.
    pub fn status_level_to_mode_mapper(&self) -> StatusLevelToModeMapper {
        StatusLevelToModeMapper::new(
            led_state_from_config(&self.inactive),
            led_state_from_config(&self.active),
            led_state_from_config(&self.blocked),
            led_state_from_config(&self.alert),
        )
    }
}

/// Returns the `RedAmberGreenLightMode` value corresponding to the referenced string value.
/// Panics is the string does not map to an `RedAmberGreenLightMode` value.
fn led_state_from_config(value: &String) -> RedAmberGreenLightMode {
    match value.as_str() {
        CONFIG_OFF => RedAmberGreenLightMode::Off,
        CONFIG_RED => RedAmberGreenLightMode::Red,
        CONFIG_AMBER => RedAmberGreenLightMode::Amber,
        CONFIG_GREEN => RedAmberGreenLightMode::Green,
        CONFIG_RED_AMBER => RedAmberGreenLightMode::FlashingRedAmber,
        _ => panic!("Unsupported configuration value '{}'", value),
    }
}

/// Writes a default configuration file in the current working directory if a
/// file with the expected name does not exist. Panics if the file cannot be
/// written, e.g. if the user does not have permission.
pub fn write_default_file_if_missing() {
    if Path::new(CONFIG_FILENAME).exists() {
        return;
    }

    info!("Writing default configuration file");

    let config = Config {
        inactive: CONFIG_GREEN.to_string(),
        active: CONFIG_AMBER.to_string(),
        blocked: CONFIG_RED.to_string(),
        alert: CONFIG_RED_AMBER.to_string(),
    };

    let toml = toml::to_string(&config).expect("Could not serialize default configuration");
    fs::write(CONFIG_FILENAME, toml).expect("Could not write default configuration file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_toml_returns_an_instance() {
        let toml = format!(
            r#"
            inactive = "{}"
            active = "{}"
            blocked = "{}"
            alert = "{}"
        "#,
            CONFIG_GREEN, CONFIG_AMBER, CONFIG_RED, CONFIG_RED_AMBER
        );

        let expected = Config {
            inactive: CONFIG_GREEN.to_string(),
            active: CONFIG_AMBER.to_string(),
            blocked: CONFIG_RED.to_string(),
            alert: CONFIG_RED_AMBER.to_string(),
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }

    fn assert_led_state_from_config(input: &str, expected: RedAmberGreenLightMode) {
        assert_eq!(led_state_from_config(&String::from(input)), expected);
    }

    #[test]
    fn led_state_from_string_maps_strings_to_values() {
        assert_led_state_from_config(CONFIG_OFF, RedAmberGreenLightMode::Off);
        assert_led_state_from_config(CONFIG_RED, RedAmberGreenLightMode::Red);
        assert_led_state_from_config(CONFIG_AMBER, RedAmberGreenLightMode::Amber);
        assert_led_state_from_config(CONFIG_GREEN, RedAmberGreenLightMode::Green);
        assert_led_state_from_config(CONFIG_RED_AMBER, RedAmberGreenLightMode::FlashingRedAmber);
    }

    #[test]
    #[should_panic]
    fn led_state_from_string_panics_on_unsupported_values() {
        led_state_from_config(&String::from("blue"));
    }
}
