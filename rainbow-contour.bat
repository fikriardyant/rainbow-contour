@echo off
setlocal

:: Get directory of this script
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

:: Check for pre-built executable
if exist "%SCRIPT_DIR%rainbow-contour.exe" (
    "%SCRIPT_DIR%rainbow-contour.exe" %*
    goto :end
)

if exist "%SCRIPT_DIR%target\release\rainbow-contour.exe" (
    "%SCRIPT_DIR%target\release\rainbow-contour.exe" %*
    goto :end
)

if exist "%SCRIPT_DIR%target\debug\rainbow-contour.exe" (
    "%SCRIPT_DIR%target\debug\rainbow-contour.exe" %*
    goto :end
)

:: Fallback to cargo if binary not found but rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% equ 0 (
    cargo run --release -- %*
    goto :end
)

echo [ERROR] rainbow-contour.exe tidak ditemukan!
echo Pastikan Anda telah mendownload file release atau meletakkan rainbow-contour.exe di folder ini.
pause

:end
endlocal
