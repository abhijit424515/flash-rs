mod actions;
mod models;

use actions::actions;
use dashmap::DashMap;
use lazy_static::lazy_static;
use models::Value;
use pest::Parser;
use pest_derive::Parser;
use std::{fs, hash::Hash};

lazy_static! {
    static ref DB: DashMap<String, Value> = DashMap::new();
}

fn init_db() {
    // will use later to restore backup
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Repl;

fn main() {
    init_db();
    let input = fs::read_to_string("input").expect("cannot read file");

    input
        .lines()
        .into_iter()
        .for_each(|line| match Repl::parse(Rule::command, line) {
            Ok(pairs) => {
                pairs.into_iter().for_each(|pair| actions(pair));
            }
            Err(e) => {
                println!("[parse error]:\t{:?}", e);
            }
        });
}
