# EidosInsight 完整构建打包脚本
# 一次性完成：前端构建 → Rust 编译 → NSIS 安装包 → 便携版
# 用法: powershell -ExecutionPolicy Bypass -File build-all.ps1

param(
    [switch]$SkipBuild,      # 跳过编译步骤，仅打包
    [switch]$PortableOnly,   # 仅生成便携版
    [switch]$NsisOnly        # 仅生成 NSIS 安装包
)

$ErrorActionPreference = "Stop"
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " EidosInsight Full Build & Package" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Step 1: Build
if (!$SkipBuild) {
    Write-Host "[1/3] Building application..." -ForegroundColor Green
    
    # 前端构建
    Write-Host "   Building frontend (vite)..." -ForegroundColor Gray
    Push-Location "$PSScriptRoot"
    pnpm build 2>&1 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   Frontend build failed! Trying without type check..." -ForegroundColor Yellow
        npx vite build 2>&1 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
    }
    
    # Rust 编译 + NSIS 打包
    if (!$PortableOnly) {
        Write-Host "   Building Rust + NSIS installer (this takes ~3min)..." -ForegroundColor Gray
        pnpm tauri build 2>&1 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
    } else {
        Write-Host "   Building Rust binary only..." -ForegroundColor Gray
        Push-Location "$PSScriptRoot\src-tauri"
        cargo build --release 2>&1 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
        Pop-Location
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   Build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "   Build complete ✓" -ForegroundColor Green
} else {
    Write-Host "[1/3] Skipping build (using existing binaries)" -ForegroundColor Yellow
}

# Step 2: NSIS installer info
if (!$PortableOnly) {
    Write-Host "`n[2/3] NSIS Installer:" -ForegroundColor Green
    $nsisPath = Get-ChildItem "$PSScriptRoot\src-tauri\target\release\bundle\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($nsisPath) {
        $nsisSize = [math]::Round($nsisPath.Length / 1MB, 2)
        Write-Host "   Path: $($nsisPath.FullName)" -ForegroundColor White
        Write-Host "   Size: $nsisSize MB" -ForegroundColor White
        Write-Host "   Includes: WebView2 Offline Installer + VC++ Redistributable" -ForegroundColor Gray
    } else {
        Write-Host "   WARNING: NSIS installer not found" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n[2/3] Skipping NSIS (Portable only)" -ForegroundColor Yellow
}

# Step 3: Portable build
if (!$NsisOnly) {
    Write-Host "`n[3/3] Building portable package..." -ForegroundColor Green
    & powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\build-portable.ps1" 2>&1 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
} else {
    Write-Host "`n[3/3] Skipping portable (NSIS only)" -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " All builds complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan
