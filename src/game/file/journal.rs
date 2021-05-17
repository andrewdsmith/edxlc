use crate::events;
use hotwatch::Hotwatch;
use log::{debug, info};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::mpsc::Sender,
};

/// Watch the given directory for new journal files then send a
/// `NewJournalFile` event using the channel sender.
pub fn watch_dir(dir_path: PathBuf, watcher: &mut Hotwatch, tx: &Sender<events::Event>) {
    let tx = tx.clone();

    watcher
        .watch(dir_path, move |event: hotwatch::Event| {
            debug!("Journal directory watch event: {:?}", event);

            if let hotwatch::Event::Create(file_path) = event {
                let file_name = file_path
                    .file_name()
                    .expect("Can't get file name for created file")
                    .to_str()
                    .expect("Can't convert file name to UTF-8");

                debug!("New file in journal directory: {}", file_name);

                if file_name.starts_with("Journal") && file_name.ends_with(".log") {
                    tx.send(events::Event::NewJournalFile(file_path))
                        .expect("Can't send new journal file message");
                }
            }
        })
        .expect("Can't watch journal directory");
}

/// A stateful reader that can be called repeatedly, each time returning only
/// the new journal events appended to journal file since the last call.
pub struct JournalReader {
    journal_buf_reader: Option<BufReader<File>>,
}

impl JournalReader {
    /// Returns a new instance of the reader not associated with any journal
    /// file.
    pub fn new() -> Self {
        JournalReader {
            journal_buf_reader: None,
        }
    }

    /// Opens the given file for reading.
    pub fn open(&mut self, journal_file_path: PathBuf) {
        debug!("Opening journal file: {:?}", journal_file_path);
        let journal_file = File::open(&journal_file_path).expect("Can't open journal file");
        self.journal_buf_reader = Some(BufReader::new(journal_file));
    }

    /// When called before `open` returns an empty vector. When called the first
    /// time after `open` returns all the journal events currently in the
    /// journal. On subsequent calls returns the new journal events appended to
    /// journal file since the last call.
    pub fn new_events(&mut self) -> Vec<Event> {
        if let Some(reader) = &mut self.journal_buf_reader {
            events_from_buf_reader(reader, event_from_json)
        } else {
            vec![]
        }
    }
}

/// Read lines from the given reader and map to journal events using the given
/// parser, filtering out `Event::Other`.
fn events_from_buf_reader<T>(reader: &mut BufReader<T>, parser: fn(&str) -> Event) -> Vec<Event>
where
    T: std::io::Read,
{
    let mut events = Vec::new();
    let mut line = String::new();

    while reader
        .read_line(&mut line)
        .expect("Can't read journal file")
        != 0
    {
        match parser(&line) {
            Event::Other => (),
            event => {
                info!("Journal event {:?}", event);
                events.push(event);
            }
        }

        // The `read_line` call above *appends* to the string but we want to
        // parse one line at a time.
        line.clear();
    }

    events
}

/// Returns a journal event parsed from the given JSON string.
fn event_from_json(json: &str) -> Event {
    serde_json::from_str(&json).expect("Can't parse journal event JSON")
}

// This enum should be renamed `JournalEvent` to reduce name collisions outside
// this module (given it's public).
#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "event")]
pub enum Event {
    Docked,
    DockingCancelled,
    DockingGranted,
    DockingTimeout,
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_from_buf_reader_maps_each_line_to_an_event() {
        let events = "LINE1\nLINE2\n".as_bytes();
        let mut reader = BufReader::new(events);
        fn fake_parser(json: &str) -> Event {
            match json {
                "LINE1\n" => Event::DockingGranted,
                "LINE2\n" => Event::Other,
                _ => panic!("Unexpected line value passed to parser '{}'", json),
            }
        }

        // Filters out `Event::Other`.
        assert_eq!(
            events_from_buf_reader(&mut reader, fake_parser),
            vec![Event::DockingGranted]
        );
    }

    #[test]
    fn event_from_json_returns_parsed_journal_events() {
        assert_eq!(
            event_from_json(
                r#"{ "timestamp":"2021-05-14T00:00:00Z", "event":"Docked", "StationName":"A", "StationType":"B", "StarSystem":"C", "SystemAddress":1, "MarketID":2, "StationFaction":{ "Name":"D" }, "StationGovernment":"E", "StationGovernment_Localised":"F", "StationAllegiance":"G", "StationServices":[ "H" ], "StationEconomy":"I", "StationEconomy_Localised":"J", "StationEconomies":[ { "Name":"K", "Name_Localised":"L", "Proportion":3.0 } ], "DistFromStarLS":4.0 }"#
            ),
            Event::Docked
        );
        assert_eq!(
            event_from_json(
                r#"{ "timestamp":"2021-05-13T00:00:00Z", "event":"DockingCancelled", "MarketID":1, "StationName":"A", "StationType":"B" }"#
            ),
            Event::DockingCancelled
        );
        assert_eq!(
            event_from_json(
                r#"{ "timestamp":"2021-05-12T00:00:00Z", "event":"DockingGranted", "LandingPad":1, "MarketID":1, "StationName":"A", "StationType":"B" }"#
            ),
            Event::DockingGranted
        );
        assert_eq!(
            event_from_json(
                r#"{ "timestamp":"2021-05-14T00:00:00Z", "event":"DockingTimeout", "MarketID":1, "StationName":"A", "StationType":"B" }"#
            ),
            Event::DockingTimeout
        );
        assert_eq!(
            event_from_json(
                r#"{ "timestamp":"2021-05-12T00:00:00Z", "event":"Music", "MusicTrack":"NoTrack" }"#
            ),
            Event::Other
        );
    }
}
