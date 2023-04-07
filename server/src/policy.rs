#[derive(Debug, Clone)]
pub enum Statement {
    Allow(AllowStmt),
    Msg(MsgStmt),
    Role(RoleStmt),
}

#[derive(Debug, Clone)]
struct AllowStmt {
    role_name: String,
    msg_type: MsgType,
    msg_name: String,
    filter_exp: Option<String>,
}

#[derive(Debug, Clone)]
enum MsgType {
    Broadcast,
    Listen,
    Request,
    Response,
}

#[derive(Debug, Clone)]
enum MsgStmt {
    Broadcast(BroadcastStmt),
    Request(RequestStmt),
    Response(ResponseStmt),
}

#[derive(Debug, Clone)]
struct BroadcastStmt {
    msg_name: String,
    msg_params: Vec<MsgParam>,
}

#[derive(Debug, Clone)]
struct RequestStmt {
    msg_name: String,
    msg_params: Vec<MsgParam>,
}

#[derive(Debug, Clone)]
struct ResponseStmt {
    msg_name: String,
    msg_params: Vec<MsgParam>,
}

#[derive(Debug, Clone)]
struct MsgParam {
    param_name: String,
    param_type: String,
}

#[derive(Debug, Clone)]
struct RoleStmt {
    role_name: String,
    extends_role: Option<String>,
}

// lexer
#[derive(Debug, PartialEq)]
enum Token {
    Allow,
    Broadcast,
    Listen,
    Request,
    Response,
    Role,
    Extends,
    When,
    LBrace,
    RBrace,
    Colon,
    Identifier(String),
    Whitespace,
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        let token = match c {
            'a'..='z' | 'A'..='Z' | '_' | '-' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        identifier.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                recognize_reserved_word(identifier)
            }
            ' ' | '\t' | '\r' | '\n' => Token::Whitespace,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ':' => Token::Colon,
            _ => panic!("Unexpected character: {}", c),
        };
        tokens.push(token);
    }
    tokens
}

fn recognize_reserved_word(identifier: String) -> Token {
    match identifier.as_str() {
        "allow" => Token::Allow,
        "broadcast" => Token::Broadcast,
        "listen" => Token::Listen,
        "request" => Token::Request,
        "response" => Token::Response,
        "role" => Token::Role,
        "extends" => Token::Extends,
        "when" => Token::When,
        _ => Token::Identifier(identifier),
    }
}

//parser
struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, index: 0 }
    }

    fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while self.index < self.tokens.len() {
            let statement = self.parse_statement();
            statements.push(statement);
            self.skip_whitespace();
        }
        statements
    }

    fn parse_statement(&mut self) -> Statement {
        self.skip_whitespace();
        match &self.tokens[self.index] {
            Token::Allow => Statement::Allow(self.parse_allow_stmt()),
            Token::Broadcast | Token::Request | Token::Response => {
                Statement::Msg(self.parse_msg_stmt())
            }
            Token::Role => Statement::Role(self.parse_role_stmt()),
            _ => panic!("Unexpected token {:?}", self.tokens[self.index]),
        }
    }

    fn parse_role_stmt(&mut self) -> RoleStmt {
        self.expect(Token::Role);
        self.skip_whitespace();

        let role_name = self.parse_role_name();
        self.skip_whitespace();

        let extends_role = if self.maybe_expect(Token::Extends) {
            self.skip_whitespace();
            let extends_role_name = self.parse_role_name();
            self.skip_whitespace();
            Some(extends_role_name)
        } else {
            None
        };

        RoleStmt {
            role_name,
            extends_role,
        }
    }

    fn parse_msg_stmt(&mut self) -> MsgStmt {
        self.skip_whitespace();
        match &self.tokens[self.index] {
            Token::Broadcast => MsgStmt::Broadcast(self.parse_broadcast_stmt()),
            Token::Request => MsgStmt::Request(self.parse_request_stmt()),
            Token::Response => MsgStmt::Response(self.parse_response_stmt()),
            _ => panic!("Expected message statement"),
        }
    }

    fn parse_broadcast_stmt(&mut self) -> BroadcastStmt {
        self.expect(Token::Broadcast);
        self.skip_whitespace();

        let msg_name = self.parse_msg_name();
        self.skip_whitespace();

        let msg_params = self.parse_msg_params();

        BroadcastStmt { msg_name, msg_params }
    }

    fn parse_request_stmt(&mut self) -> RequestStmt {
        self.expect(Token::Request);
        self.skip_whitespace();

        let msg_name = self.parse_msg_name();
        self.skip_whitespace();

        let msg_params = self.parse_msg_params();

        RequestStmt { msg_name, msg_params }
    }

    fn parse_response_stmt(&mut self) -> ResponseStmt {
        self.expect(Token::Response);
        self.skip_whitespace();

        let msg_name = self.parse_msg_name();
        self.skip_whitespace();

        let msg_params = self.parse_msg_params();

        ResponseStmt { msg_name, msg_params }
    }

    fn parse_msg_params(&mut self) -> Vec<MsgParam> {
        self.expect(Token::LBrace);
        self.skip_whitespace();

        let mut msg_params = Vec::new();
        while self.tokens[self.index] != Token::RBrace {
            let msg_param = self.parse_msg_param();
            self.skip_whitespace();
            msg_params.push(msg_param);
        }

        self.expect(Token::RBrace);

        msg_params
    }

    fn parse_msg_param(&mut self) -> MsgParam {
        let param_name = self.parse_param_name();
        self.skip_whitespace();

        self.expect(Token::Colon);
        self.skip_whitespace();

        let param_type = self.parse_param_type();

        MsgParam {
            param_name,
            param_type,
        }
    }

    fn parse_param_name(&mut self) -> String {
        if let Token::Identifier(name) = &self.tokens[self.index] {
            self.index += 1;
            name.clone()
        } else {
            panic!("Expected parameter name");
        }
    }

    fn parse_param_type(&mut self) -> String {
        if let Token::Identifier(ty) = &self.tokens[self.index] {
            self.index += 1;
            ty.clone()
        } else {
            panic!("Expected parameter type");
        }
    }

    fn parse_allow_stmt(&mut self) -> AllowStmt {
        self.expect(Token::Allow);
        self.skip_whitespace();

        let role_name = self.parse_role_name();
        self.skip_whitespace();

        let msg_type = self.parse_msg_type();
        self.skip_whitespace();

        let msg_name = self.parse_msg_name();
        self.skip_whitespace();

        let filter_exp = if self.maybe_expect(Token::When) {
            self.skip_whitespace();

            let filter_exp = self.parse_filter_exp();
            self.skip_whitespace();

            Some(filter_exp)
        } else {
            None
        };

        AllowStmt {
            role_name,
            msg_type,
            msg_name,
            filter_exp,
        }
    }

    fn expect(&mut self, expected: Token) {
        if self.index >= self.tokens.len() || self.tokens[self.index] != expected {
            panic!("Expected {:?}, found {:?}", expected, self.tokens.get(self.index));
        }
        self.index += 1;
    }

    fn maybe_expect(&mut self, expected: Token) -> bool {
        if self.index < self.tokens.len() && self.tokens[self.index] == expected {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn parse_role_name(&mut self) -> String {
        if let Token::Identifier(name) = &self.tokens[self.index] {
            self.index += 1;
            name.clone()
        } else {
            panic!("Expected role name");
        }
    }

    fn parse_msg_type(&mut self) -> MsgType {
        match &self.tokens[self.index] {
            Token::Broadcast => {
                self.index += 1;
                MsgType::Broadcast
            }
            Token::Listen => {
                self.index += 1;
                MsgType::Listen
            }
            Token::Request => {
                self.index += 1;
                MsgType::Request
            }
            Token::Response => {
                self.index += 1;
                MsgType::Response
            }
            _ => panic!("Expected message type"),
        }
    }

    fn parse_msg_name(&mut self) -> String {
        if let Token::Identifier(name) = &self.tokens[self.index] {
            self.index += 1;
            name.clone()
        } else {
            panic!("Expected message name");
        }
    }

    fn parse_filter_exp(&mut self) -> String {
        if let Token::Identifier(exp) = &self.tokens[self.index] {
            self.index += 1;
            exp.clone()
        } else {
            panic!("Expected filter expression");
        }
    }



    fn skip_whitespace(&mut self) {
        while self.index < self.tokens.len() && matches!(self.tokens[self.index], Token::Whitespace) {
            self.index += 1;
        }
    }
}

pub fn parse(input: &str) -> Vec<Statement> {
    let tokens = lex(input);
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    println!("{:?}", statements);
    statements
}