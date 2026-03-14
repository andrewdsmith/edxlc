use crate::game::GlobalStatus;
use crate::x52pro::{
    StatusLevelToModeMapper,
    device::{BooleanLightMode, LightMode, RedAmberGreenLightMode},
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

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
    files: Option<Files>,
    default: ModeConfig,
    hardpoints_deployed: Option<ModeConfig>,
    night_vision: Option<ModeConfig>,
    silent_running: Option<ModeConfig>,
    mfd: Option<MfdConfig>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Default)]
pub struct MfdConfig {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub line3: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Files {
    bindings: Option<String>,
}

const DEFAULT_BINDINGS_FILE_PATH: &str =
    r"Frontier Developments\Elite Dangerous\Options\Bindings\Custom.4.2.binds";

impl Config {
    /// Returns a new instance constructed by loading the give configuration
    /// file. Panics if the TOML cannot be parsed.
    pub fn from_file(config_filename: String) -> Self {
        let toml = fs::read_to_string(config_filename).expect("Could not read configuration file");
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
            GlobalStatus::HardpointsDeployed => {
                self.mode_config_or_default(&self.hardpoints_deployed)
            }
            GlobalStatus::NightVisionOn => self.mode_config_or_default(&self.night_vision),
            GlobalStatus::SilentRunning => self.mode_config_or_default(&self.silent_running),
        };

        StatusLevelToModeMapper::new(
            light_mode_from_config_values(mode_config.inactive),
            light_mode_from_config_values(mode_config.active),
            light_mode_from_config_values(mode_config.blocked),
            light_mode_from_config_values(mode_config.alert),
        )
    }

    fn mode_config_or_default<'a>(
        &'a self,
        optional_config: &'a Option<ModeConfig>,
    ) -> &'a ModeConfig {
        match optional_config {
            Some(mode_config) => mode_config,
            None => &self.default,
        }
    }

    /// Returns the configured path for the bindings file or the default if not
    /// configured.
    pub fn bindings_file_path(&self) -> PathBuf {
        if let Some(files) = &self.files {
            if let Some(bindings) = &files.bindings {
                return PathBuf::from(bindings);
            }
        }

        dirs::data_local_dir()
            .expect("Can't find user app data directory")
            .join(DEFAULT_BINDINGS_FILE_PATH)
    }

    /// Returns the MFD configuration if provided.
    pub fn mfd(&self) -> Option<&MfdConfig> {
        self.mfd.as_ref()
    }

    /// Returns the MFD lines as a vector of strings.
    pub fn mfd_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(mfd) = &self.mfd {
            if let Some(line1) = &mfd.line1 {
                lines.push(line1.clone());
            } else {
                lines.push(String::new());
            }
            if let Some(line2) = &mfd.line2 {
                lines.push(line2.clone());
            } else {
                lines.push(String::new());
            }
            if let Some(line3) = &mfd.line3 {
                lines.push(line3.clone());
            } else {
                lines.push(String::new());
            }
        } else {
            lines.push(String::from("EDXLC"));
            lines.push(String::new());
            lines.push(String::new());
        }
        lines
    }
}

/// Returns the `LightMode` value corresponding to the mode tuple.
fn light_mode_from_config_values(value: (BooleanLightMode, RedAmberGreenLightMode)) -> LightMode {
    let (boolean, red_amber_green) = value;
    LightMode::new(boolean, red_amber_green)
}

/// Writes a default configuration file to the given filename if that file does
/// not exist. Panics if the file cannot be written, e.g. if the user does not
/// have permission.
pub fn write_default_file_if_missing(config_filename: &str) {
    if Path::new(config_filename).exists() {
        return;
    }

    info!("Writing default configuration file");

    let config = Config {
        files: None,
        default: ModeConfig {
            inactive: (BooleanLightMode::On, RedAmberGreenLightMode::Green),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::AmberFlash),
        },
        hardpoints_deployed: Some(ModeConfig {
            inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Off),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::AmberFlash),
        }),
        night_vision: Some(ModeConfig {
            inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Off),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Green),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Off),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::GreenFlash),
        }),
        silent_running: Some(ModeConfig {
            inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Off),
            active: (BooleanLightMode::On, RedAmberGreenLightMode::Green),
            blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
            alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::GreenFlash),
        }),
        mfd: Some(MfdConfig {
            line1: Some(String::from("EDXLC")),
            line2: None,
            line3: None,
        }),
    };

    let toml = toml::to_string(&config).expect("Could not serialize default configuration");
    fs::write(config_filename, toml).expect("Could not write default configuration file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_toml_returns_an_instance() {
        let toml = r#"
            [files]
            bindings = 'C:\Path\To.binds'
            [default]
            inactive = ["off", "green"]
            active = ["on", "amber"]
            blocked = ["on", "red"]
            alert = ["flash", "red-amber"]
            [hardpoints-deployed]
            inactive = ["on", "green"]
            active = ["off", "amber"]
            blocked = ["flash", "red"]
            alert = ["off", "red-amber"]
            [night-vision]
            inactive = ["flash", "green"]
            active = ["flash", "amber"]
            blocked = ["off", "red"]
            alert = ["on", "red-amber"]
            [silent-running]
            inactive = ["off", "amber"]
            active = ["on", "amber"]
            blocked = ["off", "red"]
            alert = ["flash", "red-amber"]"#;

        let expected = Config {
            files: Some(Files {
                bindings: Some(String::from(r"C:\Path\To.binds")),
            }),
            default: ModeConfig {
                inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::On, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
            },
            hardpoints_deployed: Some(ModeConfig {
                inactive: (BooleanLightMode::On, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::Off, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::Flash, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Off, RedAmberGreenLightMode::RedAmber),
            }),
            night_vision: Some(ModeConfig {
                inactive: (BooleanLightMode::Flash, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::Flash, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::On, RedAmberGreenLightMode::RedAmber),
            }),
            silent_running: Some(ModeConfig {
                inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Amber),
                active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::Off, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
            }),
            mfd: None,
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }

    #[test]
    fn config_from_toml_returns_an_instance_with_none_for_missing_blocks() {
        let toml = r#"
            [default]
            inactive = ["off", "green"]
            active = ["on", "amber"]
            blocked = ["on", "red"]
            alert = ["flash", "red-amber"]"#;

        let expected = Config {
            files: None,
            default: ModeConfig {
                inactive: (BooleanLightMode::Off, RedAmberGreenLightMode::Green),
                active: (BooleanLightMode::On, RedAmberGreenLightMode::Amber),
                blocked: (BooleanLightMode::On, RedAmberGreenLightMode::Red),
                alert: (BooleanLightMode::Flash, RedAmberGreenLightMode::RedAmber),
            },
            hardpoints_deployed: None,
            night_vision: None,
            silent_running: None,
            mfd: None,
        };

        assert_eq!(Config::from_toml(&String::from(toml)), expected);
    }

    #[test]
    fn config_status_level_to_mode_mapper_returns_configured_mapped() {
        let default_light_config = (BooleanLightMode::On, RedAmberGreenLightMode::Green);
        let other_light_config = (BooleanLightMode::Off, RedAmberGreenLightMode::Red);

        let default_light_mode = LightMode {
            boolean: default_light_config.0,
            red_amber_green: default_light_config.1,
        };

        let config = Config {
            files: None,
            default: ModeConfig {
                inactive: default_light_config,
                active: default_light_config,
                blocked: default_light_config,
                alert: default_light_config,
            },
            hardpoints_deployed: Some(ModeConfig {
                inactive: other_light_config,
                active: other_light_config,
                blocked: other_light_config,
                alert: other_light_config,
            }),
            night_vision: None,
            silent_running: None,
            mfd: None,
        };

        let actual_mapper = config.status_level_to_mode_mapper(GlobalStatus::Normal);
        let expected_mapper = StatusLevelToModeMapper {
            inactive: default_light_mode,
            active: default_light_mode,
            blocked: default_light_mode,
            alert: default_light_mode,
        };

        assert_eq!(actual_mapper, expected_mapper);
    }

    #[test]
    fn config_status_level_to_mode_mapper_returns_night_vision_mapper() {
        let default_light_config = (BooleanLightMode::On, RedAmberGreenLightMode::Green);
        let night_vision_light_config = (BooleanLightMode::Off, RedAmberGreenLightMode::Off);

        let config = Config {
            files: None,
            default: ModeConfig {
                inactive: default_light_config,
                active: default_light_config,
                blocked: default_light_config,
                alert: default_light_config,
            },
            hardpoints_deployed: None,
            night_vision: Some(ModeConfig {
                inactive: night_vision_light_config,
                active: night_vision_light_config,
                blocked: night_vision_light_config,
                alert: night_vision_light_config,
            }),
            silent_running: None,
            mfd: None,
        };

        let actual_mapper = config.status_level_to_mode_mapper(GlobalStatus::NightVisionOn);

        let expected_light_mode = LightMode {
            boolean: night_vision_light_config.0,
            red_amber_green: night_vision_light_config.1,
        };
        let expected_mapper = StatusLevelToModeMapper {
            inactive: expected_light_mode,
            active: expected_light_mode,
            blocked: expected_light_mode,
            alert: expected_light_mode,
        };

        assert_eq!(actual_mapper, expected_mapper);
    }

    #[test]
    fn config_status_level_to_mode_mapper_returns_defaults() {
        let default_light_config = (BooleanLightMode::On, RedAmberGreenLightMode::Green);

        let default_light_mode = LightMode {
            boolean: default_light_config.0,
            red_amber_green: default_light_config.1,
        };

        // Could remove the `None` values by implementing `Default` on the
        // struct.
        let config_without_hardpoints_deployed = Config {
            files: None,
            default: ModeConfig {
                inactive: default_light_config,
                active: default_light_config,
                blocked: default_light_config,
                alert: default_light_config,
            },
            hardpoints_deployed: None,
            night_vision: None,
            silent_running: None,
            mfd: None,
        };

        let expected_mapper = StatusLevelToModeMapper {
            inactive: default_light_mode,
            active: default_light_mode,
            blocked: default_light_mode,
            alert: default_light_mode,
        };

        let global_statuses = vec![
            GlobalStatus::Normal,
            GlobalStatus::HardpointsDeployed,
            GlobalStatus::NightVisionOn,
        ];

        for global_status in global_statuses {
            let actual_mapper =
                config_without_hardpoints_deployed.status_level_to_mode_mapper(global_status);

            assert_eq!(actual_mapper, expected_mapper);
        }
    }
}
