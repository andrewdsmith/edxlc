use crate::game::file::{journal, Status};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Event {
    NewJournalFile(PathBuf),
    AnimationTick,
    StatusUpdate(Status),
    JournalEvent(journal::Event),
    Exit,
}
