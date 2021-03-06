use node_ini::{from_str, Item};

const INI: &str = r#"
toplevel = hello
other = world
"mismatched' = hi

; Amazing!
# [Wow]

[goodbye] # nother inline comment
# Wow, that's also a comment
cruel = intentions
world = Captain Planet! ; This is a comment
hello\;not a comment = "mismatched'
fun = goodbye\#also valid
"this should have ;everything" = yes
"this should stop #here = no
with no quotes this ;doesn't have = a value

[goodbye.amazing.nested]
test=Wow!

[goodbye\.amazing.nested]
test=Should be different
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
