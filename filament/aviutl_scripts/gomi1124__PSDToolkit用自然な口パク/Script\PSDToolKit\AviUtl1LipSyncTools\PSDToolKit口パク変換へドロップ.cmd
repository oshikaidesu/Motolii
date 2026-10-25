@echo off
setlocal
chcp 65001 >nul
title PSDToolKit Natural LipSync Converter

if "%~1"=="" (
    echo Drop one or more .anm2/.obj2 files or definition folders onto this launcher.
    echo.
    if /I not "%PSD_TOOLKIT_CONVERTER_NO_PAUSE%"=="1" pause
    exit /b 2
)

"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0Invoke-DroppedDefinitionConversion.ps1" %*
set "EXIT_CODE=%ERRORLEVEL%"

echo.
if not "%EXIT_CODE%"=="0" echo Conversion failed with exit code %EXIT_CODE%.
if /I not "%PSD_TOOLKIT_CONVERTER_NO_PAUSE%"=="1" pause
exit /b %EXIT_CODE%
