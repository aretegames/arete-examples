@echo off

cmake -Bbuild -DCMAKE_WINDOWS_EXPORT_ALL_SYMBOLS=TRUE -DBUILD_SHARED_LIBS=TRUE
cmake --build build --config Release

if errorlevel 1 goto failed
    xcopy /y build\Release\game_module.dll ..\arete\modules\
    ..\arete\game_client.exe
:failed
