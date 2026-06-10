//! `ignite --doctor` diagnostic command.

use library::platform::native::sys_info::{GlyphMap, query_os_version};

pub fn run() {
    println!("===================================================");
    println!("             ignite Diagnostic Doctor             ");
    println!("===================================================\n");

    let glyphs = GlyphMap::load();

    // 1. Check OS
    let os = query_os_version();
    println!("{} OS: {}", glyphs.info, os);

    // 2. Check Registry / Startup Paths Access
    println!("\nChecking Registry Startup Keys Access...");
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        let hkcu_run_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let hklm_run_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        
        match hkcu.open_subkey(hkcu_run_path) {
            Ok(_) => println!("{} HKCU Run Key: Accessible (Read)", glyphs.status_ok),
            Err(e) => println!("{} HKCU Run Key: Failed ({})", glyphs.status_err, e),
        }
        
        match hklm.open_subkey(hklm_run_path) {
            Ok(_) => println!("{} HKLM Run Key: Accessible (Read)", glyphs.status_ok),
            Err(e) => println!("{} HKLM Run Key: Failed ({}) (Normal if not run as Admin)", glyphs.warning, e),
        }
    }
    #[cfg(not(windows))]
    {
        println!("{} Registry Storage: [Skipped - Non-Windows]", glyphs.info);
    }

    println!("\n===================================================");
    println!("Diagnostics Complete.");
    println!("===================================================");
}
