@echo off

cargo build --manifest-path game-module/crates/game_module/Cargo.toml

if errorlevel 1 goto failed
    xcopy /y game-module\target\debug\game_module.dll ..\arete\modules\
    ..\arete\game_client.exe
:failed
