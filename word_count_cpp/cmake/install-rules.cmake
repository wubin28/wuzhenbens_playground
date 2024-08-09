install(
    TARGETS word_count_cpp_exe
    RUNTIME COMPONENT word_count_cpp_Runtime
)

if(PROJECT_IS_TOP_LEVEL)
  include(CPack)
endif()
