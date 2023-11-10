#!/bin/zsh

scriptdir=${0:a:h}

cmake $scriptdir -B$scriptdir/build-ios -DCMAKE_SYSTEM_NAME=iOS
cmake --build $scriptdir/build-ios --target game_module --config Release

if [ $? -eq 0 ]; then
    cmake -E make_directory $scriptdir/../arete-ios/module
    cmake -E copy $scriptdir/build-ios/libgame_module.a $scriptdir/../arete-ios/module
    cmake -E copy_directory_if_different $scriptdir/res $scriptdir/../arete-ios/res
fi
