#include <algorithm>
#include <cctype>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <mutex>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <thread>
#include <unordered_map>
#include <vector>

namespace
{
constexpr std::size_t BUFFER_SIZE = 8192;  // 8 KB buffer
constexpr std::size_t NUM_THREADS = 4;  // 线程数

struct FileChunk
{
  std::streamoff start;
  std::streamoff end;
};

std::mutex cout_mutex;

void threadSafeOutput(const std::string& message)
{
  std::lock_guard<std::mutex> lock(cout_mutex);
  std::cout << message << std::endl;
}

[[nodiscard]] inline std::vector<FileChunk> divideFileIntoChunks(
    const std::filesystem::path& filePath, std::size_t numChunks)
{
  std::ifstream file(filePath, std::ios::binary | std::ios::ate);
  if (!file) {
    throw std::runtime_error("Unable to open file: " + filePath.string());
  }

  std::streamoff fileSize = file.tellg();
  file.seekg(0, std::ios::beg);

  threadSafeOutput("File size: " + std::to_string(fileSize) + " bytes");

  std::vector<FileChunk> chunks;
  std::streamoff chunkSize = fileSize / static_cast<std::streamoff>(numChunks);

  for (std::size_t i = 0; i < numChunks; ++i) {
    FileChunk chunk;
    chunk.start = static_cast<std::streamoff>(i) * chunkSize;
    chunk.end = (i == numChunks - 1) ? fileSize : chunk.start + chunkSize;
    chunks.push_back(chunk);
    threadSafeOutput("Chunk " + std::to_string(i) + ": "
                     + std::to_string(chunk.start) + " - "
                     + std::to_string(chunk.end));
  }

  return chunks;
}

[[nodiscard]] inline std::vector<std::string> readFileChunk(
    const std::filesystem::path& filePath, const FileChunk& chunk)
{
  std::ifstream file(filePath, std::ios::in | std::ios::binary);
  if (!file) {
    throw std::runtime_error("Unable to open file: " + filePath.string());
  }

  file.seekg(chunk.start);
  std::vector<std::string> lines;
  std::string buffer;
  buffer.reserve(BUFFER_SIZE);

  std::streamoff bytesRead = 0;
  bool reachedEnd = false;
  while (file && !reachedEnd) {
    char ch;
    file.read(&ch, 1);
    if (file.eof()) {
      reachedEnd = true;
      break;
    }

    bytesRead++;

    if (ch == '\n') {
      lines.push_back(std::move(buffer));
      buffer.clear();
      buffer.reserve(BUFFER_SIZE);
      if (file.tellg() >= chunk.end) {
        reachedEnd = true;
      }
    } else {
      buffer.push_back(ch);
    }

    if (buffer.size() == BUFFER_SIZE) {
      lines.push_back(std::move(buffer));
      buffer.clear();
      buffer.reserve(BUFFER_SIZE);
    }
  }

  // If we're not at the end of the file, read until the next newline
  if (!reachedEnd && !file.eof()) {
    std::string remainingBuffer;
    std::getline(file, remainingBuffer);
    buffer += remainingBuffer;
  }

  if (!buffer.empty()) {
    lines.push_back(std::move(buffer));
  }

  threadSafeOutput("Read " + std::to_string(bytesRead) + " bytes from chunk");
  return lines;
}
}  // namespace

inline std::string processWord(const std::string& word)
{
  std::string processed = word;
  // 移除标点符号
  processed.erase(
      std::remove_if(processed.begin(),
                     processed.end(),
                     [](char c)
                     { return std::ispunct(static_cast<unsigned char>(c)); }),
      processed.end());
  // 转换为小写
  std::transform(processed.begin(),
                 processed.end(),
                 processed.begin(),
                 [](unsigned char c) { return std::tolower(c); });
  return processed;
}

[[nodiscard]] inline std::unordered_map<std::string, std::size_t> countWords(
    const std::vector<std::string>& lines, int threadId) noexcept
{
  std::unordered_map<std::string, std::size_t> wordCount;
  std::size_t totalWords = 0;
  for (const auto& line : lines) {
    std::istringstream iss(line);
    std::string word;
    while (iss >> word) {
      std::string processedWord = processWord(word);
      if (!processedWord.empty()
          && processedWord.find_first_not_of(" \t\n\r") != std::string::npos)
      {
        ++wordCount[processedWord];
        ++totalWords;
        if (totalWords % 10000 == 0) {
          threadSafeOutput("Thread " + std::to_string(threadId) + " processed "
                           + std::to_string(totalWords) + " words");
        }
      }
    }
  }
  threadSafeOutput("Thread " + std::to_string(threadId)
                   + " finished processing " + std::to_string(totalWords)
                   + " words");
  return wordCount;
}

inline void writeResults(
    const std::filesystem::path& outputPath,
    const std::unordered_map<std::string, std::size_t>& wordCount)
{
  std::vector<std::pair<std::string, std::size_t>> sortedWords(
      wordCount.begin(), wordCount.end());

  std::sort(sortedWords.begin(),
            sortedWords.end(),
            [](const auto& a, const auto& b) { return a.first < b.first; });

  std::ofstream outFile(outputPath);
  if (!outFile) {
    throw std::runtime_error("Unable to open output file: "
                             + outputPath.string());
  }

  for (const auto& [word, count] : sortedWords) {
    outFile << word << ": " << count << '\n';
  }
  threadSafeOutput("Results written to " + outputPath.string());
}

[[nodiscard]] inline std::optional<std::string> processFile(
    std::string_view inputFile, std::string_view outputFile) noexcept
{
  try {
    const auto inputPath = std::filesystem::path(inputFile);
    const auto outputPath = std::filesystem::path(outputFile);

    auto start = std::chrono::high_resolution_clock::now();

    threadSafeOutput("Starting file processing");
    auto chunks = divideFileIntoChunks(inputPath, NUM_THREADS);
    std::vector<std::thread> threads;
    std::vector<std::unordered_map<std::string, std::size_t>> threadResults(
        NUM_THREADS);

    for (std::size_t i = 0; i < NUM_THREADS; ++i) {
      threads.emplace_back(
          [&, i]()
          {
            threadSafeOutput("Thread " + std::to_string(i) + " started");
            auto lines = readFileChunk(inputPath, chunks[i]);
            threadSafeOutput("Thread " + std::to_string(i) + " read "
                             + std::to_string(lines.size()) + " lines");
            threadResults[i] = countWords(lines, i);
          });
    }

    for (auto& thread : threads) {
      thread.join();
    }

    threadSafeOutput("All threads finished, merging results");

    std::unordered_map<std::string, std::size_t> totalWordCount;
    for (const auto& result : threadResults) {
      for (const auto& [word, count] : result) {
        totalWordCount[word] += count;
      }
    }

    writeResults(outputPath, totalWordCount);

    auto end = std::chrono::high_resolution_clock::now();
    auto duration =
        std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    threadSafeOutput(
        "Total processing time: " + std::to_string(duration.count()) + " ms");

    return std::nullopt;  // 成功时返回空的 optional
  } catch (const std::exception& e) {
    return e.what();  // 失败时返回错误信息
  }
}

int main()
{
  constexpr std::string_view inputFile = "../input.txt";
  constexpr std::string_view outputFile = "output.txt";

  threadSafeOutput("Starting word count process");

  if (const auto error = processFile(inputFile, outputFile)) {
    std::cerr << "Error: " << *error << '\n';
    return 1;
  } else {
    std::cout << "Processing completed successfully.\n";
    return 0;
  }
}
// input.txt
// Run command 'python3 generate_input.py' to generate the large input file.
//
// output:
// Starting word count process
// Starting file processing
// File size: 209733 bytes
// Chunk 0: 0 - 52433
// Chunk 1: 52433 - 104866
// Chunk 2: 104866 - 157299
// Chunk 3: 157299 - 209733
// Thread 0 started
// Thread 1 started
// Thread 2 started
// Thread 3 started
// Read 52446 bytes from chunk
// Thread 2 read 1525 lines
// Read 52434 bytes from chunk
// Thread 3 read 1523 lines
// Read 52456 bytes from chunk
// Thread 0 read 1512 lines
// Read 52452 bytes from chunk
// Thread 1 read 1512 lines
// Thread 2 processed 10000 words
// Thread 2 finished processing 10129 words
// Thread 0 processed 10000 words
// Thread 3 processed 10000 words
// Thread 0 finished processing 10226 words
// Thread 3 finished processing 10108 words
// Thread 1 processed 10000 words
// Thread 1 finished processing 10117 words
// All threads finished, merging results
// Results written to output.txt
// Total processing time: 32 ms
// Processing completed successfully.
//
// output.txt
// a: 1804
// algorithm: 76
// all: 618
// am: 617
// analysis: 102
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
// hardware: 84
// have: 580
// home: 1226
// houston: 579
// i: 1233
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
// problem: 581
// programming: 83
// python: 83
// question: 593
// quick: 610
// refore: 1
// science: 85
// software: 91
// that: 593
// the: 2440
// therefore: 616
// theres: 592
// think: 616
// to: 1186
// watson: 594
// we: 580
// with: 627
// work: 618
// you: 627
