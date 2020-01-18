use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub enum Item {
    Value(String),
    Array(Vec<String>),
    Section(HashMap<String, Item>),
}

pub(crate) struct Parser<C> {
    data: C,
    ch: Option<char>,
}

impl<C: Iterator<Item = char>> Parser<C> {
    pub fn new(data: C) -> Self {
        let mut parser = Parser { data, ch: None };

        parser.step();
        parser
    }

    fn step(&mut self) {
        self.ch = self.data.next();
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.ch {
            if !c.is_whitespace() {
                break;
            }
            self.step();
        }
    }

    fn consume_comment(&mut self) {
        while let Some(c) = self.ch {
            self.step();
            if c == '\n' {
                break;
            }
        }
    }

    pub fn parse(mut self) -> HashMap<String, Item> {
        let mut root = self.parse_section();

        while self.ch.is_some() {
            let title = self.parse_section_title();
            let map = self.parse_section();

            // TODO: Handle tested section titles (with '.' characters in them)

            try_insert(&mut root, title, Item::Section(map));
        }

        root
    }

    fn parse_section(&mut self) -> HashMap<String, Item> {
        let mut section = HashMap::new();

        self.consume_whitespace();

        while let Some(c) = self.ch {
            match c {
                // Start of a new section, this section must be complete
                '[' => {
                    break;
                }
                // Comments, consume and ignore
                ';' | '#' => {
                    self.consume_comment();
                }
                _ => {
                    let key = self.parse_key();
                    let value = self.parse_value();

                    // TODO: Handle a key that ends with [] or a value that is already an array

                    try_insert(&mut section, key, Item::Value(value));
                }
            }

            self.consume_whitespace();
        }

        section
    }

    fn parse_section_title(&mut self) -> String {
        let mut title = String::new();

        // Consume the initial '[' character
        self.step();
        while let Some(c) = self.ch {
            match c {
                '\r' | '\n' => {
                    panic!("Unclosed section header. Expected ']', found end of line.");
                }
                ']' => {
                    self.step();
                    return title.trim().to_string();
                }
                _ => {
                    self.step();
                    title.push(c);
                }
            }
        }

        panic!("Unclosed section header. Expected ']', found end of file.");
    }

    fn parse_key(&mut self) -> String {
        let mut key = String::new();

        while let Some(c) = self.ch {
            match c {
                '\r' | '\n' | ';' | '#' => {
                    panic!("Key found with no value. Expected '=', found end of line.");
                }
                '=' => {
                    self.step();
                    return key.trim().to_string();
                }
                _ => {
                    self.step();
                    key.push(c);
                }
            }
        }

        panic!("Key found with no value. Expected '=', found end of line.");
    }

    fn parse_value(&mut self) -> String {
        let mut value = String::new();

        while let Some(c) = self.ch {
            match c {
                '\r' | '\n' | ';' | '#' => {
                    break;
                }
                _ => {
                    self.step();
                    value.push(c);
                }
            }
        }

        value.trim().to_string()
    }
}

fn try_insert(map: &mut HashMap<String, Item>, key: String, value: Item) {
    match map.entry(key) {
        Entry::Occupied(o) => panic!("Duplicate value found: {}", o.key()),
        Entry::Vacant(v) => {
            v.insert(value);
        }
    }
}
