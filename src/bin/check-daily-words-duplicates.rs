use std::collections::HashSet;

const WORDS: &str = include_str!("../../daily-words.txt");

fn main() {
    let mut words = HashSet::new();

    for (line, word) in WORDS.lines().enumerate() {
        if !words.insert(word) {
            println!("{word} on line {line} is a duplicate");
        }
    }
}
