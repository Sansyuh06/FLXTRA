@echo off
echo.
echo ========================================
echo  Starting Marceline AI Assistant
echo ========================================
echo.

cd /d "%~dp0marceline-ai"

:: Check if node_modules exist
if not exist "backend\node_modules" (
    echo Installing backend dependencies...
    cd backend
    call npm install
    cd ..
)

if not exist "frontend\node_modules" (
    echo Installing frontend dependencies...
    cd frontend
    call npm install
    cd ..
)

if not exist "node_modules" (
    echo Installing root dependencies...
    call npm install
)

echo.
echo Starting servers...
echo.
echo Backend: http://localhost:3001
echo Frontend: http://localhost:3000
echo.
echo Press Ctrl+C to stop
echo.

:: Start both servers
call npm run dev
