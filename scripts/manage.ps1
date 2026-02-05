#!/usr/bin/env pwsh
# TimerApp 项目管理脚本
# 用法: .\scripts\manage.ps1 [命令]

param(
    [Parameter(Position = 0)]
    [ValidateSet("dev", "build", "check", "clean", "test", "docs", "release", "help")]
    [string]$Command = "help"
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$TimerDir = Join-Path $ProjectRoot "timer"
$SrcTauriDir = Join-Path $TimerDir "src-tauri"
$DocsDir = Join-Path $ProjectRoot "docs"

# 颜色定义
$Colors = @{
    Green = "`e[32m"
    Yellow = "`e[33m"
    Red = "`e[31m"
    Blue = "`e[34m"
    Cyan = "`e[36m"
    Reset = "`e[0m"
}

function Write-Color($Text, $Color) {
    Write-Host "$($Colors[$Color])$Text$($Colors.Reset)"
}

function Show-Help {
    Write-Color "TimerApp 项目管理脚本" "Cyan"
    Write-Host ""
    Write-Color "用法: .\scripts\manage.ps1 [命令]" "Yellow"
    Write-Host ""
    Write-Color "可用命令:" "Blue"
    Write-Host "  dev     - 启动开发服务器"
    Write-Host "  build   - 构建项目"
    Write-Host "  check   - 检查代码 (check + clippy)"
    Write-Host "  test    - 运行测试"
    Write-Host "  clean   - 清理构建缓存"
    Write-Host "  docs    - 打开项目文档目录"
    Write-Host "  release - 构建发布版本"
    Write-Host "  help    - 显示此帮助信息"
    Write-Host ""
    Write-Color "示例:" "Blue"
    Write-Host "  .\scripts\manage.ps1 dev"
    Write-Host "  .\scripts\manage.ps1 check"
}

function Start-DevServer {
    Write-Color "🚀 启动开发服务器..." "Green"
    Set-Location $TimerDir
    npm run tauri dev
}

function Build-Project {
    Write-Color "🔨 构建项目..." "Green"
    Set-Location $SrcTauriDir
    cargo build
    Write-Color "✅ 构建完成" "Green"
}

function Build-Release {
    Write-Color "📦 构建发布版本..." "Green"
    Set-Location $TimerDir
    npm run tauri build
}

function Check-Code {
    Write-Color "🔍 检查代码..." "Yellow"
    Set-Location $SrcTauriDir

    Write-Color "  → cargo check" "Blue"
    cargo check 2>&1 | ForEach-Object {
        if ($_ -match "error") { Write-Color $_ "Red" }
        elseif ($_ -match "warning") { Write-Color $_ "Yellow" }
        else { Write-Host $_ }
    }

    Write-Color "  → cargo clippy" "Blue"
    cargo clippy -- -D warnings 2>&1 | ForEach-Object {
        if ($_ -match "error") { Write-Color $_ "Red" }
        elseif ($_ -match "warning") { Write-Color $_ "Yellow" }
        else { Write-Host $_ }
    }

    Write-Color "✅ 检查完成" "Green"
}

function Run-Tests {
    Write-Color "🧪 运行测试..." "Green"
    Set-Location $SrcTauriDir
    cargo test
}

function Clear-BuildCache {
    Write-Color "🧹 清理构建缓存..." "Yellow"
    Set-Location $SrcTauriDir

    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
        Write-Color "  ✓ 已删除 target/" "Green"
    }

    Set-Location $TimerDir
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-Color "  ✓ 已删除 dist/" "Green"
    }

    Write-Color "✅ 清理完成" "Green"
}

function Open-Docs {
    Write-Color "📚 打开项目文档..." "Green"
    if (Test-Path $DocsDir) {
        Start-Process explorer.exe $DocsDir
    } else {
        Write-Color "❌ 文档目录不存在" "Red"
    }
}

# 主逻辑
switch ($Command) {
    "dev" { Start-DevServer }
    "build" { Build-Project }
    "release" { Build-Release }
    "check" { Check-Code }
    "test" { Run-Tests }
    "clean" { Clear-BuildCache }
    "docs" { Open-Docs }
    "help" { Show-Help }
    default { Show-Help }
}

Set-Location $ProjectRoot
