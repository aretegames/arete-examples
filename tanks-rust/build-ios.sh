#!/bin/zsh

cargo rustc --release --manifest-path game-module/crates/game_module/Cargo.toml --crate-type=staticlib --target=aarch64-apple-ios

if [ $? -eq 0 ]; then
    mkdir -p ../arete-ios/module/
    cp game-module/target/aarch64-apple-ios/release/libgame_module.a ../arete-ios/module
fi
