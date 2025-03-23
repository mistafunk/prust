cmake_minimum_required(VERSION 3.24)

# for unknown reasons, I cannot achieve clean cxx flags in build.rs with cmake-rs
set(CMAKE_CXX_FLAGS "")
set(CMAKE_CXX_FLAGS_RELEASE "")
set(CMAKE_CXX_FLAGS_RELWITHDEBINFO "")
set(CMAKE_CXX_FLAGS_MINSIZEREL "")
set(CMAKE_CXX_FLAGS_DEBUG "")

project(prust-cpp-bindings CXX)

add_library(bindings STATIC bindings.cpp)
set_target_properties(bindings PROPERTIES
	CXX_STANDARD 17
	COMPILE_WARNING_AS_ERROR 0
	POSITION_INDEPENDENT_CODE 1)

target_compile_options(bindings PRIVATE
     $<$<OR:$<CXX_COMPILER_ID:Clang>,$<CXX_COMPILER_ID:AppleClang>,$<CXX_COMPILER_ID:GNU>>: -Wall -Wextra>)

find_package(prt CONFIG REQUIRED)
target_include_directories(bindings PRIVATE ${PRT_INCLUDE_PATH})
target_link_libraries(bindings PRIVATE prt)

install(TARGETS bindings)
