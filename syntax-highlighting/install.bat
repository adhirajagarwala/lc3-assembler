@echo off
REM ==============================================================================
REM LC-3 Assembly Syntax Highlighting Installer for Windows
REM ==============================================================================
REM Installs syntax highlighting for various editors on Windows
REM Usage: install.bat [editor]
REM   editor: vscode, vim, sublime, all (default: all)
REM ==============================================================================

setlocal enabledelayedexpansion

REM Script directory
set "SCRIPT_DIR=%~dp0"

REM Colors are not easily done in batch, so we use simple text
set "INSTALL_TARGET=%~1"
if "%INSTALL_TARGET%"=="" set "INSTALL_TARGET=all"

echo ========================================
echo LC-3 Syntax Highlighting Installer
echo ========================================
echo.

REM ==============================================================================
REM VS Code Installation
REM ==============================================================================
:install_vscode
if not "%INSTALL_TARGET%"=="vscode" if not "%INSTALL_TARGET%"=="all" goto :skip_vscode

echo [INFO] Installing VS Code syntax highlighting...

set "VSCODE_EXT_DIR=%USERPROFILE%\.vscode\extensions"
set "EXT_DIR=%VSCODE_EXT_DIR%\lc3-assembly-1.0.0"

if not exist "%VSCODE_EXT_DIR%" (
    echo [ERROR] VS Code extensions directory not found: %VSCODE_EXT_DIR%
    echo [ERROR] Please make sure VS Code is installed
    goto :skip_vscode
)

REM Create extension directory
if not exist "%EXT_DIR%" mkdir "%EXT_DIR%"

REM Copy extension files
copy /Y "%SCRIPT_DIR%vscode\lc3asm.tmLanguage.json" "%EXT_DIR%\" >nul
copy /Y "%SCRIPT_DIR%vscode\language-configuration.json" "%EXT_DIR%\" >nul
copy /Y "%SCRIPT_DIR%vscode\package.json" "%EXT_DIR%\" >nul
copy /Y "%SCRIPT_DIR%vscode\snippets.json" "%EXT_DIR%\" >nul

echo [SUCCESS] VS Code extension installed to: %EXT_DIR%
echo [INFO] Please reload VS Code (Ctrl+Shift+P -^> 'Reload Window')
echo.

if not "%INSTALL_TARGET%"=="all" goto :end

:skip_vscode

REM ==============================================================================
REM Vim Installation
REM ==============================================================================
:install_vim
if not "%INSTALL_TARGET%"=="vim" if not "%INSTALL_TARGET%"=="all" goto :skip_vim

echo [INFO] Installing Vim syntax highlighting...

set "VIM_DIR=%USERPROFILE%\.vim"
set "VIM_SYNTAX_DIR=%VIM_DIR%\syntax"
set "VIM_FTDETECT_DIR=%VIM_DIR%\ftdetect"

REM Create directories
if not exist "%VIM_SYNTAX_DIR%" mkdir "%VIM_SYNTAX_DIR%"
if not exist "%VIM_FTDETECT_DIR%" mkdir "%VIM_FTDETECT_DIR%"

REM Copy syntax file
copy /Y "%SCRIPT_DIR%vim\lc3asm.vim" "%VIM_SYNTAX_DIR%\" >nul

REM Create ftdetect file
echo au BufRead,BufNewFile *.asm set filetype=lc3asm > "%VIM_FTDETECT_DIR%\lc3asm.vim"

echo [SUCCESS] Vim syntax file installed
echo [INFO] Filetype detection: %VIM_FTDETECT_DIR%\lc3asm.vim
echo.

if not "%INSTALL_TARGET%"=="all" goto :end

:skip_vim

REM ==============================================================================
REM Sublime Text Installation
REM ==============================================================================
:install_sublime
if not "%INSTALL_TARGET%"=="sublime" if not "%INSTALL_TARGET%"=="all" goto :skip_sublime

echo [INFO] Installing Sublime Text syntax highlighting...

set "SUBLIME_DIR=%APPDATA%\Sublime Text\Packages\User"

if not exist "%SUBLIME_DIR%" (
    echo [WARNING] Sublime Text directory not found: %SUBLIME_DIR%
    echo [WARNING] Please install Sublime Text first or create the directory manually
    goto :skip_sublime
)

REM Copy syntax file
copy /Y "%SCRIPT_DIR%sublime\LC3.sublime-syntax" "%SUBLIME_DIR%\" >nul

echo [SUCCESS] Sublime Text syntax file installed to: %SUBLIME_DIR%
echo.

if not "%INSTALL_TARGET%"=="all" goto :end

:skip_sublime

REM ==============================================================================
REM Summary
REM ==============================================================================
:end

echo.
echo [SUCCESS] Installation complete!
echo [INFO] Open a .asm file to see syntax highlighting in action.
echo [INFO] Test file available at: %SCRIPT_DIR%test.asm
echo.

pause
