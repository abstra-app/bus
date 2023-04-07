use futures::StreamExt;
use policy::{parse, Statement};
use std::env;
use warp::ws::WebSocket;
use warp::Filter;
mod policy;
mod validator;
use validator::message_from_str;
mod connections;
use lazy_static::lazy_static;


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
    let (mut ws_tx, mut ws_rx) = ws.split();
    while let Some(result) = ws_rx.next().await {
        let msg = result.unwrap();
        let message = message_from_str(
            POLICY.clone(),
            msg.to_str().unwrap()).unwrap();
        println!("Message: {:?}", message);
    }
}