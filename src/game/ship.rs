use super::file::{journal::Event, LegalState, Status as FileStatus};
use log::{info, warn};

type StatusBitField = u64;

// See: https://elite-journal.readthedocs.io/en/latest/Status%20File/
const LANDING_GEAR_DEPLOYED: StatusBitField = 1 << 2;
const SUPERCRUISE: StatusBitField = 1 << 4;
const HARDPOINTS_DEPLOYED: StatusBitField = 1 << 6;
const EXTERNAL_LIGHTS_ON: StatusBitField = 1 << 8;
const CARGO_SCOOP_DEPLOYED: StatusBitField = 1 << 9;
const SILENT_RUNNING: StatusBitField = 1 << 10;
const MASS_LOCKED: StatusBitField = 1 << 16;
const FRAME_SHIFT_DRIVE_CHARGING: StatusBitField = 1 << 17;
const FRAME_SHIFT_DRIVE_COOLDOWN: StatusBitField = 1 << 18;
const OVERHEATING: StatusBitField = 1 << 20;

// These statuses are derived from sources other than the flag fields (e.g.
// legal status and journal events) so we pack them into the unused high bits.
const DOCKING: StatusBitField = 1 << (32 + 16);
const SPEEDING: StatusBitField = 1 << (32 + 17);

const STATUS_FILTER: StatusBitField = LANDING_GEAR_DEPLOYED
    | CARGO_SCOOP_DEPLOYED
    | EXTERNAL_LIGHTS_ON
    | FRAME_SHIFT_DRIVE_CHARGING
    | MASS_LOCKED
    | FRAME_SHIFT_DRIVE_COOLDOWN
    | OVERHEATING
    | SILENT_RUNNING
    | HARDPOINTS_DEPLOYED
    | SUPERCRUISE
    | SPEEDING;

/// An attribute of a `Ship` that can be associated with a value.
#[derive(Clone, Copy, PartialEq)]
pub enum Attribute {
    Boost,
    CargoScoop,
    ExternalLights,
    FrameShiftDrive,
    Hardpoints,
    HeatSink,
    LandingGear,
    SilentRunning,
    Throttle,
}

/// An association of a `Attribute` to a `StatusLevel` value for a `Ship`.
pub struct Status {
    pub attribute: Attribute,
    pub level: StatusLevel,
}

/// A status value that can associated to an `Attibute` through a `Status`
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum StatusLevel {
    Inactive,
    Active,
    Blocked,
    Alert,
}

/// A condition that can be used to specify a `StatusLevel` through a
/// `ConditionStatusLevelMapping`.
enum Condition {
    Any(StatusBitField),
    All(StatusBitField),
}

/// A mapping that defines the `Condition` that indicates a `StatusLevel`
/// applies.
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

/// A list of `ConditionStatusLevelMapping` instances that apply to an
/// `Attribute`.
struct AttributeStatusLevelMappings {
    attribute: Attribute,
    condition_status_level_mappings: Vec<ConditionStatusLevelMapping>,
}

impl AttributeStatusLevelMappings {
    fn new(
        attribute: Attribute,
        condition_status_level_mappings: Vec<ConditionStatusLevelMapping>,
    ) -> Self {
        Self {
            attribute,
            condition_status_level_mappings,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GlobalStatus {
    Normal,
    HardpointsDeployed,
}

pub struct Ship {
    status_flags: StatusBitField,
    attribute_status_level_mappings: Vec<AttributeStatusLevelMappings>,
}

impl Ship {
    /// Returns a `Ship` instance.
    pub fn new() -> Self {
        Self {
            status_flags: 0,
            attribute_status_level_mappings: vec![
                AttributeStatusLevelMappings::new(
                    Attribute::CargoScoop,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(CARGO_SCOOP_DEPLOYED),
                        StatusLevel::Active,
                    )],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::ExternalLights,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(EXTERNAL_LIGHTS_ON),
                        StatusLevel::Active,
                    )],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::FrameShiftDrive,
                    vec![
                        ConditionStatusLevelMapping::new(
                            Condition::All(FRAME_SHIFT_DRIVE_CHARGING | OVERHEATING),
                            StatusLevel::Alert,
                        ),
                        // Supercruise is higher precendence than normal
                        // flight, specifically for blocking states like
                        // hardpoints deployed.
                        ConditionStatusLevelMapping::new(
                            Condition::All(SUPERCRUISE),
                            StatusLevel::Active,
                        ),
                        ConditionStatusLevelMapping::new(
                            Condition::Any(
                                CARGO_SCOOP_DEPLOYED
                                    | MASS_LOCKED
                                    | FRAME_SHIFT_DRIVE_COOLDOWN
                                    | HARDPOINTS_DEPLOYED
                                    | LANDING_GEAR_DEPLOYED,
                            ),
                            StatusLevel::Blocked,
                        ),
                        ConditionStatusLevelMapping::new(
                            Condition::All(FRAME_SHIFT_DRIVE_CHARGING),
                            StatusLevel::Active,
                        ),
                    ],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::LandingGear,
                    vec![
                        ConditionStatusLevelMapping::new(
                            Condition::All(LANDING_GEAR_DEPLOYED),
                            StatusLevel::Active,
                        ),
                        ConditionStatusLevelMapping::new(
                            Condition::All(DOCKING),
                            StatusLevel::Alert,
                        ),
                    ],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::HeatSink,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(OVERHEATING),
                        StatusLevel::Alert,
                    )],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::SilentRunning,
                    vec![
                        ConditionStatusLevelMapping::new(
                            Condition::All(SILENT_RUNNING | OVERHEATING),
                            StatusLevel::Alert,
                        ),
                        ConditionStatusLevelMapping::new(
                            Condition::All(SILENT_RUNNING),
                            StatusLevel::Active,
                        ),
                    ],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::Hardpoints,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(HARDPOINTS_DEPLOYED),
                        StatusLevel::Active,
                    )],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::Boost,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(LANDING_GEAR_DEPLOYED),
                        StatusLevel::Blocked,
                    )],
                ),
                AttributeStatusLevelMappings::new(
                    Attribute::Throttle,
                    vec![ConditionStatusLevelMapping::new(
                        Condition::All(SPEEDING),
                        StatusLevel::Alert,
                    )],
                ),
            ],
        }
    }

    /// Updates the ship statuses give the event.
    pub fn apply_journal_event(&mut self, event: Event) {
        match event {
            Event::Docked | Event::DockingCancelled | Event::DockingTimeout => {
                info!("Docking terminated");
                self.status_flags &= !DOCKING
            }
            Event::DockingGranted => {
                info!("Docking commenced");
                self.status_flags |= DOCKING
            }
            Event::Other => warn!("Can't apply `Event::Other` journal event"),
        };
    }

    pub fn update_status(&mut self, status: FileStatus) -> bool {
        // Flatten non-flag statuses into the bit-field.
        let incoming_status_flags = status.flags as u64
            | if status.legal_state == LegalState::Speeding {
                SPEEDING
            } else {
                0
            };

        let updated_status_flags = Self::filtered_status_flags(incoming_status_flags);

        if Self::filtered_status_flags(self.status_flags) == updated_status_flags {
            false
        } else {
            // Reinstate derived status flags that were filtered out (and
            // necessarily can't have triggered a status change).
            self.status_flags = updated_status_flags | (self.status_flags & DOCKING);
            true
        }
    }

    #[cfg(test)]
    // Could refactor this into a private constructor instead.
    fn set_status(&mut self, status_flags: StatusBitField) {
        self.status_flags = status_flags;
    }

    /// Returns a vector of `Status` instances, one each for every `Attribute`,
    /// specifying the current `StatusLevel` for that attribute.
    pub fn statuses(&self) -> Vec<Status> {
        let mut statuses = Vec::new();

        // This should probably be done by functionally mapping the vector.
        for mapping in &self.attribute_status_level_mappings {
            statuses.push(Status {
                attribute: mapping.attribute,
                level: self.status_level_for_condition(&mapping.condition_status_level_mappings),
            });
        }

        statuses
    }

    /// Returns the current global (highest precendence) status for the ship.
    pub fn global_status(&self) -> GlobalStatus {
        if self.any_status_flags_set(HARDPOINTS_DEPLOYED) {
            GlobalStatus::HardpointsDeployed
        } else {
            GlobalStatus::Normal
        }
    }

    fn status_level_for_condition(
        &self,
        mappings: &Vec<ConditionStatusLevelMapping>,
    ) -> StatusLevel {
        for mapping in mappings {
            if match mapping.condition {
                Condition::Any(flags) => self.any_status_flags_set(flags),
                Condition::All(flags) => self.all_status_flags_set(flags),
            } {
                return mapping.status_level;
            }
        }

        StatusLevel::Inactive
    }

    fn any_status_flags_set(&self, flags: StatusBitField) -> bool {
        self.status_flags & flags != 0
    }

    fn all_status_flags_set(&self, flags: StatusBitField) -> bool {
        self.status_flags & flags == flags
    }

    fn filtered_status_flags(flags: StatusBitField) -> StatusBitField {
        flags & STATUS_FILTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statuses() -> Vec<StatusBitField> {
        vec![
            LANDING_GEAR_DEPLOYED,
            EXTERNAL_LIGHTS_ON,
            CARGO_SCOOP_DEPLOYED,
            SILENT_RUNNING,
            FRAME_SHIFT_DRIVE_CHARGING,
            MASS_LOCKED,
            FRAME_SHIFT_DRIVE_COOLDOWN,
            OVERHEATING,
            HARDPOINTS_DEPLOYED,
            SUPERCRUISE,
        ]
    }

    #[test]
    fn ship_update_status_sets_statuses() {
        for flag in statuses() {
            let mut ship = Ship::new();
            ship.update_status(FileStatus {
                flags: flag as u32,
                legal_state: LegalState::Other,
            });
            assert_eq!(ship.all_status_flags_set(flag), true);
        }
    }

    #[test]
    fn ship_update_status_clears_statuses() {
        for flag in statuses() {
            let mut ship = Ship::new();
            ship.set_status(flag);
            ship.update_status(FileStatus {
                flags: 0,
                legal_state: LegalState::Other,
            });
            assert_eq!(ship.all_status_flags_set(flag), false);
        }
    }

    #[test]
    fn ship_update_status_returns_true_on_change() {
        for flag in statuses() {
            let mut ship = Ship::new();
            assert_eq!(
                ship.update_status(FileStatus {
                    flags: flag as u32,
                    legal_state: LegalState::Other,
                }),
                true
            );
            assert_eq!(
                ship.update_status(FileStatus {
                    flags: flag as u32,
                    legal_state: LegalState::Other,
                }),
                false
            );
        }
    }

    #[test]
    fn ship_update_status_does_not_clobber_derived_states() {
        let mut ship = Ship::new();
        ship.set_status(DOCKING);
        ship.update_status(FileStatus {
            flags: LANDING_GEAR_DEPLOYED as u32,
            legal_state: LegalState::Other,
        });
        assert_eq!(ship.all_status_flags_set(DOCKING), true);
    }

    fn assert_status(status_flags: StatusBitField, attribute: Attribute, level: StatusLevel) {
        let mut ship = Ship::new();
        ship.set_status(status_flags);
        let statuses = ship.statuses();
        let status = statuses
            .iter()
            .find(|&status| status.attribute == attribute)
            .expect("Statuses did not include expected attribute");

        assert_eq!(status.level, level);
    }

    #[test]
    fn ship_update_status_sets_speeding() {
        let mut ship = Ship::new();
        ship.update_status(FileStatus {
            flags: 0,
            legal_state: LegalState::Speeding,
        });
        assert_eq!(ship.all_status_flags_set(SPEEDING), true);
    }

    #[test]
    fn ship_update_status_clears_speeding() {
        let mut ship = Ship::new();
        ship.update_status(FileStatus {
            flags: 0,
            legal_state: LegalState::Other,
        });
        assert_eq!(ship.all_status_flags_set(SPEEDING), false);
    }

    #[test]
    fn ship_update_speeding_returns_true_on_change() {
        let mut ship = Ship::new();
        assert_eq!(
            ship.update_status(FileStatus {
                flags: 0,
                legal_state: LegalState::Speeding,
            }),
            true
        );
        assert_eq!(
            ship.update_status(FileStatus {
                flags: 0,
                legal_state: LegalState::Speeding,
            }),
            false
        );
    }

    #[test]
    fn ship_update_status_speeding_does_not_clobber_derived_states() {
        let mut ship = Ship::new();
        ship.set_status(DOCKING);
        ship.update_status(FileStatus {
            flags: 0,
            legal_state: LegalState::Speeding,
        });
        assert_eq!(ship.all_status_flags_set(DOCKING), true);
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
    fn zero_state_maps_to_silent_running_inactive() {
        assert_status(0, Attribute::SilentRunning, StatusLevel::Inactive);
    }

    #[test]
    fn zero_state_maps_to_throttle_inactive() {
        assert_status(0, Attribute::Throttle, StatusLevel::Inactive);
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
    fn frame_shift_drive_charging_and_supercruise_and_overheating_maps_to_frame_shift_drive_alert()
    {
        assert_status(
            FRAME_SHIFT_DRIVE_CHARGING + SUPERCRUISE + OVERHEATING,
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
    fn hardpoints_deployed_maps_to_hardpoints_active() {
        assert_status(
            HARDPOINTS_DEPLOYED,
            Attribute::Hardpoints,
            StatusLevel::Active,
        );
    }

    #[test]
    fn hardpoints_deployed_maps_to_frame_shift_drive_blocked() {
        assert_status(
            HARDPOINTS_DEPLOYED,
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
    fn landing_gear_not_deployed_and_docking_maps_to_landing_gear_alert() {
        assert_status(DOCKING, Attribute::LandingGear, StatusLevel::Alert);
    }

    #[test]
    fn landing_gear_deployed_maps_to_boost_blocked() {
        assert_status(
            LANDING_GEAR_DEPLOYED,
            Attribute::Boost,
            StatusLevel::Blocked,
        );
    }

    #[test]
    fn landing_gear_deployed_maps_to_frame_shift_drive_blocked() {
        assert_status(
            LANDING_GEAR_DEPLOYED,
            Attribute::FrameShiftDrive,
            StatusLevel::Blocked,
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
    #[test]
    fn silent_running_maps_to_silent_running_active() {
        assert_status(
            SILENT_RUNNING,
            Attribute::SilentRunning,
            StatusLevel::Active,
        );
    }

    #[test]
    fn silent_running_and_overheating_maps_to_silent_running_alert() {
        assert_status(
            SILENT_RUNNING + OVERHEATING,
            Attribute::SilentRunning,
            StatusLevel::Alert,
        );
    }

    #[test]
    fn speeding_maps_to_throttle_alert() {
        assert_status(SPEEDING, Attribute::Throttle, StatusLevel::Alert);
    }

    #[test]
    fn supercruise_maps_to_frame_shift_drive_active() {
        assert_status(SUPERCRUISE, Attribute::FrameShiftDrive, StatusLevel::Active);
    }

    #[test]
    fn supercruise_and_hardpoints_deployed_and_maps_to_frame_shift_drive_active() {
        assert_status(
            SUPERCRUISE + HARDPOINTS_DEPLOYED,
            Attribute::FrameShiftDrive,
            StatusLevel::Active,
        );
    }

    fn assert_global_status(status_flags: StatusBitField, expected_global_status: GlobalStatus) {
        let mut ship = Ship::new();
        ship.set_status(status_flags);
        assert_eq!(ship.global_status(), expected_global_status);
    }

    #[test]
    fn global_status_precedence_rules() {
        assert_global_status(0, GlobalStatus::Normal);
        assert_global_status(HARDPOINTS_DEPLOYED, GlobalStatus::HardpointsDeployed);
    }
}
