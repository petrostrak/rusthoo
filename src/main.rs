use std::{fs::File, io};
use xml::reader::{EventReader, XmlEvent};

fn read_entire_xml_file(file_path: &str) -> io::Result<String> {
    let file = File::open(file_path)?;
    let event_reader = EventReader::new(file);

    let mut content = String::new();
    for event in event_reader.into_iter() {
        if let XmlEvent::Characters(text) = event.expect("TODO") {
            content.push_str(&text);
        }
    }
    Ok(content)
}

fn main() {
    let file_path = "docs.gl/g14/glClear.xhtml";

    println!(
        "{content}",
        content = read_entire_xml_file(file_path).expect("TODO")
    );
}
