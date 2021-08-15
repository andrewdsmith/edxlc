use crate::game::GlobalStatus;
use crate::x52pro::{
    device::{BooleanLightMode, LightMode, RedAmberGreenLightMode},
    StatusLevelToModeMapper,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIG_FILENAME: &str = "edxlc.toml";

const CONFIG_OFF: &str = "off";
const CONFIG_RED: &str = "red";
const CONFIG_AMBER: &str = "amber";
const CONFIG_GREEN: &str = "green";
const CONFIG_RED_AMBER: &str = "red-amber";

/// Raw configuration string values (as read from a configuraiton file) for a specific game mode.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ModeConfig {
    inactive: (BooleanLightMode, String),
    active: (BooleanLightMode, String),
    blocked: (BooleanLightMode, String),
    alert: (BooleanLightMode, String),
}

/// Modal configurations as read from a configuration file.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    default: ModeConfig,
    hardpoints_deployed: ModeConfig,
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

    /// Returns a `StatusLevelToModeMapper` for the given `GlobalStatus` value,
    /// as configured from the mapped raw string values held by the instance.
    pub fn status_level_to_mode_mapper(
        &self,
        global_status: GlobalStatus,
    ) -> StatusLevelToModeMapper {
        let mode_config = match global_status {
            GlobalStatus::Normal => &self.default,
            GlobalStatus::HardpointsDeployed => &self.hardpoints_deployed,
        };

        StatusLevelToModeMapper::new(
            light_mode_from_config_values(&mode_config.inactive),
            light_mode_from_config_values(&mode_config.active),
            light_mode_from_config_values(&mode_config.blocked),
            light_mode_from_config_values(&mode_config.alert),
        )
    }
}

/// Returns the `LightMode` value corresponding to the referenced strings.
/// Panics if either of the string do not map to corresponding enum values.
fn light_mode_from_config_values(value: &(BooleanLightMode, String)) -> LightMode {
    let (boolean, red_amber_green) = value;
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

    LightMode::new(*boolean, red_amber_green)
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
            inactive: (BooleanLightMode::Off, CONFIG_GREEN.to_string()),
            active: (BooleanLightMode::On, CONFIG_AMBER.to_string()),
            blocked: (BooleanLightMode::Off, CONFIG_RED.to_string()),
            alert: (BooleanLightMode::Flashing, CONFIG_RED_AMBER.to_string()),
        },
        hardpoints_deployed: ModeConfig {
            inactive: (BooleanLightMode::Off, CONFIG_RED.to_string()),
            active: (BooleanLightMode::On, CONFIG_AMBER.to_string()),
            blocked: (BooleanLightMode::Off, CONFIG_OFF.to_string()),
            alert: (BooleanLightMode::Flashing, CONFIG_RED_AMBER.to_string()),
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
            inactive = ["off", "{}"]
            active = ["on", "{}"]
            blocked = ["on", "{}"]
            alert = ["flash", "{}"]
            [hardpoints-deployed]
            inactive = ["on", "{}"]
            active = ["off", "{}"]
            blocked = ["flash", "{}"]
            alert = ["off", "{}"]
            "#,
            CONFIG_GREEN,
            CONFIG_AMBER,
            CONFIG_RED,
            CONFIG_RED_AMBER,
            CONFIG_GREEN,
            CONFIG_AMBER,
            CONFIG_RED,
            CONFIG_RED_AMBER
        );

        let expected = Config {
            default: ModeConfig {
                inactive: (BooleanLightMode::Off, CONFIG_GREEN.to_string()),
                active: (BooleanLightMode::On, CONFIG_AMBER.to_string()),
                blocked: (BooleanLightMode::On, CONFIG_RED.to_string()),
                alert: (BooleanLightMode::Flashing, CONFIG_RED_AMBER.to_string()),
            },
            hardpoints_deployed: ModeConfig {
                inactive: (BooleanLightMode::On, CONFIG_GREEN.to_string()),
                active: (BooleanLightMode::Off, CONFIG_AMBER.to_string()),
                blocked: (BooleanLightMode::Flashing, CONFIG_RED.to_string()),
                alert: (BooleanLightMode::Off, CONFIG_RED_AMBER.to_string()),
            },
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }

    fn assert_red_amber_green_light_mode(input: &str, red_amber_green: RedAmberGreenLightMode) {
        let expected = LightMode::new(BooleanLightMode::Off, red_amber_green);
        assert_eq!(
            light_mode_from_config_values(&(BooleanLightMode::Off, String::from(input))),
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
