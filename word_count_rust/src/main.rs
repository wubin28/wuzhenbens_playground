use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, Write};
use std::path::Path;
use std::thread;
use std::time::Instant;

const BUFFER_SIZE: usize = 8192; // 8 KB buffer
const NUM_THREADS: usize = 16;  // 增加线程数
const NUM_RUNS: usize = 10;     // 增加运行次数

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

struct FileChunk {
    start: u64,
    end: u64,
}

fn divide_file_into_chunks(file_path: &Path, num_chunks: usize) -> io::Result<Vec<FileChunk>> {
    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();
    println!("File size: {} bytes", file_size);

    let chunk_size = file_size / num_chunks as u64;
    let mut chunks = Vec::new();

    for i in 0..num_chunks {
        let start = i as u64 * chunk_size;
        let end = if i == num_chunks - 1 {
            file_size
        } else {
            start + chunk_size
        };
        chunks.push(FileChunk { start, end });
        println!("Chunk {}: {} - {}", i, start, end);
    }

    Ok(chunks)
}

fn read_file_chunk(file_path: &Path, chunk: &FileChunk) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    reader.seek(io::SeekFrom::Start(chunk.start))?;

    let mut lines = Vec::new();
    let mut buffer = String::new();
    let mut bytes_read = 0;

    while reader.read_line(&mut buffer)? > 0 && reader.stream_position()? <= chunk.end {
        bytes_read += buffer.len() as u64;
        lines.push(buffer.trim().to_string());
        buffer.clear();
    }

    println!("Read {} bytes from chunk", bytes_read);
    Ok(lines)
}

fn process_word(word: &str) -> String {
    word.chars()
        .filter(|&c| !c.is_ascii_punctuation())
        .flat_map(char::to_lowercase)
        .collect()
}

fn count_words(lines: &[String], thread_id: usize) -> HashMap<String, usize> {
    let mut word_count = HashMap::new();
    let mut total_words = 0;

    for line in lines {
        for word in line.split_whitespace() {
            let processed_word = process_word(word);
            if !processed_word.is_empty() {
                *word_count.entry(processed_word).or_insert(0) += 1;
                total_words += 1;
                if total_words % 10000 == 0 {
                    println!("Thread {} processed {} words", thread_id, total_words);
                }
            }
        }
    }

    println!(
        "Thread {} finished processing {} words",
        thread_id, total_words
    );
    word_count
}

fn write_results(
    output_path: &Path,
    word_count: &HashMap<String, usize>,
) -> Result<(), WordCountError> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // 将HashMap转换为Vec并排序
    let mut sorted_words: Vec<_> = word_count.iter().collect();
    sorted_words.sort_by(|a, b| a.0.cmp(b.0));

    for (word, count) in sorted_words {
        writeln!(writer, "{}: {}", word, count)?;
    }

    println!("Results written to {}", output_path.display());
    Ok(())
}

// Unsafe global mutable state
static mut GLOBAL_WORD_COUNT: Option<HashMap<String, usize>> = None;

fn process_file(input_file: &str, output_file: &str) -> Result<(), WordCountError> {
    let start = Instant::now();
    println!("Starting file processing");

    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    let chunks = divide_file_into_chunks(input_path, NUM_THREADS)?;

    // Initialize the global word count
    unsafe {
        GLOBAL_WORD_COUNT = Some(HashMap::new());
    }

    let mut handles = vec![];

    for (i, chunk) in chunks.into_iter().enumerate() {
        let input_path = input_path.to_path_buf();

        let handle = thread::spawn(move || {
            println!("Thread {} started", i);
            let lines = read_file_chunk(&input_path, &chunk).unwrap();
            println!("Thread {} read {} lines", i, lines.len());
            let local_word_count = count_words(&lines, i);

            // Unsafe access to global state
            unsafe {
                let global_word_count = GLOBAL_WORD_COUNT.as_mut().unwrap();
                for (word, count) in local_word_count {
                    *global_word_count.entry(word).or_insert(0) += count;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads finished, merging results");

    // Unsafe access to get the final word count
    let final_word_count = unsafe { GLOBAL_WORD_COUNT.take().unwrap() };

    write_results(output_path, &final_word_count)?;

    let duration = start.elapsed();
    println!("Total processing time: {} ms", duration.as_millis());

    Ok(())
}

fn main() {
    let input_file = "input.txt";

    for run in 1..=NUM_RUNS {
        println!("Run {}", run);
        let output_file = format!("output_{}.txt", run);

        match process_file(input_file, &output_file) {
            Ok(_) => println!("Processing completed for run {}.", run),
            Err(e) => {
                eprintln!("Error in run {}: {}", run, e);
                std::process::exit(1);
            }
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
        let word_count = count_words(&lines, 0);
        assert_eq!(word_count.get("hello"), Some(&2));
        assert_eq!(word_count.get("world"), Some(&1));
        assert_eq!(word_count.get("rust"), Some(&1));
    }
}
// input.txt
// Run command 'python3 generate_input.py' to generate the large input file.
//
// Output:
// Starting word count process
// Starting file processing
// File size: 209733 bytes
// Chunk 0: 0 - 52433
// Chunk 1: 52433 - 104866
// Chunk 2: 104866 - 157299
// Chunk 3: 157299 - 209733
// Thread 0 started
// Thread 1 started
// Thread 3 started
// Thread 2 started
// Read 52428 bytes from chunk
// Thread 0 read 1511 lines
// Read 52412 bytes from chunk
// Thread 2 read 1524 lines
// Read 52434 bytes from chunk
// Thread 3 read 1523 lines
// Read 52415 bytes from chunk
// Thread 1 read 1511 lines
// Thread 0 processed 10000 words
// Thread 0 finished processing 10221 words
// Thread 1 processed 10000 words
// Thread 2 processed 10000 words
// Thread 3 processed 10000 words
// Thread 3 finished processing 10108 words
// Thread 2 finished processing 10123 words
// Thread 1 finished processing 10111 words
// All threads finished, merging results
// Results written to output.txt
// Total processing time: 22 ms
// Processing completed successfully.
//
// output.txt
// a: 1802
// algorithm: 76
// all: 618
// am: 616
// analysis: 101
// and: 618
// artificial: 88
// be: 1813
// box: 606
// boy: 618
// brown: 610
// chocolates: 606
// computer: 86
// concurrency: 93
// data: 91
// database: 113
// dear: 594
// design: 81
// dog: 610
// dull: 618
// elementary: 594
// et: 634
// force: 627
// fox: 610
// hardware: 83
// have: 578
// home: 1226
// houston: 577
// i: 1231
// intelligence: 78
// is: 1199
// jack: 618
// jumps: 610
// lazy: 610
// learning: 102
// life: 606
// like: 1198
// machine: 70
// makes: 618
// may: 627
// multithreading: 83
// my: 594
// network: 87
// no: 1210
// not: 593
// of: 606
// on: 1
// optimization: 87
// or: 593
// over: 610
// performance: 91
// phone: 634
// place: 592
// play: 618
// problem: 579
// programming: 83
// python: 83
// question: 593
// quick: 610
// refore: 1
// science: 85
// software: 91
// that: 593
// the: 2440
// therefore: 615
// theres: 592
// think: 615
// to: 1186
// watson: 594
// we: 578
// with: 627
// work: 618
// you: 627
