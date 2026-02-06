#!/usr/bin/env pwsh

param(
    [Parameter(Position = 0)]
    [int]$Count = 1
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ManifestPath = Join-Path -Path $ProjectRoot -ChildPath "timer\src-tauri\Cargo.toml"

if ($Count -lt 1) {
    Write-Host "Count 必须为正整数" -ForegroundColor Red
    exit 1
}

cargo run --bin activation_gen --manifest-path $ManifestPath -- $Count
