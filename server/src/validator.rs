use std::collections::HashMap as Map;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fmt;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::policy::{BroadcastStmt, MsgStmt, RequestStmt, ResponseStmt, Statement, MsgParam};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ParamType {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl Serialize for ParamType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ParamType::String(s) => serializer.serialize_str(s),
            ParamType::Int(i) => serializer.serialize_i32(*i),
            ParamType::Float(f) => serializer.serialize_f32(*f),
            ParamType::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

type Params = Map<String, ParamType>;

#[derive(Debug, Deserialize, Clone)]
pub struct RequestMessage {
    pub payload: Params,
    pub channel: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ResponseMessage {
    pub payload: Params,
    pub channel: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BroadcastMessage {
    pub payload: Params,
    pub channel: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListenMessage {
    pub channel: String,
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
                let mut payload: Option<Params> = None;
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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Request(request) => write!(f, "Request: {:?}", request),
            Message::Response(response) => write!(f, "Response: {:?}", response),
            Message::Broadcast(event) => write!(f, "Broadcast: {:?}", event),
            Message::Listen(event) => write!(f, "Listen: {:?}", event),
        }
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
    InvalidMessageError(Message),
    InvalidParameterError(Params),
}

fn validate_parameters(stmt_params: &Vec<MsgParam>, message_params: &Params) -> Option<MessageError> {
    for stmt_param in stmt_params {
        match message_params.get(&stmt_param.param_name) {
            Some(value) => {
                match (stmt_param.param_type.as_str(), value) {
                    ("string", ParamType::String(_)) => {}
                    ("int", ParamType::Int(_)) => {}
                    ("float", ParamType::Float(_)) => {}
                    ("bool", ParamType::Bool(_)) => {}
                    _ => {
                        return Some(MessageError::InvalidParameterError(message_params.clone()));
                    }
                }
            }
            None => {
                return Some(MessageError::InvalidParameterError(message_params.clone()));
            }
        }
    }

    for key in message_params.keys() {
        if stmt_params.iter().find(|&x| x.param_name == *key).is_none() {
            return Some(MessageError::InvalidParameterError(message_params.clone()));
        }
    }

    None
}

fn validate_request<'a>(stmt: &'a RequestStmt, message: &'a RequestMessage) -> Option<MessageError> {
    println!("stmt.msg_name: {}, message.channel: {}", stmt.msg_name, message.channel);
    println!("stmt.msg_name == message.channel: {}", stmt.msg_name == message.channel);
    if stmt.msg_name == message.channel {
        return validate_parameters(&stmt.msg_params, &message.payload);
    } else {
        Some(MessageError::InvalidChannelError(InvalidChannelError))
    }
}

fn validate_response<'a>(stmt: &'a ResponseStmt, message: &'a ResponseMessage) -> Option<MessageError> {
    if stmt.msg_name == message.channel {
        return validate_parameters(&stmt.msg_params, &message.payload);
    } else {
        Some(MessageError::InvalidChannelError(InvalidChannelError))
    }
}

fn validate_broadcast<'a>(stmt: &'a BroadcastStmt, message: &'a BroadcastMessage) -> Option<MessageError> {
    if stmt.msg_name == message.channel {
        return validate_parameters(&stmt.msg_params, &message.payload);
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

    fn validate_request_io(stmt: RequestStmt, message: &str, expected_error: bool) {
        let message = serde_json::from_str::<crate::validator::Message>(message).unwrap();
        let req_message = match message {
            crate::validator::Message::Request(req) => req,
            _ => panic!("Invalid message type")
        };
        let result = crate::validator::validate_request(&stmt, &req_message);
        assert_eq!(!expected_error, result.is_none());
    }

    #[test]
    fn test_validate_request_valid() {
        validate_request_io(
            RequestStmt {
                msg_name: "test".to_string(),
                msg_params: vec![
                    crate::policy::MsgParam {
                        param_name: "a".to_string(),
                        param_type: "string".to_string(),
                    },
                    crate::policy::MsgParam {
                        param_name: "b".to_string(),
                        param_type: "int".to_string(),
                    }
                ],
            }, 
            r#"{
                "type": "request",
                "payload": {
                    "a": "1",
                    "b": 2
                },
                "channel": "test"
            }"#,
            false
        );
    }

    #[test]
    fn test_validate_request_missing_message_param() {
        validate_request_io(
            RequestStmt {
                msg_name: "test".to_string(),
                msg_params: vec![
                    crate::policy::MsgParam {
                        param_name: "a".to_string(),
                        param_type: "string".to_string(),
                    },
                    crate::policy::MsgParam {
                        param_name: "b".to_string(),
                        param_type: "int".to_string(),
                    }
                ],
            }, 
            r#"{
                "type": "request",
                "payload": {
                    "a": "1"
                },
                "channel": "test"
            }"#,
            true
        );
    }

    #[test]
    fn test_validate_request_missing_policy_param() {
        validate_request_io(
            RequestStmt {
                msg_name: "test".to_string(),
                msg_params: vec![
                    crate::policy::MsgParam {
                        param_name: "a".to_string(),
                        param_type: "string".to_string(),
                    },
                ],
            }, 
            r#"{
                "type": "request",
                "payload": {
                    "a": "1",
                    "b": 2
                },
                "channel": "test"
            }"#,
            true
        );
    }

    #[test]
    fn test_validate_request_wrong_parameter_type() {
        validate_request_io(
            RequestStmt {
                msg_name: "test".to_string(),
                msg_params: vec![
                    crate::policy::MsgParam {
                        param_name: "a".to_string(),
                        param_type: "string".to_string(),
                    },
                    crate::policy::MsgParam {
                        param_name: "b".to_string(),
                        param_type: "int".to_string(),
                    }
                ],
            }, 
            r#"{
                "type": "request",
                "payload": {
                    "a": "1",
                    "b": "2"
                },
                "channel": "test"
            }"#,
            true
        );
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