use std::fs::File;
use std::process::exit;
use xml::reader::{EventReader, XmlEvent};

fn main() {
    let file_path = "docs.gl/g14/glClear.xhtml";
    let file = File::open(file_path).unwrap_or_else(|err| {
        eprintln!("ERROR: could not read file {file_path}: {err}");
        exit(1)
    });
    let event_reader = EventReader::new(file);

    for event in event_reader.into_iter() {
        let event = event.unwrap_or_else(|err| {
            eprintln!("ERROR: could not read next XML event: {err}");
            exit(1)
        });
        if let XmlEvent::Characters(text) = event {
            println!("{text}");
        }
    }
}
