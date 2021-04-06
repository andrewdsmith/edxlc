use crate::file::Status;

pub struct Ship {
    status_flags: u32,
}

impl Ship {
    const LANDING_GEAR_DEPLOYED: u32 = 1 << 2;

    pub fn from_status(status: Status) -> Ship {
        Ship {
            status_flags: Ship::filtered_status_flags(status.flags),
        }
    }

    pub fn update_status(&mut self, status: Status) -> bool {
        let updated_status_flags = Ship::filtered_status_flags(status.flags & 4);

        if self.status_flags == updated_status_flags {
            false
        } else {
            self.status_flags = updated_status_flags;
            true
        }
    }

    pub fn landing_gear_deployed(&self) -> bool {
        self.is_status_flag_set(Ship::LANDING_GEAR_DEPLOYED)
    }

    fn is_status_flag_set(&self, flag: u32) -> bool {
        (self.status_flags & flag) != 0
    }

    fn filtered_status_flags(flags: u32) -> u32 {
        flags & Ship::LANDING_GEAR_DEPLOYED
    }
}
