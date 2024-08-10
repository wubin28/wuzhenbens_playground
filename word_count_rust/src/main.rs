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

/// Reads lines from a file.
///
/// # Arguments
///
/// * `file_path` - The path to the file to read
///
/// # Returns
///
/// A vector of strings, each representing a line in the file
fn read_file_lines(file_path: &Path) -> Result<Vec<String>, WordCountError> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
    reader.lines().collect::<Result<_, _>>().map_err(Into::into)
}

/// Processes a word by removing punctuation and converting to lowercase.
///
/// # Arguments
///
/// * `word` - The word to process
///
/// # Returns
///
/// The processed word
fn process_word(word: &str) -> String {
    word.chars()
        .filter(|&c| !c.is_ascii_punctuation())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Counts words in the given lines.
///
/// # Arguments
///
/// * `lines` - A slice of strings, each representing a line of text
///
/// # Returns
///
/// A HashMap with words as keys and their counts as values
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

/// Writes the word count results to a file.
///
/// # Arguments
///
/// * `output_path` - The path to the output file
/// * `word_count` - A HashMap containing words and their counts
///
/// # Returns
///
/// Result indicating success or failure
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

/// Processes a file by counting words and writing results.
///
/// # Arguments
///
/// * `input_file` - The path to the input file
/// * `output_file` - The path to the output file
///
/// # Returns
///
/// Result indicating success or failure
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
