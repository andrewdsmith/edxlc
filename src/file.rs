use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub fn status_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous\Status.json"#)
}

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
