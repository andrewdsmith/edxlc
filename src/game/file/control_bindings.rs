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
    #[serde(rename = "DeployHardpointToggle")]
    pub hardpoints: ControlBinding,
    #[serde(rename = "UseBoostJuice")]
    pub boost: ControlBinding,
    #[serde(rename = "HyperSuperCombination")]
    pub hyper_super_combo: ControlBinding,
    #[serde(rename = "Supercruise")]
    pub supercruise: ControlBinding,
    #[serde(rename = "Hyperspace")]
    pub hyperspace: ControlBinding,
    #[serde(rename = "ToggleButtonUpInput")]
    pub silent_running: ControlBinding,
    #[serde(rename = "DeployHeatSink")]
    pub heat_sink: ControlBinding,
    #[serde(rename = "ThrottleAxis")]
    pub throttle: ControlBinding,
    #[serde(rename = "NightVisionToggle")]
    pub night_vision: ControlBinding,
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
#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct ControlBinding {
    #[serde(rename = "Primary")]
    pub primary: Input,
    #[serde(rename = "Secondary")]
    pub secondary: Input,
    #[serde(rename = "Binding")]
    pub binding: Input,
}

impl ControlBinding {
    #[cfg(test)]
    pub fn new(primary: (&str, &str), secondary: (&str, &str)) -> Self {
        Self {
            primary: Input::new(primary.0, primary.1),
            secondary: Input::new(secondary.0, secondary.1),
            ..Default::default()
        }
    }
}

/// A device input as stored in the game binding files.
#[derive(Default, Deserialize, Debug, PartialEq)]
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
                <UseBoostJuice>
                    <Primary Device="D19" Key="K19" />
                    <Secondary Device="D20" Key="K20" />
                </UseBoostJuice>
                <HyperSuperCombination>
                    <Primary Device="D7" Key="K7" />
                    <Secondary Device="D8" Key="K8" />
                </HyperSuperCombination>
                <Supercruise>
                    <Primary Device="D9" Key="K9" />
                    <Secondary Device="D10" Key="K10" />
                </Supercruise>
                <Hyperspace>
                    <Primary Device="D11" Key="K11" />
                    <Secondary Device="D12" Key="K12" />
                </Hyperspace>
                <DeployHardpointToggle>
                    <Primary Device="D17" Key="K17" />
                    <Secondary Device="D18" Key="K18" />
                </DeployHardpointToggle>
                <ToggleButtonUpInput>
                    <Primary Device="D13" Key="K13" />
                    <Secondary Device="D14" Key="K14" />
                </ToggleButtonUpInput>
                <DeployHeatSink>
                    <Primary Device="D15" Key="K15" />
                    <Secondary Device="D16" Key="K16" />
                </DeployHeatSink>
                <ThrottleAxis>
                    <Binding Device="D21" Key="K21" />
                </ThrottleAxis>
                <NightVisionToggle>
                    <Primary Device="D22" Key="K22" />
                    <Secondary Device="D23" Key="K23" />
                </NightVisionToggle>
            </Root>
            "#,
        );

        let expected = ControlBindings {
            external_lights: ControlBinding::new(("D1", "K1"), ("D2", "K2")),
            cargo_scoop: ControlBinding::new(("D3", "K3"), ("D4", "K4")),
            landing_gear: ControlBinding::new(("D5", "K5"), ("D6", "K6")),
            hyper_super_combo: ControlBinding::new(("D7", "K7"), ("D8", "K8")),
            supercruise: ControlBinding::new(("D9", "K9"), ("D10", "K10")),
            hyperspace: ControlBinding::new(("D11", "K11"), ("D12", "K12")),
            silent_running: ControlBinding::new(("D13", "K13"), ("D14", "K14")),
            heat_sink: ControlBinding::new(("D15", "K15"), ("D16", "K16")),
            hardpoints: ControlBinding::new(("D17", "K17"), ("D18", "K18")),
            boost: ControlBinding::new(("D19", "K19"), ("D20", "K20")),
            throttle: ControlBinding {
                binding: Input {
                    device: String::from("D21"),
                    name: String::from("K21"),
                },
                ..Default::default()
            },
            night_vision: ControlBinding::new(("D22", "K22"), ("D23", "K23")),
        };

        assert_eq!(ControlBindings::from_str(xml), expected);
    }
}
