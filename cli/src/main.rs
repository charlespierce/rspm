use ninny::{from_str, Item};

const INI: &str = r#"
toplevel = hello
other = world

; Amazing!
# [Wow]

[goodbye] # nother inline comment
# Wow, that's also a comment
cruel = intentions
world = Captain Planet! ; This is a comment
"  hello  " = "mismatched'
"#;

fn main() {
    print_map(from_str(&INI), 1);
}

fn print_map(map: std::collections::HashMap<String, Item>, level: usize) {
    let i = indent(level);
    println!("{{");
    for (k, v) in map {
        print!("{}{}: ", i, k);
        match v {
            Item::Value(s) => println!("{}", s),
            Item::Section(m) => print_map(m, level + 1),
            Item::Array(_) => unreachable!(),
        }
    }
    println!("{}}}", indent(level - 1));
}

fn indent(level: usize) -> String {
    " ".repeat(level * 4)
}
