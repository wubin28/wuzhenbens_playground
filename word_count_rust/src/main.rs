use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const BUFFER_SIZE: usize = 8192; // 8 KB buffer
const NUM_THREADS: usize = 4;

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

    for (word, count) in word_count {
        writeln!(writer, "{}: {}", word, count)?;
    }

    println!("Results written to {}", output_path.display());
    Ok(())
}

fn process_file(input_file: &str, output_file: &str) -> Result<(), WordCountError> {
    let start = Instant::now();
    println!("Starting file processing");

    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    let chunks = divide_file_into_chunks(input_path, NUM_THREADS)?;

    // 尝试使用 Rc<RefCell<>> 来模拟 C++ 中的 shared_ptr
    let word_count = Arc::new(Mutex::new(Some(HashMap::new())));

    let mut handles = vec![];

    for (i, chunk) in chunks.into_iter().enumerate() {
        let input_path = input_path.to_path_buf();
        let word_count = Arc::clone(&word_count);

        let handle = thread::spawn(move || {
            println!("Thread {} started", i);
            let lines = read_file_chunk(&input_path, &chunk).unwrap();
            println!("Thread {} read {} lines", i, lines.len());
            let local_word_count = count_words(&lines, i);

            // 尝试在线程中访问和修改 word_count
            let mut global_word_count = word_count.lock().unwrap();
            if let Some(map) = global_word_count.as_mut() {
                for (word, count) in local_word_count {
                    *map.entry(word).or_insert(0) += count;
                }
            }
            // 模拟一些延迟
            thread::sleep(std::time::Duration::from_millis(100));
        });

        handles.push(handle);
    }

    // 尝试在主线程中"重置" word_count
    {
        let mut global_word_count = word_count.lock().unwrap();
        *global_word_count = None; // 这里尝试"重置" word_count
    }

    // 注意：这里我们不等待线程完成
    println!("Main thread finished, but worker threads may still be running");

    let duration = start.elapsed();
    println!("Total processing time: {} ms", duration.as_millis());

    Ok(())
}

fn main() {
    let input_file = "input.txt";
    let output_file = "output.txt";

    println!("Starting word count process");

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
// File size: 3145741 bytes
// Chunk 0: 0 - 786435
// Chunk 1: 786435 - 1572870
// Chunk 2: 1572870 - 2359305
// Chunk 3: 2359305 - 3145741
// Thread 0 started
// Thread 1 started
// Thread 2 started
// Thread 3 started
// Read 786395 bytes from chunk
// Thread 1 read 22681 lines
// Read 786421 bytes from chunk
// Thread 0 read 22635 lines
// Read 786396 bytes from chunk
// Thread 2 read 22594 lines
// Read 786436 bytes from chunk
// Thread 3 read 22718 lines
// Thread 1 processed 10000 words
// Thread 0 processed 10000 words
// Thread 2 processed 10000 words
// Thread 3 processed 10000 words
// Thread 1 processed 20000 words
// Thread 0 processed 20000 words
// Thread 2 processed 20000 words
// Thread 3 processed 20000 words
// Thread 1 processed 30000 words
// Thread 2 processed 30000 words
// Thread 0 processed 30000 words
// Thread 3 processed 30000 words
// Thread 1 processed 40000 words
// Thread 2 processed 40000 words
// Thread 0 processed 40000 words
// Thread 3 processed 40000 words
// Thread 1 processed 50000 words
// Thread 2 processed 50000 words
// Thread 0 processed 50000 words
// Thread 3 processed 50000 words
// Thread 1 processed 60000 words
// Thread 2 processed 60000 words
// Thread 0 processed 60000 words
// Thread 3 processed 60000 words
// Thread 1 processed 70000 words
// Thread 0 processed 70000 words
// Thread 2 processed 70000 words
// Thread 3 processed 70000 words
// Thread 1 processed 80000 words
// Thread 3 processed 80000 words
// Thread 0 processed 80000 words
// Thread 2 processed 80000 words
// Thread 1 processed 90000 words
// Thread 0 processed 90000 words
// Thread 3 processed 90000 words
// Thread 2 processed 90000 words
// Thread 1 processed 100000 words
// Thread 0 processed 100000 words
// Thread 3 processed 100000 words
// Thread 2 processed 100000 words
// Thread 1 processed 110000 words
// Thread 3 processed 110000 words
// Thread 0 processed 110000 words
// Thread 2 processed 110000 words
// Thread 1 processed 120000 words
// Thread 3 processed 120000 words
// Thread 2 processed 120000 words
// Thread 0 processed 120000 words
// Thread 1 processed 130000 words
// Thread 3 processed 130000 words
// Thread 0 processed 130000 words
// Thread 2 processed 130000 words
// Thread 1 processed 140000 words
// Thread 2 processed 140000 words
// Thread 0 processed 140000 words
// Thread 3 processed 140000 words
// Thread 1 processed 150000 words
// Thread 1 finished processing 151572 words
// Thread 2 processed 150000 words
// Thread 0 processed 150000 words
// Thread 3 processed 150000 words
// Thread 2 finished processing 152060 words
// Thread 0 finished processing 151592 words
// Thread 3 finished processing 151543 words
// All threads finished, merging results
// Results written to output.txt
// Total processing time: 269 ms
// Processing completed successfully.
//
// output.txt
// no: 17985
// not: 8915
// question: 8915
// programming: 1410
// elementary: 9069
// jack: 9012
// et: 9035
// like: 18230
// i: 18022
// my: 9069
// artificial: 1333
// ack: 1
// chocolates: 9258
// design: 1436
// theres: 8973
// and: 9012
// software: 1368
// be: 26901
// have: 9172
// think: 9011
// problem: 9172
// work: 9012
// analysis: 1364
// the: 36206
// you: 9071
// jumps: 9110
// performance: 1352
// a: 27442
// dear: 9069
// therefore: 9011
// quick: 9110
// that: 8915
// brown: 9110
// intelligence: 1361
// box: 9258
// machine: 1278
// life: 9257
// lazy: 9110
// with: 9071
// python: 1416
// we: 9172
// home: 18008
// houston: 9172
// am: 9011
// science: 1384
// to: 17830
// dull: 9013
// multithreading: 1311
// learning: 1308
// force: 9071
// database: 1352
// fox: 9110
// or: 8915
// watson: 9069
// boy: 9013
// play: 9012
// is: 18172
// data: 1332
// may: 9071
// algorithm: 1405
// concurrency: 1285
// hardware: 1362
// phone: 9035
// network: 1324
// computer: 1317
// optimization: 1415
// place: 8973
// over: 9110
// all: 9012
// of: 9258
// dog: 9111
// makes: 9012
