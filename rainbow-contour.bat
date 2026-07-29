@echo off
echo ======================================================================
echo   RAINBOW CONTOUR - Cut ^& Fill Difference Map Generator v1.0
echo   PAMA Mine Engineering Field Launcher
echo ======================================================================

if not exist "output" mkdir output

if exist "target\release\rainbow-contour.exe" (
    target\release\rainbow-contour.exe %*
) else (
    echo Building release binary...
    cargo build --release
    target\release\rainbow-contour.exe %*
)
