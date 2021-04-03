use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Eq, PartialEq)]
pub struct Status {
    #[serde(rename = "Flags")]
    pub flags: u32,
}

impl Status {
    pub fn from_file(path: &PathBuf) -> Status {
        Status::from_json(fs::read_to_string(path).expect("Could not read status file"))
    }

    pub fn from_json(json: String) -> Status {
        println!("{}", json);
        serde_json::from_str(&json).expect("Could not parse status JSON")
    }
}
