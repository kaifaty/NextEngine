@echo off
setlocal

cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust/Cargo was not found in PATH.
    echo Install the repository toolchain or start this file from a Rust-enabled environment.
    echo.
    pause
    exit /b 1
)

echo Starting Next Engine...
set "NEXT_ENGINE_PLAY_STATE=%~dp0.local\play-state-v3"
cargo run --release -p next_game --features desktop-sdl-ash -- --interactive --state-root "%NEXT_ENGINE_PLAY_STATE%" %*
set "NEXT_ENGINE_EXIT_CODE=%ERRORLEVEL%"

if not "%NEXT_ENGINE_EXIT_CODE%"=="0" (
    echo.
    echo Next Engine stopped with exit code %NEXT_ENGINE_EXIT_CODE%.
    echo The console will remain open so the error log can be inspected.
    echo.
    pause
)

exit /b %NEXT_ENGINE_EXIT_CODE%
