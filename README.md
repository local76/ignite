rStartup Windows Startup Application Manager

rsta is a lightweight, responsive, and premium terminal user interface custom-tailored for Windows to audit, configure, and clean up startup programs.


Core System Specifications

Unified Design System: Displays App Name, user, host, and OS Version in Gold or Windows DWM Accent colors.
Console Tab Title Guard: Sets the console tab title to rSta on launch and restores the original terminal tab title on exit.
Auto-Scaling Accent Theme: Dynamically queries the Windows DWM registry to color active panels and headers.
Adaptive Glyph Rendering: Detects modern terminal emulators like Windows Terminal vs legacy conhost.exe to prevent broken box-drawing characters.
Keyboard Shortcuts: Pressing h opens the help modal showing current navigation overlays.
Event Log Integration: Synchronizes application diagnostics directly with the native Windows Event Log.


Audit Locations

Registry Run Entries: Scans HKCU and HKLM Run and RunOnce hives, including 32-bit WOW6432Node compatibility paths.
Startup Directories: Monitors user-specific and system-wide Startup folder directories.
Scheduled Logon Tasks: Audits task scheduler entries configured to execute when a user logs on.


Control Actions

Toggle Status: Press Space to disable or enable startup applications. This uses the registry StartupApproved keys or task states to toggle loading without deleting the source paths.
Delete Entry: Press Delete to completely remove startup configurations.
Add Entry: Press a to open a prompt to add a new program or script path to the registry startup keys.
