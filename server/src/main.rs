use futures::{StreamExt, SinkExt};
use policy::{parse, Statement};
use std::collections::HashMap;
use std::env;
use warp::{ws::WebSocket};
use warp::Filter;
mod policy;
mod validator;
use validator::{message_from_str, Message};
mod broker;
use lazy_static::lazy_static;
use uuid::Uuid;
use serde_json;
use std::sync::Arc;
use std::sync::Mutex;
use crate::validator::ResponseMessage;


fn get_policy() -> Vec<Statement> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            panic!("Usage: cargo run --bin server <path to policy file>");
        }
        2 => {
            let path = args[1].clone();
            let body = std::fs::read_to_string(path).unwrap();
            return parse(&body);
        }
        _ => {
            panic!("Usage: cargo run --bin server <path to policy file>");
        }
    }
}

lazy_static! {
    static ref POLICY: Vec<Statement> = get_policy();
    static ref BROKER: broker::Broker = broker::Broker::new();
}

#[tokio::main]
async fn main() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_websocket)
        });
    
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_websocket(ws: WebSocket) {
    println!("New websocket connection");
    let (ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    let uuid = Uuid::new_v4();
    while let Some(result) = ws_rx.next().await {
        let message = message_from_str(
            POLICY.clone(),
            result.unwrap().to_str().unwrap()
        ).unwrap();
        match message {
            Message::Request(request) => {
                let ws_tx = Arc::clone(&ws_tx);
                BROKER.request(uuid, request, Box::new(move |response: ResponseMessage| {
                    let json = serde_json::to_string(&response.payload).unwrap();
                    let response = warp::ws::Message::text(json);
                    let mut ws_tx = ws_tx.lock().unwrap();
                    ws_tx.send(response);
                }));
            },
            Message::Response(response) => {
                let ws_tx = Arc::clone(&ws_tx);
                BROKER.respond(uuid, response, Arc::new(move |request| {
                    return ResponseMessage {
                        channel: request.channel,
                        payload: HashMap::new(),
                    };
                }));
            },
            Message::Broadcast(event) => {
                BROKER.broadcast(event);
            },
            Message::Listen(event) => {
                let ws_tx = Arc::clone(&ws_tx);
                BROKER.listen(uuid, event, Arc::new(move |event| {}));
            }
        }
    }
}