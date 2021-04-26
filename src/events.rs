use crate::game::file::Status;

pub enum Event {
    AnimationTick,
    StatusUpdate(Status),
    Exit,
}
