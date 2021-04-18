use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// A set of mappings from device inputs to game controls as stored in the game
/// binding files.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename = "Root")]
pub struct ControlBindings {
    #[serde(rename = "ShipSpotLightToggle")]
    pub external_lights: ControlBinding,
    #[serde(rename = "ToggleCargoScoop")]
    pub cargo_scoop: ControlBinding,
    #[serde(rename = "LandingGearToggle")]
    pub landing_gear: ControlBinding,
}

impl ControlBindings {
    pub fn from_file(path: &PathBuf) -> Self {
        let xml = fs::read_to_string(path).expect("Could not read bindings file");
        Self::from_str(xml)
    }

    pub fn from_str(xml: String) -> Self {
        serde_xml_rs::from_str(&xml).expect("Could not parse bindings XML")
    }
}

/// A pair of device inputs that can be mapped to a game control, as stored in
/// the game binding files.
#[derive(Deserialize, Debug, PartialEq)]
pub struct ControlBinding {
    #[serde(rename = "Primary")]
    pub primary: Input,
    #[serde(rename = "Secondary")]
    pub secondary: Input,
}

impl ControlBinding {
    #[cfg(test)]
    pub fn new(primary: (&str, &str), secondary: (&str, &str)) -> Self {
        Self {
            primary: Input::new(primary.0, primary.1),
            secondary: Input::new(secondary.0, secondary.1),
        }
    }
}

/// A device input as stored in the game binding files.
#[derive(Deserialize, Debug, PartialEq)]
pub struct Input {
    #[serde(rename = "Device")]
    pub device: String,
    #[serde(rename = "Key")]
    pub name: String,
}

impl Input {
    #[cfg(test)]
    pub fn new(device: &str, name: &str) -> Self {
        Self {
            device: String::from(device),
            name: String::from(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_bindings_from_xml() {
        let xml = String::from(
            r#"
            <Root>
                <ShipSpotLightToggle>
                    <Primary Device="D1" Key="K1" />
                    <Secondary Device="D2" Key="K2" />
                </ShipSpotLightToggle>
                <ToggleCargoScoop>
                    <Primary Device="D3" Key="K3" />
                    <Secondary Device="D4" Key="K4" />
                </ToggleCargoScoop>
                <LandingGearToggle>
                    <Primary Device="D5" Key="K5" />
                    <Secondary Device="D6" Key="K6" />
                </LandingGearToggle>
            </Root>
            "#,
        );

        let expected = ControlBindings {
            external_lights: ControlBinding::new(("D1", "K1"), ("D2", "K2")),
            cargo_scoop: ControlBinding::new(("D3", "K3"), ("D4", "K4")),
            landing_gear: ControlBinding::new(("D5", "K5"), ("D6", "K6")),
        };

        assert_eq!(ControlBindings::from_str(xml), expected);
    }
}
