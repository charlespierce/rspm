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
        let mut parser = Self { data, ch: None };

        parser.step();
        parser
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
}

impl<C: Iterator<Item = char>> Parser<C> {
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

    fn parse_line_until(&mut self, end: &[char]) -> (bool, String) {
        let mut value = String::new();
        let mut possibly_quoted = false;
        let mut quoted_comment_loc = None;

        let found = loop {
            match self.ch {
                None | Some('\r') | Some('\n') => break false,
                Some(marker) if marker == ';' || marker == '#' => {
                    if possibly_quoted {
                        if quoted_comment_loc.is_none() {
                            quoted_comment_loc = Some(value.len());
                        }
                        value.push(marker);
                    } else {
                        break false;
                    }
                }
                Some(e) if end.contains(&e) => break true,
                Some('\\') => {
                    self.step();
                    match self.ch {
                        None | Some('\r') | Some('\n') => {
                            value.push('\\');
                            break false;
                        }
                        Some('n') => value.push('\n'),
                        Some('r') => value.push('\r'),
                        Some('t') => value.push('\t'),
                        Some('b') => value.push('\x08'),
                        Some('f') => value.push('\x0C'),
                        Some('\\') => value.push('\\'),
                        Some(c) => {
                            value.push(c);
                        }
                    }
                }
                Some(q) if q == '"' || q == '\'' => {
                    possibly_quoted = true;
                    value.push(q);
                }
                Some(c) => value.push(c),
            }
            self.step();
        };

        let mut trimmed = value.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            // Handle quoted values by removing the quotes
            trimmed = &trimmed[1..trimmed.len() - 1];
        } else if let Some(loc) = quoted_comment_loc {
            // If not quoted but we found a comment character,
            // treat the comment marker as the end of the value
            trimmed = value[0..loc].trim();
        }

        (found, trimmed.to_string())
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
        self.step(); // Consume the initial '[' character
        let (found, title) = self.parse_line_until(&[']']);
        // TODO: Parse multiple titles separated by '.'
        // Also need to run out the line and make sure there aren't any
        // extra characters on the same line as the section (other than whitespace / comments)
        if !found {
            match self.ch {
                Some('\r') | Some('\n') => {
                    panic!("Unclosed section header. Expected ']', found end of line.");
                }
                Some(c) => {
                    panic!("Unexpected character found. Expected ']', found '{}'", c);
                }
                None => {
                    panic!("Unclosed section header. Expected ']', found end of file.");
                }
            }
        } else {
            self.step(); // Consume the final ']' character
            title.trim().to_string()
        }
    }

    fn parse_key_value(&mut self) -> (String, String) {
        let (has_value, key) = self.parse_line_until(&['=']);

        if has_value {
            self.step(); // Consume the '=' character
            let (_, value) = self.parse_line_until(&[]);
            (key, value)
        } else {
            // To match the npm "ini" module's behavior, if a key doesn't exist, use the value "true"
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
