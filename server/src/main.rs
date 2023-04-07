use policy::{parse, Statement};
use warp::Filter;
use warp::ws::{WebSocket};
use std::env;
use futures::StreamExt;
mod policy;
mod validator;
use validator::{message_from_str};

fn get_policy() -> Option<Vec<Statement>> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            println!("Usage: cargo run --bin server <path to policy file>");
            return None;
        },
        2 => {
            let path = args[1].clone();
            let body = std::fs::read_to_string(path).unwrap();
            return Some(parse(&body));
        },
        _ => {
            println!("Usage: cargo run --bin server <path to policy file>");
            return None;
        }
    }

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
    let policy: Option<Vec<Statement>> = get_policy();
    let (mut ws_tx, mut ws_rx) = ws.split();
    while let Some(result) = ws_rx.next().await {
        let msg = result.unwrap();
        let message = message_from_str(
            policy.clone().unwrap(),
            msg.to_str().unwrap()).unwrap();
        println!("Message: {:?}", message);
    }
}