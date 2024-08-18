# Parent project does not export its library target, so this CML implicitly
# depends on being added from it, i.e. the testing is done only from the build
# tree and is not feasible from an install location

project(word_count_cppTests LANGUAGES CXX)

# ---- Enable testing ----
enable_testing()

# ---- Include GoogleTest ----
# Manually specify the include directory for GoogleTest
include_directories(/opt/homebrew/opt/googletest/include)

# Manually specify the library directory for GoogleTest
link_directories(/opt/homebrew/opt/googletest/lib)

# Now find the GTest package
find_package(GTest REQUIRED)

# Add the executable for the tests
add_executable(lib_test source/lib_test.cpp)

# Link the GoogleTest libraries
target_link_libraries(lib_test PRIVATE word_count_cpp_lib GTest::GTest GTest::Main pthread)

# Enable C++20 standard
target_compile_features(lib_test PRIVATE cxx_std_20)

# Register the test with CTest
add_test(NAME lib_test COMMAND lib_test)# ---- End-of-file commands ----
# add_folders(Test)