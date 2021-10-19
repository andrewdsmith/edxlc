mod control_bindings;
pub mod journal;

pub use control_bindings::*;
use glob::glob;
use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default)]
pub struct Status {
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "LegalState")]
    pub legal_state: LegalState,
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

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum LegalState {
    Speeding,
    #[serde(other)]
    Other,
}

impl Default for LegalState {
    fn default() -> Self {
        LegalState::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_json_parses_flags() {
        let json = String::from(
            r#"{"timestamp": "2021-08-21T21:36:35Z", "event": "Status", "Flags": 4, "LegalState": "Speeding"}"#,
        );

        assert_eq!(
            Status::from_json(json),
            Status {
                flags: 4,
                legal_state: LegalState::Speeding
            }
        );
    }

    #[test]
    fn status_from_json_parses_when_legal_state_missing() {
        let json =
            String::from(r#"{"timestamp": "2021-08-21T21:36:35Z", "event": "Status", "Flags": 0}"#);
        assert_eq!(Status::from_json(json).legal_state, LegalState::Other);
    }
}
