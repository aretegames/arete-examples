#!/bin/zsh

scriptdir=${0:a:h}

cmake $scriptdir -B$scriptdir/build
cmake --build $scriptdir/build --config Release

if [ $? -eq 0 ]; then
    cmake -E make_directory $scriptdir/../arete/modules
    cmake -E copy $scriptdir/build/libgame_module.dylib $scriptdir/../arete/modules
    cmake -E copy_directory_if_different $scriptdir/res $scriptdir/../arete/res
    $scriptdir/../arete/game_client
fi

