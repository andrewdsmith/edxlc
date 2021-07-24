use crate::game::file::{ControlBindings, Input as BindingsInput};
use crate::x52pro::device::Input;
use std::path::PathBuf;

const X52PRO_DEVICE: &str = "SaitekX52Pro";
const X52PRO_CLUTCH: &str = "Joy_31";
const X52PRO_FIRE: &str = "Joy_2";
const X52PRO_FIRE_A: &str = "Joy_3";
const X52PRO_FIRE_B: &str = "Joy_4";
const X52PRO_FIRE_D: &str = "Joy_7";
const X52PRO_FIRE_E: &str = "Joy_8";
const X52PRO_T1: &str = "Joy_9";
const X52PRO_T2: &str = "Joy_10";
const X52PRO_T3: &str = "Joy_11";
const X52PRO_T4: &str = "Joy_12";
const X52PRO_T5: &str = "Joy_13";
const X52PRO_T6: &str = "Joy_14";

/// A supported game control that can be mapped to an X52Pro input.
pub enum Control {
    Boost,
    CargoScoop,
    ExternalLights,
    Hardpoints,
    HeatSink,
    Hyperspace,
    HyperSuperCombination,
    LandingGear,
    SilentRunning,
    Supercruise,
}

/// The set of game controls bound to X52Pro inputs as loaded from a bindings
/// file.
#[derive(Debug)]
pub struct Controls {
    file: ControlBindings,
}

impl Controls {
    /// Returns an instance built by loaded the bindings file at the give path.
    pub fn from_file(path: &PathBuf) -> Self {
        Self::from_file_control_bindings(ControlBindings::from_file(path))
    }

    /// Returns an instance built from the given `ControlBindings` instance.
    pub fn from_file_control_bindings(file: ControlBindings) -> Self {
        Controls { file }
    }

    /// Returns a vector containing all the `Input` instances that are bound to
    /// the given `Control` instance. The vector will be empty if none of the
    /// supported inputs is bound to the given control.
    pub fn inputs_for_control(&self, control: Control) -> Vec<Input> {
        let control_binding = match control {
            Control::Boost => &self.file.boost,
            Control::CargoScoop => &self.file.cargo_scoop,
            Control::ExternalLights => &self.file.external_lights,
            Control::Hardpoints => &self.file.hardpoints,
            Control::HeatSink => &self.file.heat_sink,
            Control::Hyperspace => &self.file.hyperspace,
            Control::HyperSuperCombination => &self.file.hyper_super_combo,
            Control::LandingGear => &self.file.landing_gear,
            Control::SilentRunning => &self.file.silent_running,
            Control::Supercruise => &self.file.supercruise,
        };

        let mut inputs = Vec::with_capacity(2);

        // This could probably be more elegantly written by mapping the vector
        // elements through the function and collecting the non-None elements.
        for file_input in vec![&control_binding.primary, &control_binding.secondary] {
            if let Some(input) = input_from_file_input(file_input) {
                inputs.push(input);
            }
        }

        inputs
    }
}

/// Returns a supported X52Pro `Input` that matches the bindings file input.
fn input_from_file_input(input: &BindingsInput) -> Option<Input> {
    match input.device.as_str() {
        X52PRO_DEVICE => match input.name.as_str() {
            X52PRO_CLUTCH => Some(Input::Clutch),
            X52PRO_FIRE => Some(Input::Fire),
            X52PRO_FIRE_A => Some(Input::FireA),
            X52PRO_FIRE_B => Some(Input::FireB),
            X52PRO_FIRE_D => Some(Input::FireD),
            X52PRO_FIRE_E => Some(Input::FireE),
            X52PRO_T1 => Some(Input::T1),
            X52PRO_T2 => Some(Input::T2),
            X52PRO_T3 => Some(Input::T3),
            X52PRO_T4 => Some(Input::T4),
            X52PRO_T5 => Some(Input::T5),
            X52PRO_T6 => Some(Input::T6),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::file::ControlBinding;

    #[test]
    fn controls_inputs_for_control() {
        let file_control_bindings = ControlBindings {
            cargo_scoop: ControlBinding::new((X52PRO_DEVICE, X52PRO_T2), ("", "")),
            external_lights: ControlBinding::new(("", ""), (X52PRO_DEVICE, X52PRO_T4)),
            landing_gear: ControlBinding::new(
                (X52PRO_DEVICE, X52PRO_T2),
                (X52PRO_DEVICE, X52PRO_T4),
            ),
            hyper_super_combo: ControlBinding::new((X52PRO_DEVICE, X52PRO_T1), ("", "")),
            supercruise: ControlBinding::new((X52PRO_DEVICE, X52PRO_T3), ("", "")),
            hyperspace: ControlBinding::new((X52PRO_DEVICE, X52PRO_T5), ("", "")),
            silent_running: ControlBinding::new((X52PRO_DEVICE, X52PRO_FIRE_A), ("", "")),
            heat_sink: ControlBinding::new((X52PRO_DEVICE, X52PRO_T6), ("", "")),
            hardpoints: ControlBinding::new((X52PRO_DEVICE, X52PRO_FIRE_B), ("", "")),
            boost: ControlBinding::new((X52PRO_DEVICE, X52PRO_FIRE_D), ("", "")),
        };
        let controls = Controls::from_file_control_bindings(file_control_bindings);

        assert_eq!(
            controls.inputs_for_control(Control::CargoScoop),
            vec![Input::T2]
        );
        assert_eq!(
            controls.inputs_for_control(Control::ExternalLights),
            vec![Input::T4]
        );
        assert_eq!(
            controls.inputs_for_control(Control::LandingGear),
            vec![Input::T2, Input::T4]
        );
        assert_eq!(
            controls.inputs_for_control(Control::HyperSuperCombination),
            vec![Input::T1]
        );
        assert_eq!(
            controls.inputs_for_control(Control::Supercruise),
            vec![Input::T3]
        );
        assert_eq!(
            controls.inputs_for_control(Control::Hyperspace),
            vec![Input::T5]
        );
        assert_eq!(
            controls.inputs_for_control(Control::SilentRunning),
            vec![Input::FireA]
        );
        assert_eq!(
            controls.inputs_for_control(Control::HeatSink),
            vec![Input::T6]
        );
        assert_eq!(
            controls.inputs_for_control(Control::Hardpoints),
            vec![Input::FireB]
        );
        assert_eq!(
            controls.inputs_for_control(Control::Boost),
            vec![Input::FireD]
        );
    }

    #[test]
    fn input_from_file_input_returns_optional_inputs_given_a_file_input() {
        fn call_with(device: &str, name: &str) -> Option<Input> {
            input_from_file_input(&BindingsInput::new(device, name))
        }

        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_CLUTCH), Some(Input::Clutch));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_FIRE_A), Some(Input::FireA));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_FIRE_B), Some(Input::FireB));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_FIRE_D), Some(Input::FireD));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_FIRE_E), Some(Input::FireE));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T1), Some(Input::T1));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T2), Some(Input::T2));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T3), Some(Input::T3));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T4), Some(Input::T4));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T5), Some(Input::T5));
        assert_eq!(call_with(X52PRO_DEVICE, X52PRO_T6), Some(Input::T6));
        assert_eq!(call_with(X52PRO_DEVICE, "Other"), None);
        assert_eq!(call_with("Other", X52PRO_T2), None);
    }
}
