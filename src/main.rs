use std::{fs::File, io::{Seek, Write}};

fn main() {
    let mut messages = Vec::new();
    let mut file = File::create("out.cbor").unwrap();

    for envelope in eddn::subscribe(eddn::URL) {
        if envelope.is_err() {
            continue;
        }

        match envelope.ok().unwrap().message {
            eddn::Message::Commodity(commodity) => {
                // dbg!(&commodity);
                messages.push((commodity.timestamp, commodity.event));

                // write to file (json)
                // let json = serde_json::to_string_pretty(&messages).unwrap();

                // write to file (CBOR)
                let cbor = serde_cbor::to_vec(&messages).unwrap();
                file.set_len(0).unwrap();
                file.seek(std::io::SeekFrom::Start(0)).unwrap();
                file.write(&cbor).unwrap();

                println!("Received message");
            },
            _ => {}
        }
    }
}
