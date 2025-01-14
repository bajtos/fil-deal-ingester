use json_event_parser::{JsonEvent, JsonReader, JsonWriter};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    env_logger::init();

    let infile = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("Missing required argument: path to StorageMarketDeals.json.zst"),
    };

    assert!(
        infile.ends_with(".json.zst"),
        "The StorageMarketDeals file must have .json.zst extension"
    );
    let f = File::open(infile).expect("cannot open input file");
    let decoder =
        zstd::stream::Decoder::new(BufReader::new(f)).expect("cannot create zstd decoder");
    let mut reader = JsonReader::from_reader(BufReader::new(decoder));

    let mut buffer = Vec::new();

    assert_eq!(
        reader.read_event(&mut buffer).expect("cannot parse JSON"),
        JsonEvent::StartObject,
    );

    loop {
        let event = reader.read_event(&mut buffer).expect("cannot parse JSON");
        log::debug!("{:?}", event);

        match event {
            JsonEvent::ObjectKey(_) => parse_deal(&mut reader, &mut buffer),
            JsonEvent::EndObject => {
                let event = reader.read_event(&mut buffer).expect("cannot parse JSON");
                if event == JsonEvent::Eof {
                    break;
                } else {
                    panic!("unexpected JSON event after EndObject: {event:?}")
                }
            }
            _ => panic!("unexpected JSON event: {event:?}"),
        };
    }
}

fn parse_deal<R: BufRead>(reader: &mut JsonReader<R>, buffer: &mut Vec<u8>) {
    let mut output = Vec::new();
    let mut writer = JsonWriter::from_writer(&mut output);

    let mut depth = 0;

    loop {
        let event = reader.read_event(buffer).expect("cannot parse JSON");
        if depth == 0 {
            assert_eq!(event, JsonEvent::StartObject,);
            log::debug!("==DEAL START==");
        }

        writer.write_event(event).expect("cannot write JSON");

        match event {
            JsonEvent::StartObject => {
                depth += 1;
            }
            JsonEvent::EndObject => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        log::debug!("[{depth}] {event:?}");
    }

    log::debug!("==DEAL END==");

    output.push(b'\n');
    let _ = std::io::stdout().write(&output);
    let _ = std::io::stdout().flush();
    // println!("{}", std::str::from_utf8(&output).expect("malformed UTF-8"));
}
