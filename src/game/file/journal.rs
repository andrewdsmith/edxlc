use hotwatch::Hotwatch;
use log::{debug, info};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

/// Watch the journal file at the given path for changes using the watcher.
pub fn watch(file_path: PathBuf, watcher: &mut Hotwatch) {
    info!("Watching journal file at {}", file_path.to_str().unwrap());

    // By creating the reader outside the scope of the watcher closure we keep
    // it open between events, meaning each time the event fires we only read
    // the newly added event lines.
    let file = File::open(&file_path).expect("Can't open journal file");
    let mut reader = BufReader::new(file);

    // Catch up with the event lines already in the file. There is a race here
    // because the file could be written to between this call and the watcher
    // starting but we'll catch up as soon as the file it written to again.
    read_events(&mut reader);

    watcher
        .watch(file_path, move |event: hotwatch::Event| {
            if let hotwatch::Event::Write(_path) = event {
                read_events(&mut reader);
            }
        })
        .expect("Can't watch journal file");
}

/// Read events from the journal file using the given reader.
fn read_events(reader: &mut BufReader<File>) {
    let mut line = String::new();
    while reader
        .read_line(&mut line)
        .expect("Can't read journal file")
        != 0
    {
        // Here we will parse the lines and raise events to the event loop.
        debug!("Journal line: {}", line);
    }
}
