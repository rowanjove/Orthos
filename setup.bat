@echo off
chcp 65001 >nul 2>&1

echo ============================================
echo  LintDrop - Build Environment Setup
echo ============================================
echo.

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [!] Admin privileges required. Requesting elevation...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

echo [*] Adding Windows 10 SDK via VS Build Tools installer...
echo [*] This may take a few minutes. Please wait...
echo.

set "INSTALLER=C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installershell.exe"
set "BUILDTOOLS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"

if exist "%INSTALLER%" (
    "%INSTALLER%" modify --installPath "%BUILDTOOLS%" --add Microsoft.VisualStudio.Component.Windows10SDK.19041 --quiet --wait
    echo.
    if %errorLevel% equ 0 (
        echo [OK] Windows 10 SDK installed successfully!
    ) else (
        echo [FAIL] Auto-install failed. Please install manually (see below).
    )
) else (
    echo [FAIL] VS Build Tools installer not found.
)

echo.
echo ============================================
echo  Manual install steps:
echo  1. Open Visual Studio Installer
echo  2. Click "Modify" on Build Tools 2022
echo  3. Go to "Individual components" tab
echo  4. Search "Windows 10 SDK"
echo  5. Check "Windows 10 SDK (10.0.19041.0)"
echo  6. Click "Modify" to install
echo ============================================
echo.
pause
