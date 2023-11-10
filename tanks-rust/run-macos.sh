#!/bin/zsh

scriptdir=${0:a:h}

cargo build --manifest-path $scriptdir/game-module/crates/game_module/Cargo.toml

if [ $? -eq 0 ]; then
    mkdir -p $scriptdir/../arete/modules/
    cp $scriptdir/game-module/target/debug/libgame_module.dylib $scriptdir/../arete/modules
    cp -r $scriptdir/res $scriptdir/../arete/
    $scriptdir/../arete/game_client
fi
