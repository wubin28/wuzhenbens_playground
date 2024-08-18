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
