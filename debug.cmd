@echo off
rem Test-play mode: press F1 for the debug panel. Saves go to savedata_debug.json.
cd /d "%~dp0"
if not exist target\release\dungeon_making.exe (
  echo Not built yet. Run: cargo build --release
  pause
  exit /b 1
)
start "" target\release\dungeon_making.exe --debug
