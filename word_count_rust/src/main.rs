use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192; // 8 KB buffer

#[derive(Debug)]
enum WordCountError {
    IoError(io::Error),
}

impl From<io::Error> for WordCountError {
    fn from(err: io::Error) -> Self {
        WordCountError::IoError(err)
    }
}

impl std::fmt::Display for WordCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WordCountError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for WordCountError {}

fn read_file_lines(file_path: &Path) -> Result<Vec<String>, WordCountError> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
    reader.lines().collect::<Result<_, _>>().map_err(Into::into)
}

fn process_word(word: &str) -> String {
    word.chars()
        .filter(|&c| !c.is_ascii_punctuation())
        .flat_map(char::to_lowercase)
        .collect()
}

fn count_words(lines: &[String]) -> HashMap<String, usize> {
    lines
        .iter()
        .flat_map(|line| line.split_whitespace())
        .map(process_word)
        .filter(|word| !word.is_empty())
        .fold(HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        })
}

fn write_results(
    output_path: &Path,
    word_count: &HashMap<String, usize>,
) -> Result<(), WordCountError> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for (word, count) in word_count {
        writeln!(writer, "{}: {}", word, count)?;
    }

    Ok(())
}

fn process_file(input_file: &str, output_file: &str) -> Result<(), WordCountError> {
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    let lines = read_file_lines(input_path)?;
    let word_count = count_words(&lines);
    write_results(output_path, &word_count)?;

    Ok(())
}

fn main() {
    let input_file = "input.txt";
    let output_file = "output.txt";

    match process_file(input_file, output_file) {
        Ok(_) => println!("Processing completed successfully."),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_word() {
        assert_eq!(process_word("Hello,"), "hello");
        assert_eq!(process_word("World!"), "world");
        assert_eq!(process_word("Rust-lang"), "rustlang");
    }

    #[test]
    fn test_count_words() {
        let lines = vec!["Hello, World!".to_string(), "Hello, Rust!".to_string()];
        let word_count = count_words(&lines);
        assert_eq!(word_count.get("hello"), Some(&2));
        assert_eq!(word_count.get("world"), Some(&1));
        assert_eq!(word_count.get("rust"), Some(&1));
    }
}
// input.txt
// The quick brown fox jumps over the lazy dog.
// The lazy dog sleeps all day.
// The quick brown fox is very clever.
// All work and no play makes Jack a dull boy.
//
// output.txt
// clever: 1
// and: 1
// play: 1
// work: 1
// jack: 1
// the: 4
// jumps: 1
// is: 1
// very: 1
// brown: 2
// over: 1
// day: 1
// makes: 1
// fox: 2
// dog: 2
// all: 2
// quick: 2
// lazy: 2
// a: 1
// boy: 1
// sleeps: 1
// dull: 1
// no: 1
