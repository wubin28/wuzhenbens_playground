# word_count_cpp

This is the word_count_cpp project.

# Parallel Word Count Workflow Diagram

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#e0e0e0', 'textColor': '#e0e0e0', 'lineColor': '#e0e0e0'}}}%%
graph TD
    classDef default fill:#2a2a2a,stroke:#e0e0e0,color:#e0e0e0;
    
    subgraph Process["单一进程"]
        Input["input.txt"]
        Main["主线程<br>(main函数)"]
        T1["T1"]
        T2["T2"]
        T3["T3"]
        T4["T4"]
        MainMerge["主线程<br>(合并结果)"]
        Output["output.txt"]

        Input -->|读取| Main
        Main -->|创建线程| T1
        Main -->|创建线程| T2
        Main -->|创建线程| T3
        Main -->|创建线程| T4
        T1 -->|线程join| MainMerge
        T2 -->|线程join| MainMerge
        T3 -->|线程join| MainMerge
        T4 -->|线程join| MainMerge
        MainMerge -->|写入| Output
    end

    classDef processClass fill:#2a2a2a,stroke:#e0e0e0,color:#e0e0e0,stroke-width:2px;
    class Process processClass;
```

# function diagram

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#e0e0e0', 'textColor': '#e0e0e0', 'lineColor': '#e0e0e0'}}}%%
flowchart TD
    classDef default fill:#2a2a2a,stroke:#e0e0e0,color:#e0e0e0;

    main["main()
    Functionality: Entry point of the program
Side Effects: Prints messages to stdout/stderr
Input: None
Output: int (0 for success, 1 for error)"]

processFile["processFile(inputFile: string_view, outputFile: string_view)
Functionality: Processes input file, calculates word frequencies
Side Effects: Prints messages, writes to output file
Input: inputFile, outputFile paths
Output: optional<string> (nullopt or error message)"]

divideFileIntoChunks["divideFileIntoChunks(filePath: path, numChunks: size_t)
Functionality: Divides file into chunks
Side Effects: Prints file size and chunk ranges
Input: filePath, numChunks
Output: vector<FileChunk>"]

readFileChunk["readFileChunk(filePath: path, chunk: FileChunk)
Functionality: Reads content from file chunk
Side Effects: Prints bytes read
Input: filePath, chunk
Output: vector<string> (lines)"]

countWords["countWords(lines: vector<string>, threadId: int)
Functionality: Counts word occurrences in text lines
Side Effects: Prints progress messages
Input: lines, threadId
Output: unordered_map<string, size_t>"]

processWord["processWord(word: string)
Functionality: Removes punctuation, converts to lowercase
Side Effects: None
Input: word
Output: string (processed word)"]

writeResults["writeResults(outputPath: path, wordCount: unordered_map<string, size_t>)
Functionality: Writes word count results to file
Side Effects: Creates/writes to output file, prints message
Input: outputPath, wordCount
Output: None"]

threadSafeOutput["threadSafeOutput(message: string)
Functionality: Thread-safe printing to stdout
Side Effects: Prints message to stdout
Input: message
Output: None"]

main -->|calls| processFile
processFile -->|calls| divideFileIntoChunks
processFile -->|calls| readFileChunk
processFile -->|calls| countWords
processFile -->|calls| writeResults
processFile -->|calls| threadSafeOutput
countWords -->|calls| processWord
countWords -->|calls| threadSafeOutput
divideFileIntoChunks -->|calls| threadSafeOutput
readFileChunk -->|calls| threadSafeOutput
writeResults -->|calls| threadSafeOutput

linkStyle default stroke:#e0e0e0,stroke-width:2px
```

# API documentation

## countWords

```
Function Name: countWords

Functionality:
Counts the occurrences of words in the given lines of text. The function processes each word (removes punctuation and converts to lowercase), then updates the word count. It outputs a progress message every 10,000 words processed and a completion message when finished.

Side Effects:
- Prints progress and completion messages using threadSafeOutput function

Input Parameters:
- lines: const std::vector<std::string>&
  A collection of text lines to process
- threadId: int
  The ID of the thread executing this function, used for logging

Output:
- Return Type: std::unordered_map<std::string, std::size_t>
- Business Meaning: A map where keys are processed words and values are the number of occurrences of each word in the input text

Notes:
- The function is marked as [[nodiscard]] and noexcept
- Uses processWord function to process each word
```

## divideFileIntoChunks

```
Function Name: divideFileIntoChunks

Functionality:
Divides the given file into a specified number of chunks. It calculates the file size and then creates chunks of equal size (the last chunk may be slightly larger).

Side Effects:
- Prints the file size and range of each chunk using threadSafeOutput function

Input Parameters:
- filePath: const std::filesystem::path&
  The path of the file to be divided
- numChunks: std::size_t
  The number of chunks to create

Output:
- Return Type: std::vector<FileChunk>
- Business Meaning: A vector of FileChunk structures, each defining the start and end positions of a chunk in the file

Notes:
- The function is marked as [[nodiscard]] and inline
- Throws std::runtime_error if unable to open the file
```

## main

```
Function Name: main

Functionality:
The entry point of the program. It defines input and output file paths, initiates the word count process, and handles potential errors.

Side Effects:
- Prints start and completion messages using threadSafeOutput function
- Prints error messages to standard error stream if an error occurs

Input Parameters: None

Output:
- Return Type: int
- Business Meaning: Returns 0 for successful execution, 1 if an error occurred

Notes:
- Uses constexpr std::string_view for input and output file paths
- Calls processFile function to perform the word count process
```

## processFile

```
Function Name: processFile

Functionality:
Processes the input file, calculates word frequencies, and writes the results to the output file. It uses multithreading to process different parts of the file in parallel.

Side Effects:
- Prints various processing stage messages using threadSafeOutput function
- Creates and writes to the output file

Input Parameters:
- inputFile: std::string_view
  The path of the input file to process
- outputFile: std::string_view
  The path of the output file to write results to

Output:
- Return Type: std::optional<std::string>
- Business Meaning: Returns std::nullopt if processing is successful; returns a string containing an error message if an error occurs

Notes:
- The function is marked as [[nodiscard]] and inline noexcept
- Uses std::chrono to measure processing time
- Creates NUM_THREADS threads to process file chunks in parallel
```

## processWord

```
Function Name: processWord

Functionality:
Processes a single word by removing all punctuation and converting it to lowercase.

Side Effects: None

Input Parameters:
- word: const std::string&
  The original word to process

Output:
- Return Type: std::string
- Business Meaning: Returns the processed word (without punctuation, all lowercase)

Notes:
- The function is marked as inline
- Uses std::remove_if and std::transform for processing
```

## readFileChunk

```
Function Name: readFileChunk

Functionality:
Reads content from a specified chunk of a file and splits it into lines.

Side Effects:
- Prints the number of bytes read using threadSafeOutput function

Input Parameters:
- filePath: const std::filesystem::path&
  The path of the file to read from
- chunk: const FileChunk&
  Defines the start and end positions of the file chunk to read

Output:
- Return Type: std::vector<std::string>
- Business Meaning: A vector containing all lines read from the specified file chunk

Notes:
- The function is marked as [[nodiscard]] and inline
- Uses a buffer of size BUFFER_SIZE (8192 bytes) for reading
- Throws std::runtime_error if unable to open the file
```

## threadSafeOutput

```
Function Name: threadSafeOutput

Functionality:
Prints a message to standard output in a thread-safe manner.

Side Effects:
- Prints the given message to standard output

Input Parameters:
- message: const std::string&
  The message to print

Output: None

Notes:
- Uses std::lock_guard with std::mutex for thread safety
```

## writeResults

```
Function Name: writeResults

Functionality:
Writes the word count results to the specified output file.

Side Effects:
- Creates and writes to the output file
- Prints a completion message using threadSafeOutput function

Input Parameters:
- outputPath: const std::filesystem::path&
  The path of the output file to write results to
- wordCount: const std::unordered_map<std::string, std::size_t>&
  A map containing words and their occurrence counts

Output: None

Notes:
- The function is marked as inline
- Sorts words alphabetically before writing to the file
- Throws std::runtime_error if unable to open the output file
```

# Building and installing

See the [BUILDING](BUILDING.md) document.

# Contributing

See the [CONTRIBUTING](CONTRIBUTING.md) document.

# Licensing

<!--
Please go to https://choosealicense.com/licenses/ and choose a license that
fits your needs. The recommended license for a project of this type is the
GNU AGPLv3.
-->
