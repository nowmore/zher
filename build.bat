@echo off
setlocal

set "FRONTEND_DIR=frontend"
set "BACKEND_DIR=backend"
set "BUILD_DIR=build"
set "RELEASE_DIR=release"

echo Building Frontend...
pushd %FRONTEND_DIR%
call npm install
if %errorlevel% neq 0 exit /b %errorlevel%
call npm run build
if %errorlevel% neq 0 exit /b %errorlevel%
popd

echo Building Backend...
if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"
pushd %BACKEND_DIR%
call cargo build --release --target-dir ..\%BUILD_DIR%
if %errorlevel% neq 0 exit /b %errorlevel%
popd

echo Preparing Release...
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"
copy "%BUILD_DIR%\release\zher.exe" "%RELEASE_DIR%\zher.exe" >nul

echo Build Complete! Release available at: %RELEASE_DIR%

endlocal
