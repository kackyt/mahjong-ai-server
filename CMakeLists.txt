cmake_minimum_required(VERSION 3.18)
project(loader VERSION 1.0 LANGUAGES C)

add_subdirectory(./loadlibrary/peloader)
add_subdirectory(./loadlibrary/intercept/libdisasm)

add_executable(loader loader.c)

target_compile_options(loader PRIVATE
  "-march=native"
  "-ggdb3"
  "-m32"
  "-std=gnu99"
  "-fshort-wchar"
  "-Wno-multichar"
  "-mstackrealign"
)

target_link_options(loader PRIVATE
  "-m32"
  "-lm"
)

target_compile_definitions(loader PRIVATE -D_GNU_SOURCE)

target_include_directories(loader PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/loadlibrary/include)
target_link_libraries(loader PUBLIC "$<LINK_LIBRARY:WHOLE_ARCHIVE,peloader>")
target_link_libraries(loader PUBLIC disasm)
