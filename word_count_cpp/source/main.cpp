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
// File size: 3145740 bytes
// Chunk 0: 0 - 786435
// Chunk 1: 786435 - 1572870
// Chunk 2: 1572870 - 2359305
// Chunk 3: 2359305 - 3145740
// Thread 0 started
// Thread 1 started
// Thread 2 started
// Thread 3 started
// Read 786435 bytes from chunk
// Thread 0 read 22601 lines
// Thread 0 processed 10000 words
// Thread 0 processed 20000 words
// Thread 0 processed 30000 words
// Thread 0 processed 40000 words
// Thread 0 processed 50000 words
// Read 786435 bytes from chunk
// Thread 1 read 22643 lines
// Thread 0 processed 60000 words
// Thread 1 processed 10000 words
// Read 786435 bytes from chunk
// Thread 2 read 22642 lines
// Thread 0 processed 70000 words
// Read 786435 bytes from chunk
// Thread 3 read 22686 lines
// Thread 1 processed 20000 words
// Thread 2 processed 10000 words
// Thread 0 processed 80000 words
// Thread 3 processed 10000 words
// Thread 1 processed 30000 words
// Thread 2 processed 20000 words
// Thread 0 processed 90000 words
// Thread 3 processed 20000 words
// Thread 1 processed 40000 words
// Thread 2 processed 30000 words
// Thread 0 processed 100000 words
// Thread 3 processed 30000 words
// Thread 1 processed 50000 words
// Thread 2 processed 40000 words
// Thread 0 processed 110000 words
// Thread 3 processed 40000 words
// Thread 1 processed 60000 words
// Thread 2 processed 50000 words
// Thread 0 processed 120000 words
// Thread 3 processed 50000 words
// Thread 2 processed 60000 words
// Thread 1 processed 70000 words
// Thread 3 processed 60000 words
// Thread 0 processed 130000 words
// Thread 2 processed 70000 words
// Thread 1 processed 80000 words
// Thread 3 processed 70000 words
// Thread 0 processed 140000 words
// Thread 2 processed 80000 words
// Thread 1 processed 90000 words
// Thread 3 processed 80000 words
// Thread 0 processed 150000 words
// Thread 2 processed 90000 words
// Thread 0 finished processing 151725 words
// Thread 1 processed 100000 words
// Thread 3 processed 90000 words
// Thread 2 processed 100000 words
// Thread 1 processed 110000 words
// Thread 3 processed 100000 words
// Thread 2 processed 110000 words
// Thread 1 processed 120000 words
// Thread 3 processed 110000 words
// Thread 2 processed 120000 words
// Thread 1 processed 130000 words
// Thread 3 processed 120000 words
// Thread 2 processed 130000 words
// Thread 1 processed 140000 words
// Thread 3 processed 130000 words
// Thread 2 processed 140000 words
// Thread 1 processed 150000 words
// Thread 1 finished processing 151843 words
// Thread 3 processed 140000 words
// Thread 2 processed 150000 words
// Thread 2 finished processing 151745 words
// Thread 3 processed 150000 words
// Thread 3 finished processing 151767 words
// All threads finished, merging results
// Results written to output.txt
// Total processing time: 7639 ms
// Processing completed successfully.
//
// output.txt
// a: 98457
// algorithm: 5080
// all: 32725
// am: 33101
// analysis: 4931
// and: 32787
// artificial: 5044
// be: 99818
// box: 33463
// boy: 32775
// brown: 32309
// chocolates: 33424
// computer: 5010
// concurrency: 4781
// data: 4955
// database: 4703
// dear: 32990
// design: 5040
// dog: 32293
// dull: 32690
// elementary: 32952
// em: 4
// et: 32212
// fo: 3
// force: 33191
// fox: 32245
// hardware: 5009
// have: 32933
// home: 65348
// houston: 32941
// i: 66211
// intelligence: 5046
// is: 66735
// jack: 32802
// jumps: 32274
// lazy: 32261
// learning: 4963
// life: 33392
// like: 66448
// machine: 5048
// makes: 32770
// may: 33134
// multithreading: 5182
// my: 32955
// network: 4684
// no: 65840
// not: 33419
// of: 33398
// optimization: 4852
// or: 33472
// over: 32310
// performance: 4910
// phone: 32184
// place: 33252
// play: 32717
// probl: 4
// problem: 32909
// programming: 4940
// python: 5064
// question: 33482
// quick: 32291
// rce: 4
// science: 4941
// software: 5060
// that: 33473
// the: 130310
// therefore: 33181
// theres: 33251
// think: 33080
// to: 66810
// watson: 32910
// we: 32912
// with: 33129
// work: 32723
// you: 33206
