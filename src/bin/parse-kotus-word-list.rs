use std::fs;

const ALLOWED_KEYS: [char; 28] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];

fn parse_word_list(data: String) -> Vec<String> {
    let parts = data.split("<kotus-sanalista>\n").collect::<Vec<&str>>();
    let words = parts[1].split("</kotus-sanalista>").collect::<Vec<&str>>();

    let mut word_list = Vec::new();

    for line in words[0].lines() {
        let (word, _): (String, String) = serde_scan::scan!("<st><s>{}</s>{}" <- line).unwrap();

        let count = word.chars().count();

        if (count == 6 || count == 5)
            && word
                .to_uppercase()
                .chars()
                .all(|c| ALLOWED_KEYS.contains(&c))
        {
            word_list.push(word.to_uppercase());
        }
    }

    word_list
}

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("No path to word list file given");
    let data = fs::read_to_string(filename).expect("Unable to read word list file");

    let word_list = parse_word_list(data);

    let output_data = word_list.join("\n");
    fs::write("full-words-generated.txt", output_data).expect("Unable to write file");
}
