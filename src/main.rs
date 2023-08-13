use std::{
    fs::{read_dir, File},
    io,
    path::Path,
};
use xml::reader::{EventReader, XmlEvent};

fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
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

fn main() -> io::Result<()> {
    let dir_path = "docs.gl/gl4";
    let dir = read_dir(dir_path)?;

    for file in dir {
        let file_path = file?.path();
        let content = read_entire_xml_file(&file_path)?;
        println!("{file_path:?} => {size}", size = content.len());
    }

    Ok(())
}
