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