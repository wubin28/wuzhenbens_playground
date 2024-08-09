#include <algorithm>
#include <cctype>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_map>
#include <vector>

namespace
{
constexpr std::size_t BUFFER_SIZE = 8192;  // 8 KB buffer

[[nodiscard]] inline std::vector<std::string> readFileLines(
    const std::filesystem::path& filePath)
{
  std::ifstream file(filePath, std::ios::in | std::ios::binary);
  if (!file) {
    throw std::runtime_error("Unable to open file: " + filePath.string());
  }

  std::vector<std::string> lines;
  std::string buffer;
  buffer.reserve(BUFFER_SIZE);

  while (file) {
    char ch;
    file.read(&ch, 1);
    if (file.eof())
      break;

    if (ch == '\n') {
      lines.push_back(std::move(buffer));
      buffer.clear();
      buffer.reserve(BUFFER_SIZE);
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
    const std::vector<std::string>& lines) noexcept
{
  std::unordered_map<std::string, std::size_t> wordCount;
  for (const auto& line : lines) {
    std::istringstream iss(line);
    std::string word;
    while (iss >> word) {
      std::string processedWord = processWord(word);
      if (!processedWord.empty()) {
        ++wordCount[processedWord];
      }
    }
  }
  return wordCount;
}

inline void writeResults(
    const std::filesystem::path& outputPath,
    const std::unordered_map<std::string, std::size_t>& wordCount)
{
  std::ofstream outFile(outputPath);
  if (!outFile) {
    throw std::runtime_error("Unable to open output file: "
                             + outputPath.string());
  }

  for (const auto& [word, count] : wordCount) {
    outFile << word << ": " << count << '\n';
  }
}

[[nodiscard]] inline std::optional<std::string> processFile(
    std::string_view inputFile, std::string_view outputFile) noexcept
{
  try {
    const auto inputPath = std::filesystem::path(inputFile);
    const auto outputPath = std::filesystem::path(outputFile);

    const auto lines = readFileLines(inputPath);
    const auto wordCount = countWords(lines);
    writeResults(outputPath, wordCount);

    return std::nullopt;  // 成功时返回空的 optional
  } catch (const std::exception& e) {
    return e.what();  // 失败时返回错误信息
  }
}

int main()
{
  constexpr std::string_view inputFile = "../input.txt";
  constexpr std::string_view outputFile = "output.txt";

  if (const auto error = processFile(inputFile, outputFile)) {
    std::cerr << "Error: " << *error << '\n';
    return 1;
  } else {
    std::cout << "Processing completed successfully.\n";
    return 0;
  }
}