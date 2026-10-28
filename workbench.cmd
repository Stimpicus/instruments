@echo off
rem Build and start the listening workbench, then open it in the browser.
cd /d "%~dp0"
start "" cmd /c "timeout /t 4 >nul & start http://127.0.0.1:8765"
cargo run --release --bin workbench
