use std::fs::{create_dir_all, remove_file, File};
use std::io::Write;
use std::path::PathBuf;

fn autostart_file_path() -> PathBuf {
    let mut path = if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(dir)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from("/tmp")
    };
    path.push("autostart");
    let _ = create_dir_all(&path);
    path.push("attackshark-control.desktop");
    path
}

pub fn is_enabled() -> bool {
    autostart_file_path().exists()
}

pub fn set_enabled(enable: bool) -> std::io::Result<()> {
    let path = autostart_file_path();
    if enable {
        let exec_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                if let Ok(home) = std::env::var("HOME") {
                    format!("{}/.local/bin/attackshark-r5-ultra-control", home)
                } else {
                    "/usr/local/bin/attackshark-r5-ultra-control".to_string()
                }
            });

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Attack Shark R5 Ultra Control Center\n\
             GenericName=Gaming Mouse Control Center\n\
             Comment=Native Linux Control Center & Battery Monitor for Attack Shark R5 Ultra\n\
             Exec={}\n\
             Icon=attackshark-battery\n\
             Terminal=false\n\
             Categories=Utility;HardwareSettings;\n\
             StartupNotify=false\n\
             X-KDE-autostart-phase=2\n\
             X-GNOME-Autostart-enabled=true\n",
            exec_path
        );
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;

        let _ = std::process::Command::new("systemctl")
            .args(&["--user", "enable", "attackshark-control.service"])
            .status();
    } else {
        if path.exists() {
            let _ = remove_file(&path);
        }
        let _ = std::process::Command::new("systemctl")
            .args(&["--user", "disable", "attackshark-control.service"])
            .status();
    }
    Ok(())
}
