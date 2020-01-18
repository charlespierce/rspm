use std::collections::HashMap;

mod parser;

pub use parser::Item;
use parser::Parser;

pub fn from_str(data: &str) -> HashMap<String, Item> {
    Parser::new(data.chars()).parse()
}
