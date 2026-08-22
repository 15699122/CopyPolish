@echo off
rem ============================================================
rem  Build a Windows one-file EXE for the Chinese Copywriting
rem  Formatter (customtkinter version: engine + GUI + rules.yaml).
rem
rem  Prereqs: Windows 10/11, python with pip, network for PyInstaller.
rem  Run this .bat from the project root (chinese_copywriting_formatter/).
rem ============================================================
setlocal
cd /d "%~dp0.."

echo Installing / updating PyInstaller...
python -m pip install -U pyinstaller
if errorlevel 1 goto :err

echo Building one-file windowed exe (bundles tcl/tk, customtkinter, engine, rules.yaml)...
python -m PyInstaller ^
  --onefile ^
  --windowed ^
  --name chinese_copywriting_formatter ^
  --collect-tcl-data ^
  --collect-data customtkinter ^
  --add-data "rules.yaml;." ^
  chinese_copywriting_formatter.py
if errorlevel 1 goto :err2

echo Copying exe to build\windows\...
if not exist build\windows mkdir build\windows
move /y dist\chinese_copywriting_formatter.exe build\windows\chinese_copywriting_formatter.exe
if errorlevel 1 goto :err3

echo.
echo SUCCESS: build\windows\chinese_copywriting_formatter.exe
pause
exit /b 0

:err
echo Python or pip failed. Make sure Python is on PATH.
pause
exit /b 1

:err2
echo PyInstaller build failed; see messages above.
pause
exit /b 1

:err3
echo Could not move exe to build\windows; it is in dist\ instead.
pause
exit /b 1
