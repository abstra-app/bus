use std::collections::HashMap as Map;

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::policy::Statement;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ParamType {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

#[derive(Debug, Deserialize, Clone)]
struct RequestMessage {
    payload: Map<String, ParamType>,
}
#[derive(Debug, Deserialize, Clone)]
struct ResponseMessage {
    payload: Map<String, ParamType>,
}

#[derive(Debug, Deserialize, Clone)]
struct BroadcastMessage {
    payload: Map<String, ParamType>,
}

#[derive(Debug, Deserialize, Clone)]
struct ListenMessage {}

#[derive(Debug, Clone)]
pub enum Message {
    Request(RequestMessage),
    Response(ResponseMessage),
    Broadcast(BroadcastMessage),
    Listen(ListenMessage),
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageVisitor;

        impl<'de> Visitor<'de> for MessageVisitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a message with a `type` field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut message_type: Option<String> = None;
                let mut payload: Option<Map<String, ParamType>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "type" => {
                            message_type = Some(map.next_value()?);
                        }
                        "payload" => {
                            payload = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }

                let message_type = message_type.ok_or_else(|| de::Error::missing_field("type"))?;
                let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;
                match message_type.as_str() {
                    "request" => Ok(Message::Request(RequestMessage { payload })),
                    "response" => Ok(Message::Response(ResponseMessage { payload })),
                    "broadcast" => Ok(Message::Broadcast(BroadcastMessage { payload })),
                    "listen" => Ok(Message::Listen(ListenMessage {})),
                    _ => Err(de::Error::unknown_variant(
                        &message_type,
                        &["request", "response", "broadcast", "listen"],
                    )),
                }
            }
        }

        deserializer.deserialize_map(MessageVisitor)
    }
}

fn validate(policy: Vec<Statement>, message: Message) {
    println!("policy: {:#?}", policy);
    println!("message: {:#?}", message);
}

pub fn message_from_str(policy: Vec<Statement>, message: &str) -> Result<Message, serde_json::Error> {
    let message: Message = serde_json::from_str(message).unwrap();
    validate(policy, message.clone());
    Ok(message)
}
