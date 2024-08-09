# Parent project does not export its library target, so this CML implicitly
# depends on being added from it, i.e. the testing is done only from the build
# tree and is not feasible from an install location

project(word_count_cppTests LANGUAGES CXX)

# ---- Tests ----

add_executable(word_count_cpp_test source/word_count_cpp_test.cpp)
target_link_libraries(word_count_cpp_test PRIVATE word_count_cpp_lib)
target_compile_features(word_count_cpp_test PRIVATE cxx_std_20)

add_test(NAME word_count_cpp_test COMMAND word_count_cpp_test)

# ---- End-of-file commands ----

add_folders(Test)
