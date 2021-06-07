use crate::game::StatusLevel;
use crate::x52pro::device::RedAmberGreenLightMode;

/// Maps status levels to light modes based on the given configuration.
pub struct StatusLevelToModeMapper {
    inactive: RedAmberGreenLightMode,
    active: RedAmberGreenLightMode,
    blocked: RedAmberGreenLightMode,
    alert: RedAmberGreenLightMode,
}

impl StatusLevelToModeMapper {
    /// Returns a new instance the mapper.
    pub fn new(
        inactive: RedAmberGreenLightMode,
        active: RedAmberGreenLightMode,
        blocked: RedAmberGreenLightMode,
        alert: RedAmberGreenLightMode,
    ) -> Self {
        Self {
            inactive,
            active,
            blocked,
            alert,
        }
    }

    pub fn map(&self, status_level: &StatusLevel) -> RedAmberGreenLightMode {
        match status_level {
            StatusLevel::Inactive => self.inactive,
            StatusLevel::Active => self.active,
            StatusLevel::Blocked => self.blocked,
            StatusLevel::Alert => self.alert,
        }
    }
}
