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
    path.push("glitch-r5u.desktop");
    path
}

pub fn is_enabled() -> bool {
    autostart_file_path().exists()
}

pub fn set_enabled(enable: bool) -> std::io::Result<()> {
    let path = autostart_file_path();
    // Also cleanup legacy desktop autostart file if present
    if let Some(parent) = path.parent() {
        let legacy_file = parent.join("attackshark-control.desktop");
        let _ = remove_file(legacy_file);
    }

    if enable {
        let exec_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                if let Ok(home) = std::env::var("HOME") {
                    format!("{}/.local/bin/glitch-r5u", home)
                } else {
                    "/usr/local/bin/glitch-r5u".to_string()
                }
            });

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Glitch R5U\n\
             GenericName=Gaming Mouse Utility\n\
             Comment=Glitch R5U Linux Control Suite for Attack Shark R5 Ultra Mouse\n\
             Exec={} --daemon\n\
             Icon=glitch-r5u\n\
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
            .args(&["--user", "enable", "glitch-r5u.service"])
            .status();
    } else {
        if path.exists() {
            let _ = remove_file(&path);
        }
        let _ = std::process::Command::new("systemctl")
            .args(&["--user", "disable", "glitch-r5u.service"])
            .status();
    }
    Ok(())
}
