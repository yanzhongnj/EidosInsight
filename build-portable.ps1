# EidosInsight 离线部署打包脚本
# 将应用 + WebView2 运行时 + VC++ 运行时 打包为便携版
# 用法: powershell -ExecutionPolicy Bypass -File build-portable.ps1

param(
    [string]$OutputDir = ".\dist-portable",
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

# 路径定义
$ProjectRoot = $PSScriptRoot
$ReleaseDir = Join-Path $ProjectRoot "src-tauri\target\release"
$ResourcesDir = Join-Path $ProjectRoot "src-tauri\resources"
$MainExe = Join-Path $ReleaseDir "eidosinsight.exe"
$NativeHostExe = Join-Path $ReleaseDir "eidosinsight-native-host.exe"

# 检查主程序是否存在
if (!(Test-Path $MainExe)) {
    Write-Host "ERROR: Main executable not found at $MainExe" -ForegroundColor Red
    Write-Host "Please run 'pnpm tauri build' first." -ForegroundColor Yellow
    exit 1
}

# 创建输出目录
$PortableDir = Join-Path $OutputDir "EidosInsight-v$Version-Portable"
if (Test-Path $PortableDir) { Remove-Item $PortableDir -Recurse -Force }
New-Item -ItemType Directory -Path $PortableDir -Force | Out-Null

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " EidosInsight v$Version Portable Build" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 1. 复制主程序
Write-Host "[1/5] Copying main executable..." -ForegroundColor Green
Copy-Item $MainExe (Join-Path $PortableDir "EidosInsight.exe")
$exeSize = [math]::Round((Get-Item $MainExe).Length / 1MB, 2)
Write-Host "   EidosInsight.exe ($exeSize MB)"

# 2. 复制 native messaging host（如果存在）
if (Test-Path $NativeHostExe) {
    Copy-Item $NativeHostExe (Join-Path $PortableDir "eidosinsight-native-host.exe")
    Write-Host "   eidosinsight-native-host.exe"
}

# 3. 复制 WebView2 离线安装包
Write-Host "[2/5] Copying WebView2 Runtime installer..." -ForegroundColor Green
$WebView2Installer = Join-Path $ResourcesDir "MicrosoftEdgeWebView2RuntimeInstaller.exe"
if (Test-Path $WebView2Installer) {
    $runtimeDir = Join-Path $PortableDir "runtime"
    New-Item -ItemType Directory -Path $runtimeDir -Force | Out-Null
    Copy-Item $WebView2Installer (Join-Path $runtimeDir "MicrosoftEdgeWebView2RuntimeInstaller.exe")
    $wv2Size = [math]::Round((Get-Item $WebView2Installer).Length / 1MB, 2)
    Write-Host "   MicrosoftEdgeWebView2RuntimeInstaller.exe ($wv2Size MB)"
} else {
    Write-Host "   WARNING: WebView2 installer not found" -ForegroundColor Yellow
}

# 4. 复制 VC++ Redistributable
Write-Host "[3/5] Copying VC++ Redistributable..." -ForegroundColor Green
$VcRedist = Join-Path $ResourcesDir "vc_redist.x64.exe"
if (Test-Path $VcRedist) {
    if (!(Test-Path (Join-Path $PortableDir "runtime"))) {
        New-Item -ItemType Directory -Path (Join-Path $PortableDir "runtime") -Force | Out-Null
    }
    Copy-Item $VcRedist (Join-Path $PortableDir "runtime\vc_redist.x64.exe")
    $vcSize = [math]::Round((Get-Item $VcRedist).Length / 1MB, 2)
    Write-Host "   vc_redist.x64.exe ($vcSize MB)"
} else {
    Write-Host "   WARNING: VC++ Redistributable not found" -ForegroundColor Yellow
}

# 5. 创建启动脚本（先安装运行时再启动应用）
Write-Host "[4/5] Creating launcher scripts..." -ForegroundColor Green

# 启动脚本 - 先检测运行时，缺失则安装，然后启动应用
$launcherContent = @'
@echo off
chcp 65001 >nul 2>&1
title EidosInsight Launcher

:: 检查是否以管理员身份运行（安装运行时可能需要）
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 正在请求管理员权限来安装运行时依赖...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

echo.
echo ========================================
echo   EidosInsight 环境检测与启动
echo ========================================
echo.

:: 检查并安装 VC++ Redistributable
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" /v Installed 2>nul | findstr /i "0x1" >nul
if %errorLevel% neq 0 (
    echo [检测] 未找到 Visual C++ 运行时，正在安装...
    if exist "%~dp0runtime\vc_redist.x64.exe" (
        "%~dp0runtime\vc_redist.x64.exe" /install /quiet /norestart
        echo [完成] Visual C++ 运行时安装完毕
    ) else (
        echo [警告] 未找到 vc_redist.x64.exe，跳过安装
    )
) else (
    echo [检测] Visual C++ 运行时已安装 ✓
)

:: 检查并安装 WebView2 Runtime
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv 2>nul >nul
if %errorLevel% neq 0 (
    reg query "HKCU\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv 2>nul >nul
)
if %errorLevel% neq 0 (
    echo [检测] 未找到 WebView2 运行时，正在安装...
    if exist "%~dp0runtime\MicrosoftEdgeWebView2RuntimeInstaller.exe" (
        "%~dp0runtime\MicrosoftEdgeWebView2RuntimeInstaller.exe" /install /silent
        echo [完成] WebView2 运行时安装完毕
    ) else (
        echo [警告] 未找到 WebView2 安装程序，跳过安装
    )
) else (
    echo [检测] WebView2 运行时已安装 ✓
)

echo.
echo [启动] 正在启动 EidosInsight...
start "" "%~dp0EidosInsight.exe"
exit /b
'@

[System.IO.File]::WriteAllText((Join-Path $PortableDir "Launch-EidosInsight.bat"), $launcherContent, [System.Text.Encoding]::Unicode)

# 创建卸载运行时脚本（可选）
$uninstallContent = @'
@echo off
chcp 65001 >nul 2>&1
echo.
echo ========================================
echo   EidosInsight 运行时卸载
echo ========================================
echo.
echo 此脚本将卸载由 EidosInsight 安装的运行时组件。
echo 注意：如果其他程序也在使用这些运行时，卸载可能影响其他程序。
echo.
set /p confirm=确定要卸载运行时吗？(y/N): 
if /i "%confirm%" neq "y" exit /b

echo.
echo [卸载] 正在卸载 WebView2 Runtime...
"C:\Program Files (x86)\Microsoft\EdgeUpdate\MicrosoftEdgeUpdate.exe" /uninstall appguid={F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
echo [完成] WebView2 Runtime 卸载完毕

echo.
echo [卸载] 正在卸载 VC++ Redistributable...
if exist "%~dp0runtime\vc_redist.x64.exe" (
    "%~dp0runtime\vc_redist.x64.exe" /uninstall /quiet
)
echo [完成] VC++ Redistributable 卸载完毕

echo.
echo 卸载完成。你可以安全删除此文件夹。
pause
'@

[System.IO.File]::WriteAllText((Join-Path $PortableDir "Uninstall-Runtimes.bat"), $uninstallContent, [System.Text.Encoding]::Unicode)

# 6. 创建 README
Write-Host "[5/5] Creating README..." -ForegroundColor Green
$readmeContent = @"
# EidosInsight v$Version 便携版

## 使用方法

1. 双击 **Launch-EidosInsight.bat**
2. 首次运行会自动检测并安装所需运行时（需要管理员权限）：
   - Microsoft Edge WebView2 Runtime
   - Microsoft Visual C++ Redistributable
3. 运行时安装完成后，应用会自动启动
4. 后续启动无需再次安装运行时，可直接运行 EidosInsight.exe

## 系统要求

- Windows 10 版本 1803 或更高（x64）
- 约 300MB 可用磁盘空间（含运行时安装包）

## 文件说明

- EidosInsight.exe — 主程序
- Launch-EidosInsight.bat — 自动检测/安装运行时并启动应用
- Uninstall-Runtimes.bat — 卸载由本程序安装的运行时组件
- runtime/ — 运行时安装包（首次安装后可删除以节省空间）

## 注意事项

- 首次安装运行时需要管理员权限
- runtime 文件夹在运行时安装成功后可以删除（节省约 290MB 空间）
- 删除整个文件夹即可卸载应用（数据存储在 %APPDATA%\com.eidosinsight.desktop）
"@

[System.IO.File]::WriteAllText((Join-Path $PortableDir "README.txt"), $readmeContent, [System.Text.Encoding]::Unicode)

# 统计输出
Write-Host "`n========================================" -ForegroundColor Cyan
$totalSize = [math]::Round((Get-ChildItem $PortableDir -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB, 2)
Write-Host " Portable build complete!" -ForegroundColor Green
Write-Host " Output: $PortableDir" -ForegroundColor White
Write-Host " Total size: $totalSize MB" -ForegroundColor White
Write-Host "========================================`n" -ForegroundColor Cyan

# 可选：创建 ZIP
if (Get-Command Compress-Archive -ErrorAction SilentlyContinue) {
    $zipPath = Join-Path $OutputDir "EidosInsight-v$Version-Portable.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    Compress-Archive -Path $PortableDir -DestinationPath $zipPath
    $zipSize = [math]::Round((Get-Item $zipPath).Length / 1MB, 2)
    Write-Host " ZIP archive: $zipPath ($zipSize MB)" -ForegroundColor White
}
