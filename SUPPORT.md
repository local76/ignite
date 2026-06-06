Application Support

Thank you for using rStartup (Windows Startup Application Manager)! If you are experiencing issues, follow these steps to get help.

Run rStartup Doctor Self-Healing Diagnostics

Before filing an issue, check if the diagnostic command can detect configuration or environmental problems. Open your terminal and run:

rsta doctor

Check the Logs

rStartup logs events and diagnostics to a background log file.

Log Location: %APPDATA%\rStartup\log.txt
How to open in PowerShell:
notepad "$env:APPDATA\rStartup\log.txt"

Open an Issue

If the doctor tool did not resolve your issue, please open an issue in the official repository.

What to include:
Your Windows version (e.g. Windows 11 23H2).
The terminal environment you are using.
The relevant output or error logs from %APPDATA%\rStartup\log.txt.

