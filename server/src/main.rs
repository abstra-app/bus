use warp::Filter;
use warp::ws::{WebSocket};
use std::env;

mod policy;
use policy::{BusParser, Rule};
use pest::Parser;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            println!("Usage: cargo run --bin server <path to policy file>");
            return;
        },
        2 => {},
        _ => {
            println!("Usage: cargo run --bin server <path to policy file>");
            return;
        }
    }
    let path = args[1].clone();
    let body = std::fs::read_to_string(path).unwrap();

    let parsed = BusParser::parse(Rule::body, &body).unwrap_or_else(|e| panic!("{}", e));
    for pair in parsed {
        println!("Rule: {:?}", pair.as_rule());
        for inner_pair in pair.into_inner() {
            println!("Inner Rule: {:?}", inner_pair.as_rule());
            println!("Inner Text: {:?}", inner_pair.as_str());
        }
    }
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_websocket)
        });

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_websocket(ws: WebSocket) {

}