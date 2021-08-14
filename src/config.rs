use crate::x52pro::{
    device::{BooleanLightMode, LightMode, RedAmberGreenLightMode},
    StatusLevelToModeMapper,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIG_FILENAME: &str = "edxlc.toml";

const CONFIG_BOOLEAN_OFF: &str = "off";
const CONFIG_BOOLEAN_ON: &str = "on";
const CONFIG_BOOLEAN_FLASH: &str = "flash";

const CONFIG_OFF: &str = "off";
const CONFIG_RED: &str = "red";
const CONFIG_AMBER: &str = "amber";
const CONFIG_GREEN: &str = "green";
const CONFIG_RED_AMBER: &str = "red-amber";

/// Raw configuration straing values (as read from a configuraiton file) for a specific game mode.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ModeConfig {
    inactive: (String, String),
    active: (String, String),
    blocked: (String, String),
    alert: (String, String),
}

/// Modal configurations as read from a configuration file.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    default: ModeConfig,
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
        toml::from_str(&toml).expect("Could not load configuration")
    }

    /// Returns a `StatusLevelToModeMapper` configured from the mapped raw
    /// string values held by the instance.
    pub fn status_level_to_mode_mapper(&self) -> StatusLevelToModeMapper {
        StatusLevelToModeMapper::new(
            light_mode_from_config_values(&self.default.inactive),
            light_mode_from_config_values(&self.default.active),
            light_mode_from_config_values(&self.default.blocked),
            light_mode_from_config_values(&self.default.alert),
        )
    }
}

/// Returns the `LightMode` value corresponding to the referenced strings.
/// Panics if either of the string do not map to corresponding enum values.
fn light_mode_from_config_values(value: &(String, String)) -> LightMode {
    let (boolean, red_amber_green) = value;
    let boolean = match boolean.as_str() {
        CONFIG_BOOLEAN_OFF => BooleanLightMode::Off,
        CONFIG_BOOLEAN_ON => BooleanLightMode::On,
        CONFIG_BOOLEAN_FLASH => BooleanLightMode::Flashing,
        _ => panic!("Unsupported boolean configuration value '{}'", boolean),
    };
    let red_amber_green = match red_amber_green.as_str() {
        CONFIG_OFF => RedAmberGreenLightMode::Off,
        CONFIG_RED => RedAmberGreenLightMode::Red,
        CONFIG_AMBER => RedAmberGreenLightMode::Amber,
        CONFIG_GREEN => RedAmberGreenLightMode::Green,
        CONFIG_RED_AMBER => RedAmberGreenLightMode::FlashingRedAmber,
        _ => panic!(
            "Unsupported red/amber/green configuration value '{}'",
            red_amber_green
        ),
    };

    LightMode::new(boolean, red_amber_green)
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
        default: ModeConfig {
            inactive: (CONFIG_BOOLEAN_OFF.to_string(), CONFIG_GREEN.to_string()),
            active: (CONFIG_BOOLEAN_ON.to_string(), CONFIG_AMBER.to_string()),
            blocked: (CONFIG_BOOLEAN_OFF.to_string(), CONFIG_RED.to_string()),
            alert: (
                CONFIG_BOOLEAN_FLASH.to_string(),
                CONFIG_RED_AMBER.to_string(),
            ),
        },
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
            [default]
            inactive = ["{}", "{}"]
            active = ["{}", "{}"]
            blocked = ["{}", "{}"]
            alert = ["{}", "{}"]
        "#,
            CONFIG_BOOLEAN_OFF,
            CONFIG_GREEN,
            CONFIG_BOOLEAN_ON,
            CONFIG_AMBER,
            CONFIG_BOOLEAN_ON,
            CONFIG_RED,
            CONFIG_BOOLEAN_FLASH,
            CONFIG_RED_AMBER
        );

        let expected = Config {
            default: ModeConfig {
                inactive: (CONFIG_BOOLEAN_OFF.to_string(), CONFIG_GREEN.to_string()),
                active: (CONFIG_BOOLEAN_ON.to_string(), CONFIG_AMBER.to_string()),
                blocked: (CONFIG_BOOLEAN_ON.to_string(), CONFIG_RED.to_string()),
                alert: (
                    CONFIG_BOOLEAN_FLASH.to_string(),
                    CONFIG_RED_AMBER.to_string(),
                ),
            },
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }

    fn assert_boolean_light_mode(input: &str, boolean: BooleanLightMode) {
        let expected = LightMode::new(boolean, RedAmberGreenLightMode::Off);
        assert_eq!(
            light_mode_from_config_values(&(String::from(input), CONFIG_OFF.to_string())),
            expected
        );
    }

    #[test]
    fn light_mode_from_config_values_boolean_values() {
        assert_boolean_light_mode(CONFIG_BOOLEAN_OFF, BooleanLightMode::Off);
        assert_boolean_light_mode(CONFIG_BOOLEAN_ON, BooleanLightMode::On);
        assert_boolean_light_mode(CONFIG_BOOLEAN_FLASH, BooleanLightMode::Flashing);
    }

    #[test]
    #[should_panic]
    fn light_mode_from_config_values_boolean_unsupported_value() {
        assert_boolean_light_mode("inverted", BooleanLightMode::Off);
    }

    fn assert_red_amber_green_light_mode(input: &str, red_amber_green: RedAmberGreenLightMode) {
        let expected = LightMode::new(BooleanLightMode::Off, red_amber_green);
        assert_eq!(
            light_mode_from_config_values(&(CONFIG_BOOLEAN_OFF.to_string(), String::from(input))),
            expected
        );
    }

    #[test]
    fn light_mode_from_config_values_rag_values() {
        assert_red_amber_green_light_mode(CONFIG_OFF, RedAmberGreenLightMode::Off);
        assert_red_amber_green_light_mode(CONFIG_RED, RedAmberGreenLightMode::Red);
        assert_red_amber_green_light_mode(CONFIG_AMBER, RedAmberGreenLightMode::Amber);
        assert_red_amber_green_light_mode(CONFIG_GREEN, RedAmberGreenLightMode::Green);
        assert_red_amber_green_light_mode(
            CONFIG_RED_AMBER,
            RedAmberGreenLightMode::FlashingRedAmber,
        );
    }

    #[test]
    #[should_panic]
    fn light_mode_from_config_values_rag_unsupported_values() {
        assert_red_amber_green_light_mode("blue", RedAmberGreenLightMode::Off);
    }
}
