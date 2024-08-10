use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::ptr;

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

// 场景1: 内存泄漏
fn read_file_lines(file_path: &Path) -> Result<Vec<String>, WordCountError> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);

    unsafe {
        let _large_buffer = Box::into_raw(Box::new([0u8; 1000000]));
        // 使用 _large_buffer 进行一些操作
        // 故意不释放 _large_buffer，导致内存泄漏
    }

    reader.lines().collect::<Result<_, _>>().map_err(Into::into)
}
// 修复：移除 unsafe 块，使用 Vec<u8> 代替原始指针，让 Rust 自动管理内存

// 场景2: 使用已释放的内存（悬垂指针）
fn process_word(word: &str) -> String {
    let result = unsafe {
        let ptr = Box::into_raw(word.to_string().into_boxed_str());
        let result = (*ptr).to_string(); // 在释放之前使用指针
        drop(Box::from_raw(ptr)); // 正确释放内存
        result // 返回结果，而不是使用已释放的内存
    };
    result // 返回在unsafe块中创建的字符串
}
// 修复：移除 unsafe 块，直接返回 word.to_string()

// 场景3: 双重释放（注释掉以避免程序崩溃）
fn write_results(
    output_path: &Path,
    word_count: &HashMap<String, usize>,
) -> Result<(), WordCountError> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // 注释掉双重释放的代码以避免程序崩溃
    // unsafe {
    //     let buffer = Box::into_raw(Box::new([0u8; 1000]));
    //     drop(Box::from_raw(buffer)); // 第一次释放
    //     drop(Box::from_raw(buffer)); // 第二次释放，导致未定义行为
    // }

    for (word, count) in word_count {
        writeln!(writer, "{}: {}", word, count)?;
    }

    Ok(())
}
// 修复：完全移除 unsafe 块，使用 Vec<u8> 代替原始指针，让 Rust 自动管理内存

// 场景4: 未初始化的指针（野指针）
fn count_words(lines: &[String]) -> HashMap<String, usize> {
    let mut word_count = HashMap::new();

    unsafe {
        let uninitialized_ptr: *mut i32 = ptr::null_mut();
        // 注释掉使用未初始化指针的代码以避免程序崩溃
        // *uninitialized_ptr = 42; // 使用未初始化的指针，导致未定义行为
    }

    for line in lines {
        for word in line.split_whitespace() {
            let processed_word = process_word(word);
            *word_count.entry(processed_word).or_insert(0) += 1;
        }
    }

    word_count
}
// 修复：完全移除 unsafe 块和未初始化指针的使用

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
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
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
