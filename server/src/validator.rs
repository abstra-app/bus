use std::collections::HashMap as Map;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::policy::{BroadcastStmt, MsgStmt, RequestStmt, ResponseStmt, Statement};

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
    channel: String,
}
#[derive(Debug, Deserialize, Clone)]
struct ResponseMessage {
    payload: Map<String, ParamType>,
    channel: String,
}

#[derive(Debug, Deserialize, Clone)]
struct BroadcastMessage {
    payload: Map<String, ParamType>,
    channel: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ListenMessage {
    channel: String,
}

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
                let mut channel: Option<String> = None; // Add this line

                while let Some(key) = map.next_key()? {
                    match key {
                        "type" => {
                            message_type = Some(map.next_value()?);
                        }
                        "payload" => {
                            payload = Some(map.next_value()?);
                        }
                        "channel" => {
                            // Add this block
                            channel = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }

                let message_type = message_type.ok_or_else(|| de::Error::missing_field("type"))?;
                let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;
                let channel = channel.ok_or_else(|| de::Error::missing_field("channel"))?;
                match message_type.as_str() {
                    "request" => Ok(Message::Request(RequestMessage { payload, channel })),
                    "response" => Ok(Message::Response(ResponseMessage { payload, channel })),
                    "broadcast" => Ok(Message::Broadcast(BroadcastMessage { payload, channel })),
                    "listen" => Ok(Message::Listen(ListenMessage { channel })),
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


#[derive(Debug)]
pub struct InvalidChannelError;
impl Error for InvalidChannelError {}
impl Display for InvalidChannelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Invalid channel")
    }
}

#[derive(Debug)]
pub struct InvalidMessageTypeError {
    given_type: String
}
impl Error for InvalidMessageTypeError {}
impl Display for InvalidMessageTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Invalid message type")
    }
}

#[derive(Debug)]
pub enum MessageError {
    InvalidChannelError(InvalidChannelError),
    InvalidMessageTypeError(InvalidMessageTypeError),
    InvalidMessageError(Message)
}


fn validate_request<'a>(stmt: &'a RequestStmt, message: &'a RequestMessage) -> Option<MessageError> {
    println!("stmt.msg_name: {}, message.channel: {}", stmt.msg_name, message.channel);
    println!("stmt.msg_name == message.channel: {}", stmt.msg_name == message.channel);
    if stmt.msg_name == message.channel {
        None
    } else {
        Some(MessageError::InvalidChannelError(InvalidChannelError))
    }
}

fn validate_response<'a>(stmt: &'a ResponseStmt, message: &'a ResponseMessage) -> Option<MessageError> {
    if stmt.msg_name == message.channel {
        None
    } else {
        Some(MessageError::InvalidChannelError(InvalidChannelError))
    }
}

fn validate_broadcast<'a>(stmt: &'a BroadcastStmt, message: &'a BroadcastMessage) -> Option<MessageError> {
    if stmt.msg_name == message.channel {
        None
    } else {
        Some(MessageError::InvalidChannelError(InvalidChannelError))
    }
}

fn validate_message(policy: Vec<Statement>, message: &Message) -> Option<MessageError> {
    let mut matched = false;

    for stmt in &policy {
        match (stmt, message) {
            (
                Statement::Msg(MsgStmt::Request(ref req_stmt)),
                &Message::Request(ref request_message),
            ) => {
                if let Some(err) = validate_request(req_stmt, request_message) {
                    return Some(err);
                } else {
                    matched = true;
                }
            }

            (
                Statement::Msg(MsgStmt::Response(ref res_stmt)),
                &Message::Response(ref response_message),
            ) => {
                if let Some(err) = validate_response(res_stmt, response_message) {
                    return Some(err);
                } else {
                    matched = true;
                }
            }

            (
                Statement::Msg(MsgStmt::Broadcast(ref broadcast_stmt)),
                &Message::Broadcast(ref broadcast_message),
            ) => {
                if let Some(err) = validate_broadcast(broadcast_stmt, broadcast_message) {
                    return Some(err);
                } else {
                    matched = true;
                }
            }
            (_, &Message::Listen(_)) => (),
            _ => {}
        }

        if matched {
            break;
        }
    }

    if matched {
        None
    } else {
        Some(MessageError::InvalidMessageError(message.clone()))
    }
}

pub fn message_from_str(
    policy: Vec<Statement>,
    message: &str,
) -> Result<Message, MessageError> {
    let message: Message = serde_json::from_str(message).unwrap();
    match validate_message(policy, &message) {
        Some(err) => Err(err),
        None => Ok(message),
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::{Statement, MsgStmt, RequestStmt};

    #[test]
    fn test_validate_request() {
        let stmt = RequestStmt {
            msg_name: "test".to_string(),
            msg_params: vec![],
        };
        let message = r#"{
            "type": "request",
            "payload": {},
            "channel": "test"
        }"#;
        let message = serde_json::from_str::<crate::validator::Message>(message).unwrap();
        let req_message = match message {
            crate::validator::Message::Request(req) => req,
            _ => panic!("Invalid message type")
        };
        let result = crate::validator::validate_request(&stmt, &req_message);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_message() {
        let policy = vec![
            Statement::Msg(MsgStmt::Request(RequestStmt {
                msg_name: "test".to_string(),
                msg_params: vec![],
            }))
        ];
        let message = r#"{
            "type": "request",
            "payload": {},
            "channel": "test"
        }"#;
        let message = serde_json::from_str::<crate::validator::Message>(message).unwrap();
        let result = crate::validator::validate_message(policy, &message);
        assert!(result.is_none());
    }
}