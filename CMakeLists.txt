cmake_minimum_required(VERSION 3.10)
project(vrisc)

set(EXECUTABLE_OUTPUT_PATH ${PROJECT_SOURCE_DIR}/bin/)
set(LIBRARY_OUTPUT_PATH ${PROJECT_SOURCE_DIR}/lib/)

add_subdirectory(vrisc)
add_subdirectory(libvcore)
