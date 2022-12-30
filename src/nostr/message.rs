use std::str::FromStr;

use eyre::{eyre, Result};
use serde_json::{self, Value};

use super::event::Event;

#[derive(Debug)]
pub struct MessageUnknown {
    pub data: Value
}

// ["EVENT", <event JSON as defined above>], used to publish events.
// ["REQ", <subscription_id>, <filters JSON>...], used to request events and subscribe to new updates.
// ["CLOSE", <subscription_id>], used to stop previous subscriptions.
#[derive(Debug)]
pub enum ClientMessage {
    Unknown(MessageUnknown),
    Event(ClientMessageEvent),
    Req(ClientMessageReq),
    Close(ClientMessageClose)
}

#[derive(Debug)]
pub struct ClientMessageEvent {
    pub event: Event
}

#[derive(Debug)]
pub struct ClientMessageReq {
    pub subscription_id: String
}

#[derive(Debug)]
pub struct ClientMessageClose {
    pub subscription_id: String
}


// ["EVENT", <subscription_id>, <event JSON as defined above>], used to send events requested by clients.
// ["NOTICE", <message>], used to send human-readable error messages or other things to clients.
#[derive(Debug)]
pub enum RelayMessage {
    Unknown(MessageUnknown),
    Event(RelayMessageEvent),
    Notice(RelayMessageNotice),
}

#[derive(Debug)]
pub struct RelayMessageEvent {
    pub subscription_id: String,
    pub event: Event
}

#[derive(Debug)]
pub struct RelayMessageNotice {
    pub message: String,
}

pub fn parse_unstructured_message(raw: &str) -> Result<(String, Value)> {
    let result = serde_json::from_str(&raw);

    if result.is_err() {
        return Err(eyre!("Failed to parse message JSON: {:?}:", result.err()))
    }

    let value: Value = result.unwrap();
    let first = &value[0];
    let opt_type = first.as_str();
    if opt_type.is_none() {
        return Err(eyre!("Malformed Nostr message: `type` not found as first element."));
    }

    let msg_type = opt_type.unwrap();

    // TODO: why does the borrow checker not like returning a &str here
    // directly.  It was complaining about referencing the element in `value`,
    // event w/ a clone().
    let type_clone_result = String::from_str(msg_type);
    if type_clone_result.is_err() {
        return Err(eyre!("Failed to clone message type: {:?}:", type_clone_result.err()));
    }

    Ok((type_clone_result.unwrap(), value))
}

pub fn parse_relay_message(raw: &str) -> Result<RelayMessage> {
    let result = parse_unstructured_message(&raw);
    if result.is_err() {
        return Err(eyre!("{:?}", result.err()));
    }

    let (kind, value) = result.unwrap();

    let message: RelayMessage = match kind.as_str() {
        "EVENT" => RelayMessage::Event(RelayMessageEvent {
            subscription_id: value[1].clone().to_string(),
            event: serde_json::from_value(value[2].clone())?
        }),
        "NOTICE" => RelayMessage::Notice(RelayMessageNotice {
            message: value[1].clone().to_string(),
        }),
        _ => RelayMessage::Unknown(MessageUnknown {
            data: value.clone(),
        })
    };

    Ok(message)
}

#[allow(dead_code)]
pub fn parse_client_message(raw: &str) -> Result<ClientMessage> {
    let result = parse_unstructured_message(&raw);
    if result.is_err() {
        return Err(eyre!("{:?}", result.err()));
    }

    let (kind, value) = result.unwrap();
    let message: ClientMessage = match kind.as_str() {
        "EVENT" => ClientMessage::Event(ClientMessageEvent {
            event: serde_json::from_value(value[1].clone())?
        }),
        "REQ" => ClientMessage::Req(ClientMessageReq {
            subscription_id: value[1].clone().to_string(),
        }),
        "CLOSE" => ClientMessage::Close(ClientMessageClose{
            subscription_id: value[1].clone().to_string(),
        }),
        _ => ClientMessage::Unknown(MessageUnknown {
            data: value.clone(),
        })
    };

    Ok(message)
}
