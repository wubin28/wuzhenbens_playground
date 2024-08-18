#pragma once

#include <filesystem>
#include <string>
#include <string_view>
#include <unordered_map>
#include <vector>

namespace word_count
{

struct FileChunk
{
  std::streamoff start;
  std::streamoff end;
};

std::vector<FileChunk> divideFileIntoChunks(
    const std::filesystem::path& filePath, std::size_t numChunks);
std::vector<std::string> readFileChunk(const std::filesystem::path& filePath,
                                       const FileChunk& chunk);
std::string processWord(const std::string& word);
std::unordered_map<std::string, std::size_t> countWords(
    const std::vector<std::string>& lines, int threadId);
void writeResults(
    const std::filesystem::path& outputPath,
    const std::unordered_map<std::string, std::size_t>& wordCount);

// 添加一个处理整个文件的函数
std::optional<std::string> processFile(std::string_view inputFile,
                                       std::string_view outputFile);

}  // namespace word_count