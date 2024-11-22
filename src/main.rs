use log::{error, info};
use market::Market;
mod eddn;
mod market;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Connecting to CouchDB");

    let client = couch_rs::Client::new_local_test().unwrap();
    let db = client.db("edtear").await.unwrap();

    info!("Connecting to EDDN");
    for envelope in eddn::subscribe(eddn::URL) {
        if envelope.is_err() {
            error!("Error receiving message");
            continue;
        }

        dbg!(envelope);

        // match envelope.ok().unwrap().message {
        //     eddn::Message::Commodity() => {
        //         info!("Received commodity");
        //     }
        //     _ => {}
        // }
    }
}
