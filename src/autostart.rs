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
        let content = r#"[Desktop Entry]
Type=Application
Name=Attack Shark R5 Ultra Control Center
Comment=Gaming Mouse Control Center & Battery Monitor
Exec=/usr/local/bin/attackshark-r5-ultra-control
Icon=attackshark-battery
Terminal=false
Categories=Utility;HardwareSettings;
X-KDE-autostart-phase=2
"#;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
    } else if path.exists() {
        let _ = remove_file(&path);
    }
    Ok(())
}
