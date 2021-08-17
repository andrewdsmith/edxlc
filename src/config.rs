use crate::game::GlobalStatus;
use crate::x52pro::{
    device::{BooleanLightMode, LightMode, RedAmberGreenLightMode},
    StatusLevelToModeMapper,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIG_FILENAME: &str = "edxlc.toml";

/// Raw configuration string values (as read from a configuraiton file) for a specific game mode.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ModeConfig {
    inactive: (BooleanLightMode, RedAmberGreenLightMode),
    active: (BooleanLightMode, RedAmberGreenLightMode),
    blocked: (BooleanLightMode, RedAmberGreenLightMode),
    alert: (BooleanLightMode, RedAmberGreenLightMode),
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
            light_mode_from_config_values(mode_config.inactive),
            light_mode_from_config_values(mode_config.active),
            light_mode_from_config_values(mode_config.blocked),
            light_mode_from_config_values(mode_config.alert),
        )
    }
}

/// Returns the `LightMode` value corresponding to the mode tuple.
fn light_mode_from_config_values(value: (BooleanLightMode, RedAmberGreenLightMode)) -> LightMode {
    let (boolean, red_amber_green) = value;
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
            inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Green),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
        },
        hardpoints_deployed: ModeConfig {
            inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Off),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
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
        let toml = r#"
            [default]
            inactive = ["off", "green"]
            active = ["on", "amber"]
            blocked = ["on", "red"]
            alert = ["flash", "red-amber"]
            [hardpoints-deployed]
            inactive = ["on", "green"]
            active = ["off", "amber"]
            blocked = ["flash", "red"]
            alert = ["off", "red-amber"]"#;

        let expected = Config {
            default: ModeConfig {
                inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::On, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
            },
            hardpoints_deployed: ModeConfig {
                inactive: (BooleanLightMode::On, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::Off, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::Flash, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Off, RedAmberGreenLightMode::RedAmber),
            },
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }
}
