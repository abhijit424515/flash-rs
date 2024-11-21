use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    Ok,
    Nil,
    Int(i64),
    Flt(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Container<T>
where
    T: Eq + Hash,
{
    List(VecDeque<T>),
    UnorderedSet(HashSet<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Primitive(Primitive),
    Container(Container<Primitive>),
}

pub fn display(x: Value) -> String {
    match x {
        Value::Primitive(p) => match p {
            Primitive::Ok => format!("OK"),
            Primitive::Nil => format!("(nil)"),
            Primitive::Int(i) => format!("{}", i),
            Primitive::Flt(f) => format!("{}", f),
            Primitive::Str(s) => format!("{}", s),
        },
        Value::Container(c) => match c {
            Container::List(v) => {
                let mut s = String::new();
                for (i, x) in v.iter().enumerate() {
                    s.push_str(&format!(
                        "{}) {}\n",
                        i + 1,
                        display(Value::Primitive(x.clone()))
                    ));
                }
                s
            }
            Container::UnorderedSet(v) => {
                let mut s = String::new();
                for (i, x) in v.iter().enumerate() {
                    s.push_str(&format!(
                        "{}) {}\n",
                        i + 1,
                        display(Value::Primitive(x.clone()))
                    ));
                }
                s
            }
        },
    }
}