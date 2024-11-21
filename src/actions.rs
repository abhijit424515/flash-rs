use pest::iterators::Pair;
use std::collections::{HashSet, VecDeque};
use std::process::Command;

use crate::Rule;
use crate::DB;
use crate::models::{display, Container, Primitive, Value};

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

pub fn deserialize(x: Pair<'_, Rule>) -> Primitive {
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

pub fn actions(pair: Pair<'_, Rule>) {
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
            pair.into_inner().for_each(|x| match DB.remove(x.as_str()) {
                Some(_) => count += 1,
                None => (),
            });

            Some(Value::Primitive(Primitive::Int(count)))
        }
        Rule::lpush | Rule::rpush => {
            let dir = pair.as_rule() == Rule::rpush;

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
                        if dir {
                            l.push_back(v);
                        } else {
                            l.push_front(v);
                        }
                    });
                }
                _ => panic!(), // [TODO] handle error
            };

            let sz = match val {
                Value::Container(Container::List(l)) => l.len() as i64,
                _ => 0,
            };
            Some(Value::Primitive(Primitive::Int(sz)))
        }
        Rule::lpop | Rule::rpop => {
            let dir = pair.as_rule() == Rule::rpop;

            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();
            let popcount = match z.next() {
                Some(x) => x.as_str().parse::<i64>().unwrap(),
                None => 1,
            };

            let mut val = DB.get_mut(k).unwrap();
            let val = val.value_mut();

            match val {
                Value::Container(Container::List(l)) => {
                    let mut count = 0;
                    for _ in 0..popcount {
                        if dir {
                            l.pop_back();
                        } else {
                            l.pop_front();
                        }
                        count += 1;
                    }
                    Some(Value::Primitive(Primitive::Int(count)))
                }
                _ => panic!(), // [TODO] handle error
            }
        }
        Rule::llen => {
            let k = pair.into_inner().next().unwrap().as_str();
            let sz = match DB.get(k) {
                Some(val) => {
                    let val = val.value();
                    match val {
                        Value::Container(Container::List(l)) => l.len() as i64,
                        Value::Container(Container::UnorderedSet(s)) => s.len() as i64,
                        _ => panic!(),
                    }
                }
                None => 0,
            };

            Some(Value::Primitive(Primitive::Int(sz)))
        }
        Rule::sadd => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();

            if !DB.contains_key(k) {
                DB.insert(
                    k.to_string(),
                    Value::Container(Container::UnorderedSet(HashSet::new())),
                );
            };

            let mut val = DB.get_mut(k).unwrap();
            let val = val.value_mut();

            let mut count = 0;
            match val {
                Value::Container(Container::UnorderedSet(s)) => {
                    z.for_each(|x| {
                        let v = deserialize(x);
                        if s.insert(v) {
                            count += 1;
                        }
                    });
                }
                _ => panic!(), // [TODO] handle error
            };

            Some(Value::Primitive(Primitive::Int(count)))
        }
        Rule::srem => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();

            match DB.contains_key(k) {
                true => {
                    let mut count = 0;
                    let mut val = DB.get_mut(k).unwrap();
                    let val = val.value_mut();

                    match val {
                        Value::Container(Container::UnorderedSet(s)) => {
                            z.for_each(|x| {
                                let v = deserialize(x);
                                if s.remove(&v) {
                                    count += 1;
                                }
                            });
                        }
                        _ => panic!(), // [TODO] handle error
                    };

                    Some(Value::Primitive(Primitive::Int(count)))
                }
                false => Some(Value::Primitive(Primitive::Int(0))),
            }
        }
        Rule::sismember => {
            let mut z = pair.into_inner();
            let k = z.next().unwrap().as_str();
            let v = deserialize(z.next().unwrap());

            let res = match DB.get(k) {
                Some(val) => {
                    let val = val.value();
                    match val {
                        Value::Container(Container::UnorderedSet(s)) => s.contains(&v),
                        _ => panic!(),
                    }
                }
                None => false,
            };

            Some(Value::Primitive(Primitive::Int(res as i64)))
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
