use crate::game::StatusLevel;
use crate::x52pro::device::LightMode;

/// Maps status levels to light modes based on the given configuration.
pub struct StatusLevelToModeMapper {
    inactive: LightMode,
    active: LightMode,
    blocked: LightMode,
    alert: LightMode,
}

impl StatusLevelToModeMapper {
    /// Returns a new instance the mapper.
    pub fn new(
        inactive: LightMode,
        active: LightMode,
        blocked: LightMode,
        alert: LightMode,
    ) -> Self {
        Self {
            inactive,
            active,
            blocked,
            alert,
        }
    }

    pub fn map(&self, status_level: &StatusLevel) -> LightMode {
        match status_level {
            StatusLevel::Inactive => self.inactive,
            StatusLevel::Active => self.active,
            StatusLevel::Blocked => self.blocked,
            StatusLevel::Alert => self.alert,
        }
    }
}
