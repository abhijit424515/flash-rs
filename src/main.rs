mod models;

use dashmap::DashMap;
use lazy_static::lazy_static;
use models::{display, Container, Primitive, Value};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::collections::VecDeque;
use std::process::Command;
use std::{fs, hash::Hash};

lazy_static! {
    static ref DB: DashMap<String, Value> = DashMap::new();
}

// ----------------------------------------------------------------

pub fn deserialize(x: Pair<'_,Rule>) -> Primitive {
    match x.as_rule() {
        Rule::integer => {
            let i = x.as_str().parse::<i64>().unwrap();
            Primitive::Int(i)
        }
        Rule::float => {
            let f = x.as_str().to_string();
            Primitive::Flt(f)
        }
        Rule::str => {
            let s = x.as_str().to_string();
            Primitive::Str(s)
        }
        _ => unreachable!(),
    }
}

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
                Some(value) => Some(value.clone()), // [TODO] get shouldn't work for containers
                None => Some(Value::Primitive(Primitive::Nil)),
            }
        }
        Rule::set => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();

            let v = z.next().unwrap();
            let p = deserialize(v);

            DB.insert(k.to_string(), Value::Primitive(p));
            Some(Value::Primitive(Primitive::Ok))
        }
        Rule::del => {
            let mut count = 0;
            pair.into_inner().for_each(|x| {
                match DB.remove(x.as_str()) {
                    Some(_) => count += 1,
                    None => (),
                }
            });

            Some(Value::Primitive(Primitive::Int(count)))
        }
        Rule::lpush => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();

            if !DB.contains_key(k) {
                DB.insert(
                    k.to_string(),
                    Value::Container(Container::List(VecDeque::new())),
                );
            };

            let mut val = DB.get_mut(k).unwrap();
            let val = val.value_mut();

            match val {
                Value::Container(Container::List(l)) => {
                    z.for_each(|x| {
                        let v = deserialize(x);
                        l.push_front(v);
                    });
                }
                _ => unreachable!(), // [TODO] handle error
            };

            let sz = match val {
                Value::Container(Container::List(l)) => l.len() as i64,
                _ => 0,
            }; 
            Some(Value::Primitive(Primitive::Int(sz)))
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
