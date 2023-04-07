use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct BusParser;