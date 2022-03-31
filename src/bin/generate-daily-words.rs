use std::fs;
use std::collections::HashSet;
use rand::prelude::SliceRandom;

const WORDS: &str = include_str!("../../full-words.txt");

fn get_random_word(word_list: &[String]) -> String {
    word_list
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

fn parse_words(words: &str, word_length: usize) -> Vec<String> {
    words
        .lines()
        .filter(|word| word.chars().count() == word_length)
        .map(|word| word.chars().collect())
        .collect()
}

fn main() {
    let word_list = parse_words(WORDS, 5);

    let mut output: HashSet<String> = HashSet::new();
    while output.len() < 1000 {
        let word = get_random_word(&word_list);
        output.insert(word);
    }

    let output_data = output.into_iter().collect::<Vec<String>>().join("\n");
    fs::write("daily-words.txt", output_data).expect("Unable to write file");
}
