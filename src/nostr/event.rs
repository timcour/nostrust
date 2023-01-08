use eyre::{eyre, Result, Error};

use secp256k1::{rand, SecretKey};
// TODO: remove allows when introduced to main app
#[allow(unused_imports)]
use secp256k1::rand::rngs::OsRng;
#[allow(unused_imports)]
use secp256k1::hashes::sha256;
#[allow(unused_imports)]
use secp256k1::{Secp256k1, Message, XOnlyPublicKey, KeyPair};

use secp256k1::schnorr::Signature;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Sha256, Digest, digest::generic_array::GenericArray, digest::typenum::U32};
use std::{fmt, num::ParseIntError};

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
pub fn derive_event_id(event: &Event) -> GenericArray<u8, U32> {
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
    hasher.finalize()
}

#[allow(dead_code)]
fn hash_to_hex_string(bytes: &GenericArray<u8, U32>) -> String {
     format!("{:x}", &bytes)
}

#[allow(dead_code)]
fn factory_build_event() -> Event {
    Event {
        id: String::from("da7d89bc06080d60ae537ff0285b51f7a5e15e63eb3c21a0c37c76edbbe24255"),
        pubkey: String::from("b708f7392f588406212c3882e7b3bc0d9b08d62f95fa170d099127ece2770e5e"),
        created_at: 1672310253,
        kind: 1,
        tags: [].to_vec(),
        content: String::from("imagine all the unfettered conversations 😯"),
        sig: String::from("d706fb48dbdd4fe272a006ee7f9fe74416a603cdfbb253dd82f1dc6bcea3cfe79334abb034701747941819878b31b28753a6dd38c4cda9c82453bf676ea2ba38")
    }
}

#[test]
fn verify_event_id() {
    let event: Event = factory_build_event();
    let result = derive_event_id(&event);
    assert_eq!(hash_to_hex_string(&result), event.id);
}

#[allow(dead_code)]
fn _decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn sig_verify(event: &Event) -> Result<()> {
    let hash = derive_event_id(&event);
    let msg = Message::from_slice(&hash)?;
    // println!("pubkey: {:?}", &event.pubkey[..]);
    let pubkey_bytes = _decode_hex(&event.pubkey)?;
    // println!("pubkey_bytes: {:?}", pubkey_bytes);
    let pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)?;
    // println!("sig: {:?}", &event.sig);
    let sig_bytes = _decode_hex(&event.sig)?;
    // println!("sig_bytes: {:?}", sig_bytes);
    let sig = Signature::from_slice(&sig_bytes)?;
    sig.verify(&msg, &pubkey)?;
    Ok(())
}

pub fn parse_event(raw: Value) -> Result<Event, Error> {
    let parse_result = serde_json::from_value::<Event>(raw);
    let event = match parse_result {
        Ok(event) => Ok(event),
        Err(e) => Err(eyre!("Could not parse event: {:?}", e))
    }?;

    let verification_result = sig_verify(&event);
    match verification_result {
        Ok(_) => Ok(()),
        Err(e) => Err(eyre!("Signature invalid: {:?}", e))
    }?;

    Ok(event)
}

#[test]
fn test_parse_event() {
    println!("test_parse_event");
    let event: Event = factory_build_event();
    let json_event = serde_json::to_value(event).expect("Evetn Serialization failed");
    let result = parse_event(json_event);
    assert!(result.is_ok());
}

#[test]
fn test_sig_verify() {
    println!("verify_event_sig");
    let event: Event = factory_build_event();
    let result = sig_verify(&event);
    println!("sig result: {:?}", result);
    assert!(result.is_ok());
}

pub fn gen_keypair() -> Result<(SecretKey, XOnlyPublicKey)> {
    let secp = Secp256k1::new();
    let key_pair = KeyPair::new(&secp, &mut rand::thread_rng());
    let public_key = XOnlyPublicKey::from_keypair(&key_pair);
    let secret_key = key_pair.secret_key();
    Ok((secret_key, public_key.0))
}

#[test]
fn test_gen_keys() {
    let (secret_key, public_key) = gen_keypair().expect("Failed to generate keypair");
    println!("\nsecret_key: {}", secret_key.display_secret());
    println!("public_key: {}", public_key);

    let message = Message::from_hashed_data::<sha256::Hash>("Hello World!".as_bytes());

    let secp = Secp256k1::new();
    let key_pair = KeyPair::new(&secp, &mut rand::thread_rng());
    // let public_key = XOnlyPublicKey::from_keypair(&key_pair);
    // let secret_key = key_pair.secret_key();
    let sig = secp.sign_schnorr(&message, &key_pair);
    println!("sig: {:?}", sig);

    assert!(sig.verify(&message, &XOnlyPublicKey::from_keypair(&key_pair).0).is_ok());
}
