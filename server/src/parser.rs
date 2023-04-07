use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum MsgType {
    Broadcast,
    Listen,
    Request,
    Response,
}

impl FromStr for MsgType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "broadcast" => Ok(MsgType::Broadcast),
            "listen" => Ok(MsgType::Listen),
            "request" => Ok(MsgType::Request),
            "response" => Ok(MsgType::Response),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MessageParams {
    param_name: String,
    param_type: String,
}

#[derive(Debug, PartialEq)]
pub enum MessageStmt {
    Broadcast(String, Vec<MessageParams>),
    Request(String, Vec<MessageParams>),
    Response(String, Vec<MessageParams>),
}

#[derive(Debug, PartialEq)]
pub enum AllowStmt {
    Allow(String, MsgType, String, Option<String>),
}

#[derive(Debug, PartialEq)]
pub enum RoleStmt {
    Role(String, Option<String>),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Allow(AllowStmt),
    Msg(MessageStmt),
    Role(RoleStmt),
}

type Body = Vec<Statement>;

fn parse_allow_stmt(input: &str) -> Option<AllowStmt> {
    // implement the parsing logic for allow_stmt
    return None;
}

fn parse_msg_type(input: &str) -> Option<MsgType> {
    // implement the parsing logic for msg_type
    return None;
}

fn parse_msg_params(input: &str) -> Option<MessageParams> {
    // implement the parsing logic for msg_params
    return None;
}

fn parse_msg_stmt(input: &str) -> Option<MessageStmt> {
    // implement the parsing logic for msg_stmt
    return None;
}

fn parse_role_stmt(input: &str) -> Option<RoleStmt> {
    // implement the parsing logic for role_stmt
    return None;
}

fn parse_statement(input: &str) -> Option<Statement> {
    if let Some(allow_stmt) = parse_allow_stmt(input) {
        Some(Statement::Allow(allow_stmt))
    } else if let Some(msg_stmt) = parse_msg_stmt(input) {
        Some(Statement::Msg(msg_stmt))
    } else if let Some(role_stmt) = parse_role_stmt(input) {
        Some(Statement::Role(role_stmt))
    } else {
        None
    }
}

pub fn parse_body(input: &str) -> Body {
    let mut body = Vec::new();

    for line in input.lines() {
        if let Some(statement) = parse_statement(line) {
            body.push(statement);
        }
    }

    body
}