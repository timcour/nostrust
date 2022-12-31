use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};

// {
//     "id": <32-bytes sha256 of the the serialized event data>
//     "pubkey": <32-bytes hex-encoded public key of the event creator>,
//     "created_at": <unix timestamp in seconds>,
//     "kind": <integer>,
//     "tags": [
//         ["e", <32-bytes hex of the id of another event>, <recommended relay URL>],
//         ["p", <32-bytes hex of the key>, <recommended relay URL>],
//         ... // other kinds of tags may be included later
//     ],
//     "content": <arbitrary string>,
//     "sig": <64-bytes signature of the sha256 hash of the serialized event data, which is the same as the "id" field>
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: i32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        derive_event_id(self);
        write!(f, r#"Event
  id: {}
  content: {}
  pubkey: {}
  sig: {}
  kind: {}
"#, self.id, self.content, self.pubkey, self.sig, self.kind)
    }
}

// [
//     0,
//     <pubkey, as a (lowercase) hex string>,
//     <created_at, as a number>,
//     <kind, as a number>,
//     <tags, as an array of arrays of non-null strings>,
//     <content, as a string>
// ]
pub fn derive_event_id(event: &Event) {
    let parts = json!([
        0,
        event.pubkey,
        event.created_at,
        event.kind,
        event.tags,
        event.content
    ]);

    let serialized = serde_json::to_string(&parts).expect("Serialization failed");
    let bytes = serialized.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    println!("Computed ID: {:X?}", &result);
}
