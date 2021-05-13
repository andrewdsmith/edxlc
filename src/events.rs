use crate::game::file::{journal, Status};

#[derive(Debug, PartialEq)]
pub enum Event {
    AnimationTick,
    StatusUpdate(Status),
    JournalEvent(journal::Event),
    Exit,
}
