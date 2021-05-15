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

/// Watch the journal file at the given path for changes using the watcher then
/// send a `JournalEvent` using the channel sender.
pub fn watch(file_path: PathBuf, watcher: &mut Hotwatch, tx: &Sender<events::Event>) {
    info!("Watching journal file at {}", file_path.to_str().unwrap());

    // By creating the reader outside the scope of the watcher closure we keep
    // it open between events, meaning each time the event fires we only read
    // the newly added event lines.
    let file = File::open(&file_path).expect("Can't open journal file");
    let mut reader = BufReader::new(file);

    let tx = tx.clone();

    // Catch up with the event lines already in the file. There is a race here
    // because the file could be written to between this call and the watcher
    // starting but we'll catch up as soon as the file it written to again.
    read_events(&mut reader, event_from_json, &tx);

    watcher
        .watch(file_path, move |event: hotwatch::Event| {
            if let hotwatch::Event::Write(_) = event {
                read_events(&mut reader, event_from_json, &tx);
            }
        })
        .expect("Can't watch journal file");
}

/// Read events from the journal file using the given reader and parse. Send supported journal
/// events on the channel sender.
fn read_events<T>(reader: &mut BufReader<T>, parser: fn(&str) -> Event, tx: &Sender<events::Event>)
where
    T: std::io::Read,
{
    let mut line = String::new();
    while reader
        .read_line(&mut line)
        .expect("Can't read journal file")
        != 0
    {
        // Could collapse duplicate and redundant events here, i.e. when
        // reading an existing journal file at start-up. Alternatively collect
        // all events and then indicate which one is the last to delay device
        // updates until the end.
        match parser(&line) {
            Event::Other => (),
            event => {
                debug!("Sending journal event {:?}", event);
                tx.send(events::Event::JournalEvent(event))
                    .expect("Can't send journal event message");
            }
        }

        // The `read_line` call above *appends* to the string but we want to
        // parse one line at a time.
        line.clear();
    }
}

/// Returns a journal event parsed from the given JSON string.
fn event_from_json(json: &str) -> Event {
    serde_json::from_str(&json).expect("Can't parse journal event JSON")
}

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
    use std::sync::mpsc;

    #[test]
    fn read_events_sends_events_except_other() {
        let events = "LINE1\nLINE2\n".as_bytes();
        let mut reader = BufReader::new(events);
        let (tx, rx) = mpsc::channel();
        fn fake_parser(json: &str) -> Event {
            match json {
                "LINE1\n" => Event::DockingGranted,
                "LINE2\n" => Event::Other,
                _ => panic!("Unexpected line value passed to parser '{}'", json),
            }
        }

        read_events(&mut reader, fake_parser, &tx);

        assert_eq!(
            rx.try_recv().unwrap(),
            events::Event::JournalEvent(Event::DockingGranted)
        );

        // Does not send `Event::Other`.
        assert_eq!(rx.try_recv().is_err(), true);
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
