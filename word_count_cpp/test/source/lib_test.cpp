#include <filesystem>
#include <fstream>

#include "lib.hpp"

#include <gtest/gtest.h>

class DivideFileIntoChunksTest : public ::testing::Test
{
protected:
  void SetUp() override
  {
    // 创建一个临时文件用于测试
    tempFilePath = std::filesystem::temp_directory_path() / "test_file.txt";
  }

  void TearDown() override
  {
    // 删除临时文件
    std::filesystem::remove(tempFilePath);
  }

  void createFileWithSize(std::size_t size)
  {
    std::ofstream file(tempFilePath, std::ios::binary);
    file.seekp(size - 1);
    file.write("", 1);
  }

  std::filesystem::path tempFilePath;
};

TEST_F(DivideFileIntoChunksTest, DivideEmptyFileIntoOneChunk)
{
  // Given: 一个空文件和1个分块
  createFileWithSize(0);
  std::size_t numChunks = 1;

  // When: 调用divideFileIntoChunks函数
  auto chunks = word_count::divideFileIntoChunks(tempFilePath, numChunks);

  // Then: 应返回一个包含单个空块的vector
  ASSERT_EQ(chunks.size(), 1);
  EXPECT_EQ(chunks[0].start, 0);
  EXPECT_EQ(chunks[0].end, 0);
}

TEST_F(DivideFileIntoChunksTest, DivideNonEmptyFileIntoMultipleEqualChunks)
{
  // Given: 一个100字节的文件和4个分块
  std::size_t fileSize = 100;
  createFileWithSize(fileSize);
  std::size_t numChunks = 4;

  // When: 调用divideFileIntoChunks函数
  auto chunks = word_count::divideFileIntoChunks(tempFilePath, numChunks);

  // Then: 应返回4个大小相等的块
  ASSERT_EQ(chunks.size(), numChunks);
  for (std::size_t i = 0; i < numChunks; ++i) {
    EXPECT_EQ(chunks[i].start, i * 25);
    EXPECT_EQ(chunks[i].end, (i == numChunks - 1) ? fileSize : (i + 1) * 25);
  }
}

TEST_F(DivideFileIntoChunksTest, DivideFileIntoMoreChunksThanFileSize)
{
  // Given: 一个5字节的文件和10个分块
  std::size_t fileSize = 5;
  createFileWithSize(fileSize);
  std::size_t numChunks = 10;

  // When: 调用divideFileIntoChunks函数
  auto chunks = word_count::divideFileIntoChunks(tempFilePath, numChunks);

  // Then: 应返回10个块，每个块大小为0或1字节
  ASSERT_EQ(chunks.size(), numChunks);
  std::size_t totalSize = 0;
  for (const auto& chunk : chunks) {
    EXPECT_GE(chunk.end, chunk.start);
    totalSize += (chunk.end - chunk.start);
  }
  EXPECT_EQ(totalSize, fileSize);
}

TEST_F(DivideFileIntoChunksTest, DivideFileSmallerThanChunkCount)
{
  // Given: 一个3字节的文件和4个分块
  std::size_t fileSize = 3;
  createFileWithSize(fileSize);
  std::size_t numChunks = 4;

  // When: 调用divideFileIntoChunks函数
  auto chunks = word_count::divideFileIntoChunks(tempFilePath, numChunks);

  // Then: 应返回4个块，总大小等于文件大小
  ASSERT_EQ(chunks.size(), numChunks);
  std::size_t totalSize = 0;
  for (const auto& chunk : chunks) {
    EXPECT_GE(chunk.end, chunk.start);
    totalSize += (chunk.end - chunk.start);
  }
  EXPECT_EQ(totalSize, fileSize);
}

TEST_F(DivideFileIntoChunksTest, ThrowExceptionForNonExistentFile)
{
  // Given: 一个不存在的文件路径
  std::filesystem::path nonExistentFile = "non_existent_file.txt";

  // When & Then: 调用divideFileIntoChunks函数应抛出异常
  EXPECT_THROW(word_count::divideFileIntoChunks(nonExistentFile, 1),
               std::runtime_error);
}

TEST_F(DivideFileIntoChunksTest, DivideFileIntoZeroChunksThrowsException)
{
  // Given: 一个有效文件和0个分块
  createFileWithSize(100);
  std::size_t numChunks = 0;

  // When & Then: 调用divideFileIntoChunks函数应抛出异常
  EXPECT_THROW(word_count::divideFileIntoChunks(tempFilePath, numChunks),
               std::invalid_argument);
}

class ReadFileChunkTest : public ::testing::Test
{
protected:
  void SetUp() override
  {
    tempFilePath = std::filesystem::temp_directory_path() / "test_read_file.txt";
  }

  void TearDown() override
  {
    std::filesystem::remove(tempFilePath);
  }

  void createFileWithContent(const std::string& content)
  {
    std::ofstream file(tempFilePath);
    file << content;
  }

  std::filesystem::path tempFilePath;
};

TEST_F(ReadFileChunkTest, ReadEmptyFile)
{
  // Given: 一个空文件和覆盖整个文件的chunk
  createFileWithContent("");
  word_count::FileChunk chunk{0, 0};

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回一个空vector
  EXPECT_TRUE(lines.empty());
}

TEST_F(ReadFileChunkTest, ReadSingleLineFile)
{
  // Given: 一个只有一行的文件和覆盖整个文件的chunk
  std::string content = "This is a single line.";
  createFileWithContent(content);
  word_count::FileChunk chunk{0, static_cast<std::streamoff>(content.length())};

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回一个包含单个元素的vector，元素内容与文件内容相同
  ASSERT_EQ(lines.size(), 1);
  EXPECT_EQ(lines[0], content);
}

TEST_F(ReadFileChunkTest, ReadMultiLineFile)
{
  // Given: 一个多行文件和覆盖整个文件的chunk
  std::string content = "Line 1\nLine 2\nLine 3\n";
  createFileWithContent(content);
  word_count::FileChunk chunk{0, static_cast<std::streamoff>(content.length())};

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回一个包含3个元素的vector，每个元素对应一行
  ASSERT_EQ(lines.size(), 3);
  EXPECT_EQ(lines[0], "Line 1");
  EXPECT_EQ(lines[1], "Line 2");
  EXPECT_EQ(lines[2], "Line 3");
}

TEST_F(ReadFileChunkTest, ReadPartialFile)
{
  // Given: 一个多行文件和只覆盖部分文件的chunk
  std::string content = "Line 1\nLine 2\nLine 3\nLine 4\n";
  createFileWithContent(content);
  word_count::FileChunk chunk{7, 19}; // 只读取 "Line 2\nLine 3\n"

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回一个包含2个元素的vector，对应chunk覆盖的两行
  ASSERT_EQ(lines.size(), 2);
  EXPECT_EQ(lines[0], "Line 2");
  EXPECT_EQ(lines[1], "Line 3");
}

TEST_F(ReadFileChunkTest, ReadBeyondFileSize)
{
  // Given: 一个文件和一个超过文件大小的chunk
  std::string content = "This is a test file.";
  createFileWithContent(content);
  word_count::FileChunk chunk{0, 1000}; // 远大于文件实际大小

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回包含整个文件内容的vector
  ASSERT_EQ(lines.size(), 1);
  EXPECT_EQ(lines[0], content);
}

TEST_F(ReadFileChunkTest, ReadNonExistentFile)
{
  // Given: 一个不存在的文件路径
  std::filesystem::path nonExistentFile = "non_existent_file.txt";
  word_count::FileChunk chunk{0, 100};

  // When & Then: 调用readFileChunk函数应抛出异常
  EXPECT_THROW(word_count::readFileChunk(nonExistentFile, chunk), std::runtime_error);
}

TEST_F(ReadFileChunkTest, ReadLargeFile)
{
  // Given: 一个大文件（超过BUFFER_SIZE）和覆盖整个文件的chunk
  std::string line = std::string(1000, 'a') + '\n'; // 1001字节的行
  std::string content;
  for (int i = 0; i < 10; ++i) {
    content += line;
  }
  createFileWithContent(content);
  word_count::FileChunk chunk{0, static_cast<std::streamoff>(content.length())};

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回正确数量的行，每行内容正确
  ASSERT_EQ(lines.size(), 10);
  for (const auto& line : lines) {
    EXPECT_EQ(line, std::string(1000, 'a'));
  }
}