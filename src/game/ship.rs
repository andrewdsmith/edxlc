use super::file::Status as FileStatus;

type StatusBitField = u32;

// See: https://elite-journal.readthedocs.io/en/latest/Status%20File/
const LANDING_GEAR_DEPLOYED: StatusBitField = 1 << 2;
const EXTERNAL_LIGHTS_ON: StatusBitField = 1 << 8;
const CARGO_SCOOP_DEPLOYED: StatusBitField = 1 << 9;
const MASS_LOCKED: StatusBitField = 1 << 16;
const FRAME_SHIFT_DRIVE_CHARGING: StatusBitField = 1 << 17;
const FRAME_SHIFT_DRIVE_COOLDOWN: StatusBitField = 1 << 18;
const OVERHEATING: StatusBitField = 1 << 20;

const STATUS_FILTER: StatusBitField = LANDING_GEAR_DEPLOYED
    | CARGO_SCOOP_DEPLOYED
    | EXTERNAL_LIGHTS_ON
    | FRAME_SHIFT_DRIVE_CHARGING
    | MASS_LOCKED
    | FRAME_SHIFT_DRIVE_COOLDOWN
    | OVERHEATING;

const FRAME_SHIFT_DRIVE_BLOCKED: StatusBitField =
    CARGO_SCOOP_DEPLOYED | MASS_LOCKED | FRAME_SHIFT_DRIVE_COOLDOWN;

/// An attribute of a `Ship` that can be associated with a value.
#[derive(PartialEq)]
pub enum Attribute {
    CargoScoop,
    ExternalLights,
    FrameShiftDrive,
    HeatSink,
    LandingGear,
}

/// An association of a `Attribute` to a `StatusLevel` value for a `Ship`.
pub struct Status {
    pub attribute: Attribute,
    pub level: StatusLevel,
}

/// A status value that can associated to an `Attibute` through a `Status`
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum StatusLevel {
    Inactive,
    Active,
    Blocked,
    Alert,
}

enum Condition {
    Any(StatusBitField),
    All(StatusBitField),
}

struct ConditionStatusLevelMapping {
    condition: Condition,
    status_level: StatusLevel,
}

impl ConditionStatusLevelMapping {
    fn new(condition: Condition, status_level: StatusLevel) -> Self {
        Self {
            condition,
            status_level,
        }
    }
}

pub struct Ship {
    status_flags: StatusBitField,
}

impl Ship {
    /// Returns a `Ship` instance.
    pub fn new() -> Self {
        Self { status_flags: 0 }
    }

    pub fn update_status(&mut self, status: FileStatus) -> bool {
        let updated_status_flags = Self::filtered_status_flags(status.flags);

        if self.status_flags == updated_status_flags {
            false
        } else {
            self.status_flags = updated_status_flags;
            true
        }
    }

    /// Returns a vector of `Status` instances, one each for every `Attribute`,
    /// specifying the current `StatusLevel` for that attribute.
    pub fn statuses(&self) -> Vec<Status> {
        let mut statuses = Vec::new();

        statuses.push(Status {
            attribute: Attribute::CargoScoop,
            level: self.status_level_for_condition(vec![ConditionStatusLevelMapping::new(
                Condition::All(CARGO_SCOOP_DEPLOYED),
                StatusLevel::Active,
            )]),
        });
        statuses.push(Status {
            attribute: Attribute::ExternalLights,
            level: self.status_level_for_condition(vec![ConditionStatusLevelMapping::new(
                Condition::All(EXTERNAL_LIGHTS_ON),
                StatusLevel::Active,
            )]),
        });
        statuses.push(Status {
            attribute: Attribute::FrameShiftDrive,
            level: self.status_level_for_condition(vec![
                ConditionStatusLevelMapping::new(
                    Condition::All(FRAME_SHIFT_DRIVE_CHARGING | OVERHEATING),
                    StatusLevel::Alert,
                ),
                ConditionStatusLevelMapping::new(
                    Condition::Any(FRAME_SHIFT_DRIVE_BLOCKED),
                    StatusLevel::Blocked,
                ),
                ConditionStatusLevelMapping::new(
                    Condition::All(FRAME_SHIFT_DRIVE_CHARGING),
                    StatusLevel::Active,
                ),
            ]),
        });
        statuses.push(Status {
            attribute: Attribute::LandingGear,
            level: self.status_level_for_condition(vec![ConditionStatusLevelMapping::new(
                Condition::All(LANDING_GEAR_DEPLOYED),
                StatusLevel::Active,
            )]),
        });
        statuses.push(Status {
            attribute: Attribute::HeatSink,
            level: self.status_level_for_condition(vec![ConditionStatusLevelMapping::new(
                Condition::All(OVERHEATING),
                StatusLevel::Alert,
            )]),
        });

        statuses
    }

    fn status_level_for_condition(
        &self,
        mappings: Vec<ConditionStatusLevelMapping>,
    ) -> StatusLevel {
        for mapping in mappings {
            if match mapping.condition {
                Condition::Any(flags) => self.status_flags & flags != 0,
                Condition::All(flags) => self.status_flags & flags == flags,
            } {
                return mapping.status_level;
            }
        }

        StatusLevel::Inactive
    }

    fn filtered_status_flags(flags: StatusBitField) -> StatusBitField {
        flags & STATUS_FILTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_update_status_returns_true_on_change() {
        for flag in vec![
            LANDING_GEAR_DEPLOYED,
            EXTERNAL_LIGHTS_ON,
            CARGO_SCOOP_DEPLOYED,
            FRAME_SHIFT_DRIVE_CHARGING,
            MASS_LOCKED,
            FRAME_SHIFT_DRIVE_COOLDOWN,
            OVERHEATING,
        ] {
            let mut ship = Ship { status_flags: 0 };

            assert_eq!(ship.update_status(FileStatus { flags: flag }), true);
            assert_eq!(ship.update_status(FileStatus { flags: flag }), false);
        }
    }

    fn assert_status(status_flags: u32, attribute: Attribute, level: StatusLevel) {
        let ship = Ship { status_flags };
        let statuses = ship.statuses();
        let status = statuses
            .iter()
            .find(|&status| status.attribute == attribute)
            .expect("Statuses did not include expected attribute");

        assert_eq!(status.level, level);
    }

    #[test]
    fn zero_state_maps_to_cargo_scoop_inactive() {
        assert_status(0, Attribute::CargoScoop, StatusLevel::Inactive);
    }

    #[test]
    fn zero_state_maps_to_external_lights_inactive() {
        assert_status(0, Attribute::ExternalLights, StatusLevel::Inactive);
    }

    #[test]
    fn zero_state_maps_to_frame_shift_drive_inactive() {
        assert_status(0, Attribute::FrameShiftDrive, StatusLevel::Inactive);
    }

    #[test]
    fn zero_state_maps_to_landing_gear_inactive() {
        assert_status(0, Attribute::LandingGear, StatusLevel::Inactive);
    }

    #[test]
    fn cargo_scoop_deployed_maps_to_cargo_scoop_active() {
        assert_status(
            CARGO_SCOOP_DEPLOYED,
            Attribute::CargoScoop,
            StatusLevel::Active,
        );
    }

    #[test]
    fn cargo_scoop_deployed_maps_to_frame_shift_drive_blocked() {
        assert_status(
            CARGO_SCOOP_DEPLOYED,
            Attribute::FrameShiftDrive,
            StatusLevel::Blocked,
        );
    }

    #[test]
    fn external_lights_on_maps_to_external_lights_active() {
        assert_status(
            EXTERNAL_LIGHTS_ON,
            Attribute::ExternalLights,
            StatusLevel::Active,
        );
    }

    #[test]
    fn frame_shift_drive_charging_maps_to_frame_shift_drive_active() {
        assert_status(
            FRAME_SHIFT_DRIVE_CHARGING,
            Attribute::FrameShiftDrive,
            StatusLevel::Active,
        );
    }

    #[test]
    fn frame_shift_drive_charging_and_overheating_maps_to_frame_shift_drive_alert() {
        assert_status(
            FRAME_SHIFT_DRIVE_CHARGING + OVERHEATING,
            Attribute::FrameShiftDrive,
            StatusLevel::Alert,
        );
    }

    #[test]
    fn frame_shift_drive_cooldown_maps_to_frame_shift_drive_blocked() {
        assert_status(
            FRAME_SHIFT_DRIVE_COOLDOWN,
            Attribute::FrameShiftDrive,
            StatusLevel::Blocked,
        );
    }

    #[test]
    fn landing_gear_deployed_maps_to_landing_gear_active() {
        assert_status(
            LANDING_GEAR_DEPLOYED,
            Attribute::LandingGear,
            StatusLevel::Active,
        );
    }

    #[test]
    fn mass_locked_maps_to_frame_shift_drive_blocked() {
        assert_status(
            MASS_LOCKED,
            Attribute::FrameShiftDrive,
            StatusLevel::Blocked,
        );
    }

    #[test]
    fn overheating_maps_to_heat_sink_alert() {
        assert_status(OVERHEATING, Attribute::HeatSink, StatusLevel::Alert);
    }
}
