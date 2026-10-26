@echo off
rem Launch the game (can be run from any folder).
cd /d "%~dp0"
if not exist target\release\dungeon_making.exe (
  echo Not built yet. Run: cargo build --release
  pause
  exit /b 1
)
start "" target\release\dungeon_making.exe
