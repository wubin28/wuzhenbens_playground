#include <algorithm>
#include <cctype>
#include <fstream>
#include <iostream>
#include <mutex>
#include <sstream>
#include <stdexcept>
#include <thread>

#include "lib.hpp"

namespace
{
std::mutex cout_mutex;

static void threadSafeOutput(const std::string& message)
{
  std::lock_guard<std::mutex> lock(cout_mutex);
  std::cout << message << std::endl;
}

constexpr std::size_t BUFFER_SIZE = 8192;  // 8 KB buffer
constexpr std::size_t NUM_THREADS = 4;  // 线程数

// 新增：创建chunk文件的辅助函数
void createChunkFile(const std::filesystem::path& inputPath,
                     const word_count::FileChunk& chunk,
                     int chunkIndex)
{
  std::ifstream inputFile(inputPath, std::ios::binary);
  if (!inputFile) {
    throw std::runtime_error("Unable to open input file: "
                             + inputPath.string());
  }

  std::filesystem::path chunkPath = std::filesystem::current_path()
      / ("input_chunk_" + std::to_string(chunkIndex) + ".txt");
  std::ofstream chunkFile(chunkPath, std::ios::binary);
  if (!chunkFile) {
    throw std::runtime_error("Unable to create chunk file: "
                             + chunkPath.string());
  }

  inputFile.seekg(chunk.start);
  std::vector<char> buffer(BUFFER_SIZE);
  std::streamsize bytesToRead = chunk.end - chunk.start;

  while (bytesToRead > 0 && inputFile) {
    std::streamsize bytesRead =
        std::min(static_cast<std::streamsize>(buffer.size()), bytesToRead);
    inputFile.read(buffer.data(), bytesRead);
    chunkFile.write(buffer.data(), inputFile.gcount());
    bytesToRead -= inputFile.gcount();
  }

  threadSafeOutput("Created chunk file: " + chunkPath.string());
}
}  // namespace

namespace word_count
{

std::vector<FileChunk> divideFileIntoChunks(
    const std::filesystem::path& filePath, std::size_t numChunks)
{
  if (numChunks == 0) {
    throw std::invalid_argument("Number of chunks must be greater than zero");
  }

  std::ifstream file(filePath, std::ios::binary | std::ios::ate);
  if (!file) {
    throw std::runtime_error("Unable to open file: " + filePath.string());
  }

  std::streamoff fileSize = file.tellg();
  file.seekg(0, std::ios::beg);

  threadSafeOutput("File size: " + std::to_string(fileSize) + " bytes");

  std::vector<FileChunk> chunks;
  std::streamoff targetChunkSize =
      fileSize / static_cast<std::streamoff>(numChunks);
  std::streamoff currentPos = 0;

  for (std::size_t i = 0; i < numChunks && currentPos < fileSize; ++i) {
    FileChunk chunk;
    chunk.start = currentPos;

    // 移动到目标chunk结束位置
    std::streamoff endPos = std::min(currentPos + targetChunkSize, fileSize);
    file.seekg(endPos);

    // 如果不是文件末尾，继续读取直到下一个换行符
    if (endPos < fileSize) {
      std::string line;
      if (std::getline(file, line)) {
        endPos = file.tellg();
      }
    }

    // 如果这个chunk太大了，我们尝试在上一个换行符处分割
    if (i < numChunks - 1 && endPos - currentPos > targetChunkSize * 1.5) {
      file.seekg(currentPos + targetChunkSize);
      std::string line;
      if (std::getline(file, line)) {
        endPos = file.tellg();
      }
    }

    chunk.end = endPos;
    chunks.push_back(chunk);
    createChunkFile(filePath, chunk, i);  // 创建chunk文件

    threadSafeOutput("Chunk " + std::to_string(i) + ": "
                     + std::to_string(chunk.start) + " - "
                     + std::to_string(chunk.end));

    currentPos = chunk.end;
  }

  return chunks;
}

std::vector<std::string> readFileChunk(const std::filesystem::path& filePath,
                                       const FileChunk& chunk)
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

  if (!buffer.empty()) {
    lines.push_back(std::move(buffer));
  }

  threadSafeOutput("Read " + std::to_string(bytesRead) + " bytes from chunk");
  return lines;
}

std::string processWord(const std::string& word)
{
  std::string processed = word;
  processed.erase(
      std::remove_if(processed.begin(),
                     processed.end(),
                     [](char c)
                     { return std::ispunct(static_cast<unsigned char>(c)); }),
      processed.end());
  std::transform(processed.begin(),
                 processed.end(),
                 processed.begin(),
                 [](unsigned char c) { return std::tolower(c); });
  return processed;
}

std::unordered_map<std::string, std::size_t> countWords(
    const std::vector<std::string>& lines, int threadId)
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

void writeResults(const std::filesystem::path& outputPath,
                  const std::unordered_map<std::string, std::size_t>& wordCount)
{
  // Only create directories if the output path has a parent path
  if (outputPath.has_parent_path()) {
    std::filesystem::create_directories(outputPath.parent_path());
  }

  std::vector<std::pair<std::string, std::size_t>> sortedWords(
      wordCount.begin(), wordCount.end());

  std::sort(sortedWords.begin(),
            sortedWords.end(),
            [](const auto& a, const auto& b)
            {
              // 如果两个字符串都可以转换为数字，按数字大小排序
              if (std::all_of(a.first.begin(), a.first.end(), ::isdigit)
                  && std::all_of(b.first.begin(), b.first.end(), ::isdigit))
              {
                return std::stoll(a.first) < std::stoll(b.first);
              }
              // 否则，按字典序排序
              return a.first < b.first;
            });

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

std::optional<std::string> processFile(std::string_view inputFile,
                                       std::string_view outputFile)
{
  try {
    const auto inputPath = std::filesystem::path(inputFile);
    const auto outputPath = std::filesystem::path(outputFile);

    auto start = std::chrono::high_resolution_clock::now();

    threadSafeOutput("Starting file processing");
    auto chunks = divideFileIntoChunks(inputPath, NUM_THREADS);
    std::vector<std::thread> threads;
    std::vector<std::unordered_map<std::string, std::size_t>> threadResults(chunks.size());

    for (std::size_t i = 0; i < chunks.size(); ++i) {
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

}  // namespace word_count