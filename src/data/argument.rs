use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Argument {
    One(String),

    Rules { rules: Vec<Rule>, value: Value },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Value {
    One(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Rule {
    pub action: String,
    pub features: HashMap<String, bool>,
}
