cmake_minimum_required(VERSION 3.14)

project(word_count_cppTests LANGUAGES CXX)

# ---- Dependencies ----

include(FetchContent)
FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG release-1.12.1
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

# ---- Tests ----

enable_testing()

add_executable(lib_test source/lib_test.cpp)
target_link_libraries(lib_test
        PRIVATE
        word_count_cpp::lib
        GTest::gtest_main
        gmock
)

target_compile_features(lib_test PRIVATE cxx_std_20)

# Add compiler warnings
target_compile_options(lib_test
        PRIVATE
        $<$<CXX_COMPILER_ID:MSVC>:/W4 /WX>
        $<$<NOT:$<CXX_COMPILER_ID:MSVC>>:-Wall -Wextra -Wpedantic -Werror>
)

include(GoogleTest)
gtest_discover_tests(lib_test)