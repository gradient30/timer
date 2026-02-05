@echo off
chcp 65001 >nul
:: TimerApp 项目管理脚本
:: 用法: scripts\manage.bat [命令]

set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=help"

set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."
set "TIMER_DIR=%PROJECT_ROOT%\timer"
set "SRC_TAURI_DIR=%TIMER_DIR%\src-tauri"
set "DOCS_DIR=%PROJECT_ROOT%\docs"

if "%COMMAND%"=="dev" goto :dev
if "%COMMAND%"=="build" goto :build
if "%COMMAND%"=="check" goto :check
if "%COMMAND%"=="test" goto :test
if "%COMMAND%"=="clean" goto :clean
if "%COMMAND%"=="docs" goto :docs
if "%COMMAND%"=="release" goto :release
if "%COMMAND%"=="help" goto :help
goto :help

:dev
echo [92m🚀 启动开发服务器...[0m
cd /d "%TIMER_DIR%"
npm run tauri dev
goto :end

:build
echo [92m🔨 构建项目...[0m
cd /d "%SRC_TAURI_DIR%"
cargo build
echo [92m✅ 构建完成[0m
pause
goto :end

:release
echo [92m📦 构建发布版本...[0m
cd /d "%TIMER_DIR%"
npm run tauri build
goto :end

:check
echo [93m🔍 检查代码...[0m
cd /d "%SRC_TAURI_DIR%"
echo [94m  → cargo check[0m
cargo check
echo [94m  → cargo clippy[0m
cargo clippy
echo [92m✅ 检查完成[0m
pause
goto :end

:test
echo [92m🧪 运行测试...[0m
cd /d "%SRC_TAURI_DIR%"
cargo test
pause
goto :end

:clean
echo [93m🧹 清理构建缓存...[0m
cd /d "%SRC_TAURI_DIR%"
if exist "target" (
    rmdir /s /q "target"
    echo [92m  ✓ 已删除 target/[0m
)
cd /d "%TIMER_DIR%"
if exist "dist" (
    rmdir /s /q "dist"
    echo [92m  ✓ 已删除 dist/[0m
)
echo [92m✅ 清理完成[0m
pause
goto :end

:docs
echo [92m📚 打开项目文档...[0m
if exist "%DOCS_DIR%" (
    start "" "%DOCS_DIR%"
) else (
    echo [91m❌ 文档目录不存在[0m
)
goto :end

:help
echo [96mTimerApp 项目管理脚本[0m
echo.
echo [93m用法: scripts\manage.bat [命令][0m
echo.
echo [94m可用命令:[0m
echo   dev     - 启动开发服务器
echo   build   - 构建项目
echo   check   - 检查代码
echo   test    - 运行测试
echo   clean   - 清理构建缓存
echo   docs    - 打开项目文档目录
echo   release - 构建发布版本
echo   help    - 显示此帮助信息
echo.
echo [94m示例:[0m
echo   scripts\manage.bat dev
echo   scripts\manage.bat check

:end
cd /d "%PROJECT_ROOT%"
