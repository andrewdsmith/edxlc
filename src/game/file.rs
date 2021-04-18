mod control_bindings;

pub use control_bindings::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub fn bindings_file_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("Can't find user app data directory")
        .join(r#"Frontier Developments\Elite Dangerous\Options\Bindings\Custom.3.0.binds"#)
}

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
    // Returns the status in the given file. Returns an Option because the file cannot be
    // guaranteed to contain a readable status at all times.
    pub fn from_file(path: &PathBuf) -> Option<Status> {
        let json = fs::read_to_string(path).expect("Could not read status file");

        // When exiting the game temporarily writes an empty file.
        if json == "" {
            println!("Status file empty");
            None
        } else {
            Some(Status::from_json(json))
        }
    }

    pub fn from_json(json: String) -> Status {
        println!("Parsing JSON: {}", json);
        serde_json::from_str(&json).expect("Could not parse status JSON")
    }
}
