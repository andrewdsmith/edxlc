mod controls;
pub mod file;

pub use controls::*;
use file::Status;

// See: https://elite-journal.readthedocs.io/en/latest/Status%20File/
const LANDING_GEAR_DEPLOYED: u32 = 1 << 2;
const EXTERNAL_LIGHTS_ON: u32 = 1 << 8;
const CARGO_SCOOP_DEPLOYED: u32 = 1 << 9;

const STATUS_FILTER: u32 = LANDING_GEAR_DEPLOYED | CARGO_SCOOP_DEPLOYED | EXTERNAL_LIGHTS_ON;

pub struct Ship {
    status_flags: u32,
}

impl Ship {
    pub fn from_status(status: Status) -> Self {
        Self {
            status_flags: Self::filtered_status_flags(status.flags),
        }
    }

    pub fn update_status(&mut self, status: Status) -> bool {
        let updated_status_flags = Self::filtered_status_flags(status.flags);

        if self.status_flags == updated_status_flags {
            false
        } else {
            self.status_flags = updated_status_flags;
            true
        }
    }

    pub fn landing_gear_deployed(&self) -> bool {
        self.is_status_flag_set(LANDING_GEAR_DEPLOYED)
    }

    pub fn cargo_scoop_deployed(&self) -> bool {
        self.is_status_flag_set(CARGO_SCOOP_DEPLOYED)
    }

    pub fn external_lights_on(&self) -> bool {
        self.is_status_flag_set(EXTERNAL_LIGHTS_ON)
    }

    fn is_status_flag_set(&self, flag: u32) -> bool {
        (self.status_flags & flag) != 0
    }

    fn filtered_status_flags(flags: u32) -> u32 {
        flags & STATUS_FILTER
    }
}
