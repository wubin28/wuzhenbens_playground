#include "lib.hpp"

auto main() -> int
{
  auto const lib = library {};

  return lib.name == "word_count_cpp" ? 0 : 1;
}
