use serde::{Deserialize, Serialize};

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
