mod models;

use dashmap::DashMap;
use lazy_static::lazy_static;
use models::{display, Primitive, Value};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::process::Command;
use std::{fs, hash::Hash};

lazy_static! {
    static ref DB: DashMap<String, Value> = DashMap::new();
}

// ----------------------------------------------------------------

fn clear_terminal() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .expect("Failed to clear terminal");
    } else {
        Command::new("clear")
            .status()
            .expect("Failed to clear terminal");
    }
}

fn init_db() {
    // will use later to restore backup
}

// ----------------------------------------------------------------

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Repl;

fn actions(pair: Pair<'_, Rule>) {
    let x = match pair.as_rule() {
        Rule::get => {
            let key = pair.into_inner().next().unwrap().as_str();
            match DB.get(key) {
                Some(value) => Some(value.clone()),
                None => Some(Value::Primitive(Primitive::Nil)),
            }
        }
        Rule::set => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();

            let v = z.next().unwrap();
            let p = match v.as_rule() {
                Rule::integer => {
                    let i = v.as_str().parse::<i64>().unwrap();
                    Primitive::Int(i)
                }
                Rule::float => {
                    let f = v.as_str().to_string();
                    Primitive::Flt(f)
                }
                Rule::str => {
                    let s = v.as_str().to_string();
                    Primitive::Str(s)
                }
                _ => unreachable!(),
            };

            DB.insert(k.to_string(), Value::Primitive(p));
            Some(Value::Primitive(Primitive::Ok))
        }
        Rule::del => {
            let mut count = 0;

            let keys = pair.into_inner().map(|x| x.as_str()).collect::<Vec<_>>();
            keys.into_iter().for_each(|key| match DB.remove(key) {
                Some(_) => count += 1,
                None => (),
            });

            Some(Value::Primitive(Primitive::Int(count)))
        }
        Rule::clear => {
            clear_terminal();
            None
        }
        Rule::EOI => None,
        _ => unreachable!(),
    };

    if let Some(x) = x {
        println!("{}", display(x));
    }
}

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
