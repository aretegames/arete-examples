cmake_minimum_required(VERSION 3.16)

project(AreteGame)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

if(WIN32)
  add_custom_command(
    OUTPUT ffi.cpp
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/src/game.cpp
    COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/../arete-cpp/arete-codegen.exe
      -i ${CMAKE_CURRENT_SOURCE_DIR}/src/game.cpp
      -o ${CMAKE_CURRENT_BINARY_DIR}/ffi.cpp
    VERBATIM
  )
else()
  add_custom_command(
    OUTPUT ffi.cpp
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/src/game.cpp
    COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/../arete-cpp/arete-codegen
      -i ${CMAKE_CURRENT_SOURCE_DIR}/src/game.cpp
      -o ${CMAKE_CURRENT_BINARY_DIR}/ffi.cpp
    VERBATIM
  )
endif()

if(IOS)
  add_library(game_module STATIC ${CMAKE_CURRENT_BINARY_DIR}/ffi.cpp)
else()
  add_library(game_module SHARED ${CMAKE_CURRENT_BINARY_DIR}/ffi.cpp)
endif()

target_include_directories(game_module
  PRIVATE
    src
    ../arete-cpp/include
)
