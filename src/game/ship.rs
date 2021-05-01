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
            level: self.map_level_to_flag(StatusLevel::Active, CARGO_SCOOP_DEPLOYED),
        });
        statuses.push(Status {
            attribute: Attribute::ExternalLights,
            level: self.map_level_to_flag(StatusLevel::Active, EXTERNAL_LIGHTS_ON),
        });
        statuses.push(Status {
            attribute: Attribute::FrameShiftDrive,
            level: self.map_flags_to_status(FRAME_SHIFT_DRIVE_CHARGING, FRAME_SHIFT_DRIVE_BLOCKED),
        });
        statuses.push(Status {
            attribute: Attribute::LandingGear,
            level: self.map_level_to_flag(StatusLevel::Active, LANDING_GEAR_DEPLOYED),
        });
        statuses.push(Status {
            attribute: Attribute::HeatSink,
            level: self.map_level_to_flag(StatusLevel::Alert, OVERHEATING),
        });

        statuses
    }

    fn map_flags_to_status(
        &self,
        active_flag: StatusBitField,
        blocking_flag: StatusBitField,
    ) -> StatusLevel {
        if self.is_status_flag_set(blocking_flag) {
            StatusLevel::Blocked
        } else {
            self.map_level_to_flag(StatusLevel::Active, active_flag)
        }
    }

    fn map_level_to_flag(&self, status_level: StatusLevel, flag: StatusBitField) -> StatusLevel {
        if self.is_status_flag_set(flag) {
            status_level
        } else {
            StatusLevel::Inactive
        }
    }

    fn is_status_flag_set(&self, flag: StatusBitField) -> bool {
        (self.status_flags & flag) != 0
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
