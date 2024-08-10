# word_count_cpp

This is the word_count_cpp project.

# function diagram

```mermaid
%%{init: {
  'theme': 'dark',
  'themeVariables': {
    'primaryColor': '#BB2528',
    'primaryTextColor': '#fff',
    'primaryBorderColor': '#7C0000',
    'lineColor': '#F8B229',
    'secondaryColor': '#006100',
    'tertiaryColor': '#00A86B'
  }
}}%%

graph TD
    A[main] -->|calls| B(processFile)
    B -->|calls| C(readFileLines)
    B -->|calls| D(countWords)
    B -->|calls| E(writeResults)
    D -->|calls| F(processWord)

    A["main
    功能: 程序入口点
输入: 无
输出: int (退出状态)"]

B["processFile
功能: 处理文件的主要逻辑
输入: string_view inputFile, string_view outputFile
输出: optional<string> (错误信息)"]

C["readFileLines
功能: 读取文件内容
输入: filesystem::path filePath
输出: vector<string> (文件行)"]

D["countWords
功能: 统计单词出现次数
输入: vector<string> lines
输出: unordered_map<string, size_t> (单词计数)"]

E["writeResults
功能: 将结果写入输出文件
输入: filesystem::path outputPath, unordered_map<string, size_t> wordCount
输出: void"]

F["processWord
功能: 处理单个单词 (去除标点, 转小写)
输入: string word
输出: string (处理后的单词)"]

classDef default fill:#2C3E50,stroke:#E74C3C,stroke-width:2px,color:#ECF0F1;
classDef main fill:#34495E,stroke:#F39C12,stroke-width:4px,color:#ECF0F1;
class A main;
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
