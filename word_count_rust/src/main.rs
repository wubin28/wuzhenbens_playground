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
    let chunk_path = PathBuf::from(format!("input_chunk_{}.txt", chunk_index));
    let mut chunk_file = File::create(&chunk_path)?;

    if chunk.start < chunk.end {
        let mut input_file = File::open(input_path)?;
        input_file.seek(SeekFrom::Start(chunk.start))?;

        let mut buffer = vec![0; BUFFER_SIZE];
        let mut bytes_to_read = chunk.end - chunk.start;

        while bytes_to_read > 0 {
            let bytes_read = input_file
                .read(&mut buffer[..std::cmp::min(BUFFER_SIZE, bytes_to_read as usize)])?;
            if bytes_read == 0 {
                break;
            }
            chunk_file.write_all(&buffer[..bytes_read])?;
            bytes_to_read -= bytes_read as u64;
        }
    }

    println!("Created chunk file: {}", chunk_path.display());
    Ok(())
}

fn divide_file_into_chunks(file_path: &Path, num_chunks: usize) -> io::Result<Vec<FileChunk>> {
    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    println!("File size: {} bytes", file_size);

    let mut chunks = Vec::new();
    if file_size == 0 {
        chunks.push(FileChunk { start: 0, end: 0 });
        create_chunk_file(file_path, &chunks[0], 0)?;
        return Ok(chunks);
    }

    let target_chunk_size = std::cmp::max(1, file_size / num_chunks as u64);
    let mut current_pos = 0;
    let mut reader = BufReader::new(file);

    for i in 0..num_chunks {
        if current_pos >= file_size {
            break;
        }

        let mut chunk = FileChunk {
            start: current_pos,
            end: std::cmp::min(current_pos + target_chunk_size, file_size),
        };

        if i < num_chunks - 1 && chunk.end < file_size {
            reader.seek(SeekFrom::Start(chunk.end))?;
            let mut buf = String::new();
            reader.read_line(&mut buf)?;
            chunk.end = reader.stream_position()?;

            // If this chunk is too small, extend it to the next line
            if chunk.end - chunk.start < target_chunk_size / 2 && chunk.end < file_size {
                reader.read_line(&mut buf)?;
                chunk.end = reader.stream_position()?;
            }
        } else {
            chunk.end = file_size;
        }

        println!("Chunk {}: {} - {}", i, chunk.start, chunk.end);
        create_chunk_file(file_path, &chunk, i)?;
        chunks.push(chunk);

        if chunk.end == file_size {
            break;
        }

        current_pos = chunk.end;
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

    mod test_divide_file_into_chunks {
        use super::*;

        #[test]
        fn test_divide_file_into_equal_chunks() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test_equal_chunks.txt");
            let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\n";
            fs::write(&file_path, content).unwrap();

            let chunks = divide_file_into_chunks(&file_path, 3).unwrap();

            assert!(
                chunks.len() >= 2 && chunks.len() <= 3,
                "Expected 2 or 3 chunks, got {}",
                chunks.len()
            );
            assert_eq!(chunks[0].start, 0);
            assert!(chunks[0].end >= 14); // At least "Line 1\nLine 2\n"
            assert_eq!(chunks[1].start, chunks[0].end);
            if chunks.len() == 3 {
                assert!(chunks[1].end >= 28); // At least up to "Line 3\nLine 4\n"
                assert_eq!(chunks[2].start, chunks[1].end);
                assert_eq!(chunks[2].end, 42); // Total file size
            } else {
                assert_eq!(chunks[1].end, 42); // Total file size
            }
        }

        #[test]
        fn test_divide_empty_file() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("empty_file.txt");
            fs::write(&file_path, "").unwrap();

            let chunks = divide_file_into_chunks(&file_path, 3).unwrap();

            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0].start, 0);
            assert_eq!(chunks[0].end, 0);
        }

        #[test]
        fn test_divide_file_more_chunks_than_lines() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("more_chunks.txt");
            let content = "Line 1\nLine 2\n";
            fs::write(&file_path, content).unwrap();

            let chunks = divide_file_into_chunks(&file_path, 5).unwrap();

            assert!(
                chunks.len() <= 5,
                "Expected at most 5 chunks, got {}",
                chunks.len()
            );
            assert!(
                chunks.len() >= 2,
                "Expected at least 2 chunks, got {}",
                chunks.len()
            );
            assert_eq!(chunks[0].start, 0);
            assert!(chunks[0].end > 0);
            assert_eq!(chunks.last().unwrap().end, 14); // Total file size
        }

        #[test]
        fn test_divide_file_with_very_long_line() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("long_line.txt");
            let content =
                "Short line\n".to_string() + &"A".repeat(10000) + "\nAnother short line\n";
            let content_len = content.len() as u64;
            fs::write(&file_path, content).unwrap();

            let chunks = divide_file_into_chunks(&file_path, 3).unwrap();

            assert!(
                chunks.len() <= 3,
                "Expected at most 3 chunks, got {}",
                chunks.len()
            );

            // Check if any chunk contains the long line
            let long_line_chunk = chunks.iter().find(|chunk| chunk.end - chunk.start >= 10000);
            assert!(long_line_chunk.is_some(), "No chunk contains the long line");

            // Ensure the last chunk ends at the file size
            assert_eq!(chunks.last().unwrap().end, content_len);
        }

        #[test]
        fn test_divide_file_into_unequal_chunks() {
            // Given
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test_unequal_chunks.txt");
            let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
            fs::write(&file_path, content).unwrap();

            // When
            let chunks = divide_file_into_chunks(&file_path, 3).unwrap();

            // Then
            assert_eq!(chunks.len(), 3);
            assert!(chunks[0].end > chunks[0].start);
            assert!(chunks[1].end > chunks[1].start);
            assert!(chunks[2].end > chunks[2].start);
            assert_eq!(chunks[2].end, 35); // Total file size
        }

        #[test]
        fn test_divide_file_with_one_chunk() {
            // Given
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("one_chunk.txt");
            let content = "Line 1\nLine 2\nLine 3\n";
            fs::write(&file_path, content).unwrap();

            // When
            let chunks = divide_file_into_chunks(&file_path, 1).unwrap();

            // Then
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0].start, 0);
            assert_eq!(chunks[0].end, 21); // Total file size
        }

        #[test]
        #[should_panic(expected = "No such file or directory")]
        fn test_divide_non_existent_file() {
            // Given
            let non_existent_file = Path::new("non_existent_file.txt");

            // When
            divide_file_into_chunks(non_existent_file, 3).unwrap();

            // Then
            // The function should panic with "No such file or directory" error
        }
    }
}
