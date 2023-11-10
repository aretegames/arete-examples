#!/bin/bash

cargo build --manifest-path game-module/crates/game_module/Cargo.toml

if [ $? -eq 0 ]; then
    mkdir -p ../arete/modules/
    cp game-module/target/debug/libgame_module.so ../arete/modules
    mkdir -p ../arete/res/
    cp -r res ../arete/res/
    ../arete/game_client
fi
