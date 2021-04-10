use crate::game::file::Status;

pub enum Event {
    StatusUpdate(Status),
    Exit,
}
