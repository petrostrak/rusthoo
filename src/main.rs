use std::{
    collections::HashMap,
    env,
    fs::{read_dir, File},
    io,
    path::{Path, PathBuf},
    process::ExitCode,
};
use tiny_http::{Response, Server};
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        // trim whitespaces from the left
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        // trim whitespaces from the left
        self.trim_left();
        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|c| c.is_numeric()));
        }

        if self.content[0].is_alphabetic() {
            return Some(self.chop_while(|c| c.is_alphanumeric()));
        }

        return Some(self.chop(1));
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let file = File::open(file_path)?;
    let event_reader = EventReader::new(file);

    let mut content = String::new();
    for event in event_reader.into_iter() {
        if let XmlEvent::Characters(text) = event.expect("TODO") {
            content.push_str(&text);
            content.push_str(" ");
        }
    }
    Ok(content)
}

fn save_tf_index(tf_index: &TermFreqIndex, index_path: &str) -> Result<(), ()> {
    println!("Saving {index_path}...");
    let index_file = File::create(index_path).unwrap();

    let json = serde_json::to_string(&tf_index).unwrap();

    serde_json::to_writer_pretty(index_file, &json).unwrap();

    for (path, tf) in tf_index {
        println!("{path:?} has {count} unique tokens", count = tf.len())
    }

    Ok(())
}

fn check_index(index_path: &str) -> Result<(), ()> {
    let index_file = File::open(index_path).unwrap();
    println!("Reading {index_path} index file...");

    let tf_index: String = serde_json::from_reader(index_file).unwrap();
    println!(
        "{index_path} contains {count} files",
        count = tf_index.len()
    );
    Ok(())
}

fn tf_index_of_folder(dir_path: &str, tf_index: &mut TermFreqIndex) -> Result<(), ()> {
    let dir = read_dir(dir_path).unwrap();

    for file in dir {
        let file_path = file.unwrap().path();

        println!("Indexing {file_path:?}...");

        let content = read_entire_xml_file(&file_path)
            .unwrap()
            .chars()
            .collect::<Vec<_>>();

        let mut table_freq = TermFreq::new();

        for token in Lexer::new(&content) {
            let term = token
                .iter()
                .map(|c| c.to_ascii_uppercase())
                .collect::<String>();

            if let Some(freq) = table_freq.get_mut(&term) {
                *freq += 1;
            } else {
                table_freq.insert(term, 1);
            };
        }

        let mut stats = table_freq.iter().collect::<Vec<_>>();
        stats.sort_by_key(|f| f.1);
        stats.reverse();

        tf_index.insert(file_path, table_freq);
    }

    Ok(())
}

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

fn usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("    index <folder> [address]            index the <folder> and seva the index to index.json file");
    eprintln!(
        "    search <index-file> [address]       check how many documents are indexed in the file"
    );
    eprintln!(
        "    serve [address]                      start local HTTP server with Web Interface "
    );
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        eprintln!("ERROR: no subcommand is provided");
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("ERROR: no directory provided for {subcommand} subcommand");
            })?;

            let mut tf_index = TermFreqIndex::new();
            tf_index_of_folder(&dir_path, &mut tf_index)?;
            save_tf_index(&tf_index, "index.json")?;
        }
        "search" => {
            let index_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("ERROR: no path to index is provided for {subcommand} subcommand");
            })?;

            check_index(&index_path)?;
        }
        "serve" => {
            let address = args.next().unwrap_or("127.0.0.1:6969".to_owned());
            let server = Server::http(&address).unwrap();

            println!("INFO: listening at http://{address}/");

            for request in server.incoming_requests() {
                println!(
                    "received request! method: {:?}, url: {:?}, headers: {:?}",
                    request.method(),
                    request.url(),
                    request.headers()
                );

                let response = Response::from_string("hello world");
                request.respond(response).unwrap();
            }
            todo!("not yet implemented")
        }

        _ => {
            usage(&program);
            eprintln!("ERROR: unknown subcommand {subcommand}");
            return Err(());
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
