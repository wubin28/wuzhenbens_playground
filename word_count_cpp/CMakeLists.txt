cmake_minimum_required(VERSION 3.14)

include(cmake/prelude.cmake)

project(
    word_count_cpp
    VERSION 0.1.0
    DESCRIPTION "Short description"
    HOMEPAGE_URL "https://example.com/"
    LANGUAGES CXX
)

# Enable testing at the top level
enable_testing()

# Add subdirectory for tests if needed
add_subdirectory(test)

include(cmake/project-is-top-level.cmake)
include(cmake/variables.cmake)

# ---- Declare library ----

add_library(
    word_count_cpp_lib OBJECT
    source/lib.cpp
)

target_include_directories(
    word_count_cpp_lib ${warning_guard}
    PUBLIC
    "\$<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/source>"
)

target_compile_features(word_count_cpp_lib PUBLIC cxx_std_20)

# ---- Declare executable ----

add_executable(word_count_cpp_exe source/main.cpp)
add_executable(word_count_cpp::exe ALIAS word_count_cpp_exe)

set_property(TARGET word_count_cpp_exe PROPERTY OUTPUT_NAME word_count_cpp)

target_compile_features(word_count_cpp_exe PRIVATE cxx_std_20)

target_link_libraries(word_count_cpp_exe PRIVATE word_count_cpp_lib)

# ---- Install rules ----

if(NOT CMAKE_SKIP_INSTALL_RULES)
  include(cmake/install-rules.cmake)
endif()

# ---- Developer mode ----

if(NOT word_count_cpp_DEVELOPER_MODE)
  return()
elseif(NOT PROJECT_IS_TOP_LEVEL)
  message(
      AUTHOR_WARNING
      "Developer mode is intended for developers of word_count_cpp"
  )
endif()

include(cmake/dev-mode.cmake)
