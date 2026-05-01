@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" amd64
cd /d "F:\AI\vibecoding\format-checker\lintdrop\src-tauri"
cargo check 2>&1
echo EXIT_CODE=%ERRORLEVEL%
