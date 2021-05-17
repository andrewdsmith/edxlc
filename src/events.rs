use crate::game::file::Status;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Event {
    NewJournalFile(PathBuf),
    AnimationTick,
    StatusUpdate(Status),
    Exit,
}
