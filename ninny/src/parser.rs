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
            // Have `parse_section_title` return a vector of Strings (split on non-escaped '.')
            // Then do a root.entry(part).or_insert_with(|| Item::Section(HashMap::new())) for each part
            // For the last we can use `map`, the rest can use `HashMap::new()`
            // If the final key already exists, then iterate each value and insert, otherwise use it directly

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
                    let (key, value) = self.parse_key_value();

                    // TODO: Handle a key that ends with [] or a value that is already an array
                    // Create an Identifier type that knows is_array for []
                    // When inserting, do the following triage:
                    //    - If is_array and exists already, look at type
                    //         - If Array, push to the end
                    //         - If Value, convert to Array then push to end
                    //         - If Section, error
                    //    - Otherwise, Look at existing type
                    //         - If Array, push to the end
                    //         - If Value, replace with new value
                    //         - If Section, error

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
            // TODO: Parse multiple titles separated by '.'
            // Also need to run out the line and make sure there aren't any
            // extra characters on the same line as the section (other than whitespace / comments)
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

    fn parse_line_until(&mut self, end: &[char]) -> (bool, String) {
        let mut value = String::new();
        let mut escape = false;

        let found = loop {
            match self.ch {
                None | Some('\r') | Some('\n') => {
                    break false;
                }
                Some(c) if c == ';' || c == '#' => {
                    if escape {
                        value.push(c);
                        escape = false;
                    } else {
                        break false;
                    }
                }
                Some(e) if end.contains(&e) => {
                    break true;
                }
                Some('\\') => {
                    if escape {
                        value.push('\\');
                        escape = false;
                    } else {
                        escape = true;
                    }
                }
                Some(c) => {
                    if escape {
                        value.push('\\');
                        escape = false;
                    }

                    value.push(c);
                }
            }
            self.step();
        };

        if escape {
            value.push('\\');
        }

        // TODO: Handle quoting by checking if the first and last character are the same quote
        // If they are, strip those off before returning

        (found, value.trim().to_string())
    }

    fn parse_key_value(&mut self) -> (String, String) {
        let (has_value, key) = self.parse_line_until(&['=']);

        if has_value {
            self.step(); // Consume the '=' character
            let (_, value) = self.parse_line_until(&[]);
            (key, value)
        } else {
            // To match the ini npm module's behavior, if a key doesn't exist, use the value "true"
            (key, String::from("true"))
        }
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
