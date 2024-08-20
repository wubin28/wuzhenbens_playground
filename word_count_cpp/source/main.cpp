#include <iostream>

#include "lib.hpp"

int main()
{
  constexpr std::string_view inputFile = "../input.txt";
  constexpr std::string_view outputFile = "./output.txt";

  std::cout << "Starting word count process" << std::endl;

  if (const auto error = word_count::processFile(inputFile, outputFile)) {
    std::cerr << "Error: " << *error << '\n';
    return 1;
  } else {
    std::cout << "Processing completed successfully.\n";
    return 0;
  }
}
// input.txt (Run command 'python3 generate_input.py' to generate the large input file.)
// Short line
// This is a much longer line that should be in its own chunk
// Another short line
//
// output:
// Starting word count process
// Starting file processing
// File size: 89 bytes
// Created chunk file: /Users/binwu/OOR-local/katas/wuzhenbens_playground/word_count_cpp/build/input_chunk_0.txt
// Chunk 0: 0 - 70
// Created chunk file: /Users/binwu/OOR-local/katas/wuzhenbens_playground/word_count_cpp/build/input_chunk_1.txt
// Chunk 1: 70 - 89
// Thread 1 started
// Thread 0 started
// Read 19 bytes from chunk
// Thread 1 read 1 lines
// Thread 1 finished processing 3 words
// Read 70 bytes from chunk
// Thread 0 read 2 lines
// Thread 0 finished processing 15 words
// All threads finished, merging results
// Results written to ./output.txt
// Total processing time: 2 ms
// Processing completed successfully.
//
// output.txt
// a: 1
// another: 1
// be: 1
// chunk: 1
// in: 1
// is: 1
// its: 1
// line: 3
// longer: 1
// much: 1
// own: 1
// short: 2
// should: 1
// that: 1
// this: 1