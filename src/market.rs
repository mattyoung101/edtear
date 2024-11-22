// Upstream source: https://github.com/nixpulvis/elite_journal/blob/master/src/entry/market.rs

use couch_rs::{types::document::DocumentId, CouchDocument};
use couch_rs::document::TypedCouchDocument;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, CouchDocument)]
pub struct Market {
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

    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    pub commodities: Vec<Commodity>,

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Commodity {
    pub name: String,
    pub mean_price: i32,
    pub buy_price: i32,
    pub sell_price: i32,
    pub demand: i32,
    pub demand_bracket: i32,
    pub stock: i32,
    pub stock_bracket: i32,
}

