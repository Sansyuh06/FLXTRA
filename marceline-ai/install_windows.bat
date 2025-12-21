@echo off
echo.
echo ========================================
echo   MARCELINE AI - Windows Installation
echo ========================================
echo.

:: Check for Python
where python >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Python not found! Please install Python 3.10+ from python.org
    pause
    exit /b 1
)

echo [INFO] Python found!

:: Check/install dependencies
echo [INFO] Installing Python dependencies...
pip install --upgrade pip >nul 2>&1
pip install SpeechRecognition requests >nul 2>&1

:: Try to install pyaudio
echo [INFO] Installing PyAudio (may take a moment)...
pip install pyaudio >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARN] PyAudio failed. Trying pipwin...
    pip install pipwin >nul 2>&1
    pipwin install pyaudio >nul 2>&1
)

:: Verify pyaudio
python -c "import pyaudio" >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [WARNING] PyAudio installation failed!
    echo Please download the wheel from:
    echo   https://www.lfd.uci.edu/~gohlke/pythonlibs/#pyaudio
    echo And install with: pip install PyAudio-xxx.whl
    echo.
)

echo.
echo ========================================
echo   Installation Complete!
echo ========================================
echo.
echo To run Marceline:
echo   python marceline_windows.py
echo.
echo Or double-click: run_marceline.bat
echo.

:: Create run script
echo @echo off > run_marceline.bat
echo cd /d "%~dp0" >> run_marceline.bat
echo set GEMINI_API_KEY=AIzaSyDV5mWyQ6Ux0rEgX_2d77BwiN5Bl6Y906k >> run_marceline.bat
echo python marceline_windows.py >> run_marceline.bat
echo pause >> run_marceline.bat

echo Created run_marceline.bat
echo.
pause
