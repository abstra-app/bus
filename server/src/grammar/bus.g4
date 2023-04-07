grammar BusPolicy;

body
    : statement
    | body statement
    ;

statement
    : allow_stmt
    | msg_stmt
    | role_stmt
    ;

allow_stmt
    : 'allow' role_name msg_type msg_name
    | 'allow' role_name msg_type msg_name 'when' filter_exp
    ;

msg_type
    : 'broadcast'
    | 'listen'
    | 'request'
    | 'response'
    ;

msg_stmt
    : broadcast_stmt
    | request_stmt
    | response_stmt
    ;

broadcast_stmt
    : 'broadcast' msg_name msg_params
    ;

request_stmt
    : 'request' msg_name msg_params
    ;

response_stmt
    : 'response' msg_name msg_params
    ;

msg_params
    : param_name param_type
    | msg_params param_name param_type
    ;

role_stmt
    : 'role' role_name
    | 'role' role_name 'extends' role_name
    ;

role_name : IDENTIFIER;
msg_name : IDENTIFIER;
filter_exp : EXPRESSION;
param_name : IDENTIFIER;
param_type : IDENTIFIER;
IDENTIFIER : [a-zA-Z_][a-zA-Z0-9_]*;
EXPRESSION : [a-zA-Z_][a-zA-Z0-9_]*;

WS : [ \t\r\n]+ -> skip;