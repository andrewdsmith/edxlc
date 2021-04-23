use super::file::Status as FileStatus;

// See: https://elite-journal.readthedocs.io/en/latest/Status%20File/
const LANDING_GEAR_DEPLOYED: u32 = 1 << 2;
const EXTERNAL_LIGHTS_ON: u32 = 1 << 8;
const CARGO_SCOOP_DEPLOYED: u32 = 1 << 9;
const FRAME_SHIFT_DRIVE_CHARGING: u32 = 1 << 17;

const STATUS_FILTER: u32 =
    LANDING_GEAR_DEPLOYED | CARGO_SCOOP_DEPLOYED | EXTERNAL_LIGHTS_ON | FRAME_SHIFT_DRIVE_CHARGING;

/// An attribute of a `Ship` that can be associated with a value.
#[derive(PartialEq)]
pub enum Attribute {
    CargoScoop,
    ExternalLights,
    FrameShiftDrive,
    LandingGear,
}

/// An association of a `Attribute` to a `StatusLevel` value for a `Ship`.
pub struct Status {
    pub attribute: Attribute,
    pub level: StatusLevel,
}

/// A status value that can associated to an `Attibute` through a `Status`
#[derive(Debug, PartialEq)]
pub enum StatusLevel {
    Inactive,
    Active,
}

pub struct Ship {
    status_flags: u32,
}

impl Ship {
    pub fn from_status(status: FileStatus) -> Self {
        Self {
            status_flags: Self::filtered_status_flags(status.flags),
        }
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
            level: self.active_if_flag(CARGO_SCOOP_DEPLOYED),
        });
        statuses.push(Status {
            attribute: Attribute::ExternalLights,
            level: self.active_if_flag(EXTERNAL_LIGHTS_ON),
        });
        statuses.push(Status {
            attribute: Attribute::FrameShiftDrive,
            level: self.active_if_flag(FRAME_SHIFT_DRIVE_CHARGING),
        });
        statuses.push(Status {
            attribute: Attribute::LandingGear,
            level: self.active_if_flag(LANDING_GEAR_DEPLOYED),
        });

        statuses
    }

    fn active_if_flag(&self, flag: u32) -> StatusLevel {
        if self.is_status_flag_set(flag) {
            StatusLevel::Active
        } else {
            StatusLevel::Inactive
        }
    }

    fn is_status_flag_set(&self, flag: u32) -> bool {
        (self.status_flags & flag) != 0
    }

    fn filtered_status_flags(flags: u32) -> u32 {
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
    fn landing_gear_deployed_maps_to_landing_gear_active() {
        assert_status(
            LANDING_GEAR_DEPLOYED,
            Attribute::LandingGear,
            StatusLevel::Active,
        );
    }
}
