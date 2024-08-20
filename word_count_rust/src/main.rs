use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

const NUM_THREADS: usize = 2;
const BUFFER_SIZE: usize = 8192; // 8 KB buffer

#[derive(Debug, Clone, Copy)] // 添加 Clone 和 Copy
struct FileChunk {
    start: u64,
    end: u64,
}

fn create_chunk_file(input_path: &Path, chunk: &FileChunk, chunk_index: usize) -> io::Result<()> {
    let mut input_file = File::open(input_path)?;
    input_file.seek(SeekFrom::Start(chunk.start))?;

    let chunk_path = PathBuf::from(format!("input_chunk_{}.txt", chunk_index));
    let mut chunk_file = File::create(&chunk_path)?;

    let mut buffer = vec![0; BUFFER_SIZE];
    let mut bytes_to_read = chunk.end - chunk.start;

    while bytes_to_read > 0 {
        let bytes_read =
            input_file.read(&mut buffer[..std::cmp::min(BUFFER_SIZE, bytes_to_read as usize)])?;
        if bytes_read == 0 {
            break;
        }
        chunk_file.write_all(&buffer[..bytes_read])?;
        bytes_to_read -= bytes_read as u64;
    }

    println!("Created chunk file: {}", chunk_path.display());
    Ok(())
}

fn divide_file_into_chunks(file_path: &Path, num_chunks: usize) -> io::Result<Vec<FileChunk>> {
    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    println!("File size: {} bytes", file_size);

    let mut chunks = Vec::new();
    let target_chunk_size = file_size / num_chunks as u64;
    let mut current_pos = 0;

    let mut reader = BufReader::new(file);

    for i in 0..num_chunks {
        let mut chunk = FileChunk {
            start: current_pos,
            end: current_pos,
        };

        if i == num_chunks - 1 {
            chunk.end = file_size;
        } else {
            let end_pos = std::cmp::min(current_pos + target_chunk_size, file_size);
            reader.seek_relative((end_pos - current_pos) as i64)?;

            let mut buf = String::new();
            reader.read_line(&mut buf)?;
            chunk.end = reader.stream_position()?;
        }

        println!("Chunk {}: {} - {}", i, chunk.start, chunk.end);
        create_chunk_file(file_path, &chunk, i)?; // 创建 chunk 文件
        chunks.push(chunk);

        current_pos = chunk.end;
        if current_pos >= file_size {
            break;
        }
    }

    Ok(chunks)
}

fn read_file_chunk(file_path: &Path, chunk: &FileChunk) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    reader.seek(io::SeekFrom::Start(chunk.start))?;

    let mut lines = Vec::new();
    let mut buffer = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        if !buffer.trim().is_empty() {
            lines.push(buffer.trim().to_string());
        }
        buffer.clear();
        if reader.stream_position()? >= chunk.end {
            break;
        }
    }

    println!("Read {} bytes from chunk", chunk.end - chunk.start);
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

fn write_results(output_path: &Path, word_count: &HashMap<String, usize>) -> io::Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut sorted_words: Vec<_> = word_count.iter().collect();
    sorted_words.sort_by(|a, b| {
        if a.0.chars().all(|c| c.is_ascii_digit()) && b.0.chars().all(|c| c.is_ascii_digit()) {
            a.0.parse::<u64>()
                .unwrap()
                .cmp(&b.0.parse::<u64>().unwrap())
        } else {
            a.0.cmp(b.0)
        }
    });

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for (word, count) in sorted_words {
        writeln!(writer, "{}: {}", word, count)?;
    }

    println!("Results written to {}", output_path.display());
    Ok(())
}

fn process_file(input_file: &str, output_file: &str) -> io::Result<()> {
    let start = std::time::Instant::now();

    println!("Starting file processing");
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    let chunks = divide_file_into_chunks(input_path, NUM_THREADS)?;

    let word_count = Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];

    for (i, chunk) in chunks.into_iter().enumerate() {
        let word_count = Arc::clone(&word_count);
        let input_path = input_path.to_path_buf();

        let handle = thread::spawn(move || {
            println!("Thread {} started", i);
            let lines = read_file_chunk(&input_path, &chunk).unwrap();
            println!("Thread {} read {} lines", i, lines.len());
            let thread_word_count = count_words(&lines, i);

            let mut total_word_count = word_count.lock().unwrap();
            for (word, count) in thread_word_count {
                *total_word_count.entry(word).or_insert(0) += count;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads finished, merging results");

    let total_word_count = Arc::try_unwrap(word_count).unwrap().into_inner().unwrap();
    write_results(output_path, &total_word_count)?;

    let duration = start.elapsed();
    println!("Total processing time: {} ms", duration.as_millis());

    Ok(())
}

fn main() {
    const INPUT_FILE: &str = "./input.txt";
    const OUTPUT_FILE: &str = "./output.txt";

    println!("Starting word count process");

    if let Err(err) = process_file(INPUT_FILE, OUTPUT_FILE) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    } else {
        println!("Processing completed successfully.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_divide_file_into_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
        fs::write(&file_path, content).unwrap();

        let chunks = divide_file_into_chunks(&file_path, 3).unwrap();

        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_read_file_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_read_file.txt");
        let content = "Line 1\nLine 2\nLine 3\nLine 4\n";
        fs::write(&file_path, content).unwrap();

        let chunk = FileChunk { start: 7, end: 19 };
        let lines = read_file_chunk(&file_path, &chunk).unwrap();

        assert_eq!(lines, vec!["Line 2", "Line 3"]);
    }

    #[test]
    fn test_process_word() {
        assert_eq!(process_word("Hello!"), "hello");
        assert_eq!(process_word("WORLD"), "world");
        assert_eq!(process_word("he!llo"), "hello");
        assert_eq!(process_word(""), "");
        assert_eq!(process_word("!!!"), "");
        assert_eq!(process_word("hello123"), "hello123");
    }

    #[test]
    fn test_count_words() {
        let lines = vec!["Hello world".to_string(), "hello Universe".to_string()];
        let result = count_words(&lines, 0);

        assert_eq!(result.len(), 3);
        assert_eq!(result["hello"], 2);
        assert_eq!(result["world"], 1);
        assert_eq!(result["universe"], 1);
    }

    #[test]
    fn test_write_results() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.txt");
        let word_count = {
            let mut map = HashMap::new();
            map.insert("world".to_string(), 2);
            map.insert("hello".to_string(), 1);
            map.insert("test".to_string(), 3);
            map
        };

        write_results(&output_path, &word_count).unwrap();

        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "hello: 1\ntest: 3\nworld: 2\n");
    }

    #[test]
    fn test_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.txt");
        let output_path = temp_dir.path().join("output.txt");

        let content = "Hello world\nThis is a test\nHello again\n";
        fs::write(&input_path, content).unwrap();

        process_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

        let result = fs::read_to_string(&output_path).unwrap();
        assert!(result.contains("hello: 2"));
        assert!(result.contains("world: 1"));
        assert!(result.contains("this: 1"));
        assert!(result.contains("is: 1"));
        assert!(result.contains("a: 1"));
        assert!(result.contains("test: 1"));
        assert!(result.contains("again: 1"));
    }
}
