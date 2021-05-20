mod control_bindings;
pub mod journal;

pub use control_bindings::*;
use glob::glob;
use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub fn bindings_file_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("Can't find user app data directory")
        .join(r#"Frontier Developments\Elite Dangerous\Options\Bindings\Custom.4.0.binds"#)
}

/// Returns a `PathBuf` for the directory containing the game's journal files.
pub fn journal_dir_path() -> PathBuf {
    dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous"#) // TODO const & use below
}

/// Optionally returns a `PathBuf` for the latest journal file if one is found.
pub fn latest_journal_file_path() -> Option<PathBuf> {
    let journal_file_pattern = dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous\Journal*.log"#);
    let journal_file_pattern = journal_file_pattern
        .to_str()
        .expect("Can't convert user home directory to UTF-8");

    debug!("Journal file pattern: {:?}", journal_file_pattern);

    glob(journal_file_pattern)
        .expect("Can't search for journal files")
        .filter_map(Result::ok)
        .max_by_key(|path| {
            path.metadata()
                .expect("Can't get journal file metadata")
                .modified()
                .expect("Can't get journal file modified date")
        })
}

pub fn status_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Can't find user home directory")
        .join(r#"Saved Games\Frontier Developments\Elite Dangerous\Status.json"#)
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
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
            debug!("Status file empty");
            None
        } else {
            Some(Status::from_json(json))
        }
    }

    pub fn from_json(json: String) -> Status {
        debug!("Parsing JSON: {}", json);
        serde_json::from_str(&json).expect("Could not parse status JSON")
    }
}
