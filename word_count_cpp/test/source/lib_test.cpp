#include <filesystem>
#include <fstream>

#include "lib.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace
{

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
    tempFilePath =
        std::filesystem::temp_directory_path() / "test_read_file.txt";
  }

  void TearDown() override { std::filesystem::remove(tempFilePath); }

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
  word_count::FileChunk chunk {0, 0};

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
  word_count::FileChunk chunk {0,
                               static_cast<std::streamoff>(content.length())};

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
  word_count::FileChunk chunk {0,
                               static_cast<std::streamoff>(content.length())};

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
  word_count::FileChunk chunk {7, 19};  // 只读取 "Line 2\nLine 3\n"

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
  word_count::FileChunk chunk {0, 1000};  // 远大于文件实际大小

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
  word_count::FileChunk chunk {0, 100};

  // When & Then: 调用readFileChunk函数应抛出异常
  EXPECT_THROW(word_count::readFileChunk(nonExistentFile, chunk),
               std::runtime_error);
}

TEST_F(ReadFileChunkTest, ReadLargeFile)
{
  // Given: 一个大文件（超过BUFFER_SIZE）和覆盖整个文件的chunk
  std::string line = std::string(1000, 'a') + '\n';  // 1001字节的行
  std::string content;
  for (int i = 0; i < 10; ++i) {
    content += line;
  }
  createFileWithContent(content);
  word_count::FileChunk chunk {0,
                               static_cast<std::streamoff>(content.length())};

  // When: 调用readFileChunk函数
  auto lines = word_count::readFileChunk(tempFilePath, chunk);

  // Then: 应返回正确数量的行，每行内容正确
  ASSERT_EQ(lines.size(), 10);
  for (const auto& line : lines) {
    EXPECT_EQ(line, std::string(1000, 'a'));
  }
}

class ProcessWordTest : public ::testing::Test
{
protected:
};

TEST_F(ProcessWordTest, LowercaseWordRemainsUnchanged)
{
  // Given
  std::string input = "hello";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello");
}

TEST_F(ProcessWordTest, UppercaseWordIsConvertedToLowercase)
{
  // Given
  std::string input = "WORLD";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "world");
}

TEST_F(ProcessWordTest, MixedCaseWordIsConvertedToLowercase)
{
  // Given
  std::string input = "MiXeD";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "mixed");
}

TEST_F(ProcessWordTest, PunctuationIsRemoved)
{
  // Given
  std::string input = "hello!";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello");
}

TEST_F(ProcessWordTest, MultiplePunctuationMarksAreRemoved)
{
  // Given
  std::string input = "hello!!!";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello");
}

TEST_F(ProcessWordTest, PunctuationInMiddleOfWordIsRemoved)
{
  // Given
  std::string input = "he!llo";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello");
}

TEST_F(ProcessWordTest, EmptyStringReturnsEmptyString)
{
  // Given
  std::string input = "";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "");
}

TEST_F(ProcessWordTest, StringWithOnlyPunctuationReturnsEmptyString)
{
  // Given
  std::string input = "!!!";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "");
}

TEST_F(ProcessWordTest, WordWithNumbersRemainsUnchanged)
{
  // Given
  std::string input = "hello123";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello123");
}

TEST_F(ProcessWordTest, WordWithSpacesRemainsUnchanged)
{
  // Given
  std::string input = "hello world";

  // When
  std::string result = word_count::processWord(input);

  // Then
  EXPECT_EQ(result, "hello world");
}

class CountWordsTest : public ::testing::Test
{
protected:
  int threadId = 0;  // 使用固定的 threadId 进行测试
};

TEST_F(CountWordsTest, EmptyInputReturnsEmptyMap)
{
  // Given: 一个空的输入向量
  std::vector<std::string> input;

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回一个空的 unordered_map
  EXPECT_TRUE(result.empty());
}

TEST_F(CountWordsTest, SingleWordCountedCorrectly)
{
  // Given: 一个包含单个单词的输入向量
  std::vector<std::string> input = {"hello"};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回包含该单词计数为1的 unordered_map
  EXPECT_EQ(result.size(), 1);
  EXPECT_EQ(result["hello"], 1);
}

TEST_F(CountWordsTest, MultipleWordsCountedCorrectly)
{
  // Given: 一个包含多个单词的输入向量
  std::vector<std::string> input = {"hello world", "hello universe"};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回正确计数的 unordered_map
  EXPECT_EQ(result.size(), 3);
  EXPECT_EQ(result["hello"], 2);
  EXPECT_EQ(result["world"], 1);
  EXPECT_EQ(result["universe"], 1);
}

TEST_F(CountWordsTest, PunctuationRemovedAndLowercased)
{
  // Given: 一个包含带标点符号和大写字母的单词的输入向量
  std::vector<std::string> input = {"Hello!", "WORLD."};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回处理后的单词计数的 unordered_map
  EXPECT_EQ(result.size(), 2);
  EXPECT_EQ(result["hello"], 1);
  EXPECT_EQ(result["world"], 1);
}

TEST_F(CountWordsTest, EmptyWordsIgnored)
{
  // Given: 一个包含空字符串的输入向量
  std::vector<std::string> input = {"hello", "", "world", "  "};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回忽略空字符串后的单词计数的 unordered_map
  EXPECT_EQ(result.size(), 2);
  EXPECT_EQ(result["hello"], 1);
  EXPECT_EQ(result["world"], 1);
}

TEST_F(CountWordsTest, LargeInputHandledCorrectly)
{
  // Given: 一个包含大量重复单词的输入向量
  std::vector<std::string> input(10000, "test");

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回正确计数的 unordered_map
  EXPECT_EQ(result.size(), 1);
  EXPECT_EQ(result["test"], 10000);
}

TEST_F(CountWordsTest, MixedCaseWordsCountedAsSame)
{
  // Given: 一个包含不同大小写形式的相同单词的输入向量
  std::vector<std::string> input = {"Hello", "hElLo", "HELLO", "hello"};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回将所有形式视为同一单词的计数的 unordered_map
  EXPECT_EQ(result.size(), 1);
  EXPECT_EQ(result["hello"], 4);
}

TEST_F(CountWordsTest, WordsWithNumbersHandledCorrectly)
{
  // Given: 一个包含带数字的单词的输入向量
  std::vector<std::string> input = {"hello123", "world456", "hello123"};

  // When: 调用 countWords 函数
  auto result = word_count::countWords(input, threadId);

  // Then: 返回正确处理带数字单词的计数的 unordered_map
  EXPECT_EQ(result.size(), 2);
  EXPECT_EQ(result["hello123"], 2);
  EXPECT_EQ(result["world456"], 1);
}

class WriteResultsTest : public ::testing::Test
{
protected:
  void SetUp() override
  {
    tempDir = std::filesystem::temp_directory_path() / "writeResultsTest";
    std::filesystem::create_directories(tempDir);
  }

  void TearDown() override { std::filesystem::remove_all(tempDir); }

  std::filesystem::path tempDir;

  std::string readFile(const std::filesystem::path& path)
  {
    std::ifstream file(path);
    return std::string(std::istreambuf_iterator<char>(file),
                       std::istreambuf_iterator<char>());
  }
};

TEST_F(WriteResultsTest, EmptyMapWritesEmptyFile)
{
  // Given: 一个空的 unordered_map 和输出文件路径
  std::unordered_map<std::string, std::size_t> wordCount;
  auto outputPath = tempDir / "empty_output.txt";

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 生成一个空文件
  EXPECT_TRUE(std::filesystem::exists(outputPath));
  EXPECT_TRUE(std::filesystem::is_empty(outputPath));
}

TEST_F(WriteResultsTest, SingleWordWrittenCorrectly)
{
  // Given: 一个包含单个单词的 unordered_map 和输出文件路径
  std::unordered_map<std::string, std::size_t> wordCount = {{"hello", 1}};
  auto outputPath = tempDir / "single_word_output.txt";

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 文件内容正确
  std::string expectedContent = "hello: 1\n";
  EXPECT_EQ(readFile(outputPath), expectedContent);
}

TEST_F(WriteResultsTest, MultipleWordsWrittenInAlphabeticalOrder)
{
  // Given: 一个包含多个单词的 unordered_map 和输出文件路径
  std::unordered_map<std::string, std::size_t> wordCount = {
      {"world", 2}, {"hello", 1}, {"test", 3}};
  auto outputPath = tempDir / "multiple_words_output.txt";

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 文件内容正确且按字母顺序排序
  std::string expectedContent = "hello: 1\ntest: 3\nworld: 2\n";
  EXPECT_EQ(readFile(outputPath), expectedContent);
}

TEST_F(WriteResultsTest, LargeDataSetWrittenCorrectly)
{
  // Given: 一个包含大量数据的 unordered_map 和输出文件路径
  std::unordered_map<std::string, std::size_t> wordCount;
  for (int i = 0; i < 1000; ++i) {
    wordCount[std::to_string(i)] = i;
  }
  auto outputPath = tempDir / "large_dataset_output.txt";

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 文件存在且不为空
  EXPECT_TRUE(std::filesystem::exists(outputPath));
  EXPECT_FALSE(std::filesystem::is_empty(outputPath));

  // 验证文件的前几行和最后几行
  std::string content = readFile(outputPath);

  EXPECT_THAT(content, testing::StartsWith("0: 0\n1: 1\n2: 2\n3: 3\n4: 4\n"));
  EXPECT_THAT(
      content,
      testing::EndsWith("995: 995\n996: 996\n997: 997\n998: 998\n999: 999\n"));
}

TEST_F(WriteResultsTest, OverwriteExistingFile)
{
  // Given: 一个已存在的文件和新的 unordered_map
  auto outputPath = tempDir / "overwrite_test.txt";
  {
    std::ofstream file(outputPath);
    file << "This is existing content\n";
  }
  std::unordered_map<std::string, std::size_t> wordCount = {{"new", 1}};

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 文件内容被新内容覆盖
  std::string expectedContent = "new: 1\n";
  EXPECT_EQ(readFile(outputPath), expectedContent);
}

TEST_F(WriteResultsTest, HandleSpecialCharacters)
{
  // Given: 一个包含特殊字符的 unordered_map 和输出文件路径
  std::unordered_map<std::string, std::size_t> wordCount = {
      {"hello!", 1}, {"world?", 2}, {"test:", 3}};
  auto outputPath = tempDir / "special_chars_output.txt";

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 文件内容正确且包含特殊字符
  std::string expectedContent = "hello!: 1\ntest:: 3\nworld?: 2\n";
  EXPECT_EQ(readFile(outputPath), expectedContent);
}

TEST_F(WriteResultsTest, NonExistentDirectoryCreated)
{
  // Given: 一个不存在的目录路径
  auto nonExistentDir = tempDir / "non_existent_dir";
  auto outputPath = nonExistentDir / "output.txt";
  std::unordered_map<std::string, std::size_t> wordCount = {{"test", 1}};

  // When: 调用 writeResults 函数
  word_count::writeResults(outputPath, wordCount);

  // Then: 目录被创建，文件写入成功
  EXPECT_TRUE(std::filesystem::exists(nonExistentDir));
  EXPECT_TRUE(std::filesystem::exists(outputPath));
  std::string expectedContent = "test: 1\n";
  EXPECT_EQ(readFile(outputPath), expectedContent);
}

TEST_F(WriteResultsTest, ContentMatchesExpectedFormat)
{
  std::unordered_map<std::string, std::size_t> wordCount = {
      {"apple", 3}, {"banana", 2}, {"cherry", 1}};
  auto outputPath = tempDir / "format_test_output.txt";

  word_count::writeResults(outputPath, wordCount);

  std::string content = readFile(outputPath);

  // 使用正则表达式匹配器来验证内容格式
  EXPECT_THAT(content,
              testing::MatchesRegex("apple: 3\n"
                                    "banana: 2\n"
                                    "cherry: 1\n"));

  // 使用 Contains 匹配器来检查特定行的存在
  EXPECT_THAT(content, testing::HasSubstr(std::string("apple: 3")));
  EXPECT_THAT(content, testing::HasSubstr(std::string("banana: 2")));
  EXPECT_THAT(content, testing::HasSubstr(std::string("cherry: 1")));
}

}  // namespace