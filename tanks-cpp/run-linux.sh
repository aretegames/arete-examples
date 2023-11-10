#!/bin/bash

cmake -Bbuild
cmake --build build --config Release

if [ $? -eq 0 ]; then
    cmake -E make_directory ../arete/modules
    cmake -E copy build/libgame_module.so ../arete/modules
    cmake -E copy_directory_if_different res ../arete/res
    ../arete/game_client
fi

