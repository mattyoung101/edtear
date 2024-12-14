use couch_rs::document::TypedCouchDocument;
use couch_rs::{types::document::DocumentId, CouchDocument};
use elite_journal::entry::{market::Commodity, Market};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CouchDocument, Default, Debug)]
pub struct MarketDocument {
    /// _ids are are the only unique enforced value within `CouchDB` so you might as well make use of this.
    /// `CouchDB` stores its documents in a B+ tree. Each additional or updated document is stored as
    /// a leaf node, and may require re-writing intermediary and parent nodes. You may be able to take
    /// advantage of sequencing your own ids more effectively than the automatically generated ids if
    /// you can arrange them to be sequential yourself. <https://docs.couchdb.org/en/stable/best-practices/documents.html>
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _id: DocumentId,
    /// Document Revision, provided by `CouchDB`, helps negotiating conflicts
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _rev: String,

    pub market_id: i64,
    pub system_name: String,
    pub station_name: String,
    pub commodities: Vec<Commodity>,
}

impl MarketDocument {
    pub fn new(entry: elite_journal::Entry<Market>) -> Self {
        return MarketDocument {
            _id: entry.event.market_id.to_string(),
            _rev: String::new(),
            market_id: entry.event.market_id,
            system_name: entry.event.system_name,
            station_name: entry.event.station_name,
            commodities: entry.event.commodities,
        };
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Connecting to CouchDB");
    let client = couch_rs::Client::new_local_test().unwrap();
    let db = client.db("edtear").await.unwrap();

    for envelope in eddn::subscribe(eddn::URL) {
        if envelope.is_err() {
            continue;
        }
        match envelope.ok().unwrap().message {
            eddn::Message::Commodity(commodity) => {
                info!("Received market update");

                let mut doc = MarketDocument::new(commodity);

                match db.save(&mut doc).await {
                    Err(error) => {
                        error!("Failed to insert into CouchDB: {}", error);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
