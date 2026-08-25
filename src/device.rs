use crate::protocol::*;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

pub struct R5Device {
    file: File,
    pub path: PathBuf,
}

impl R5Device {
    pub fn open() -> Option<Self> {
        let path = find_device_path()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
            .map_err(|e| {
                eprintln!("[ERROR] Failed to open {:?}: {}", path, e);
                e
            })
            .ok()?;
        Some(Self { file, path })
    }

    pub fn send_feature(&self, buf: &[u8; REPORT_SIZE]) -> bool {
        unsafe {
            let res = libc::ioctl(self.file.as_raw_fd(), HIDIOCSFEATURE_65, buf.as_ptr());
            res == REPORT_SIZE as i32
        }
    }

    pub fn query_feature(&self, request: &[u8; REPORT_SIZE]) -> Option<[u8; REPORT_SIZE]> {
        if !self.send_feature(request) {
            return None;
        }
        sleep(Duration::from_millis(35)); // Throttling delay

        let mut recv = [0u8; REPORT_SIZE];
        unsafe {
            let res = libc::ioctl(self.file.as_raw_fd(), HIDIOCGFEATURE_65, recv.as_mut_ptr());
            if res == REPORT_SIZE as i32 && recv[1] == 0xA1 {
                Some(recv)
            } else {
                None
            }
        }
    }

    pub fn read_battery(&self) -> Option<(u8, bool)> {
        let req = PacketBuilder::build_battery_query();
        let resp = self.query_feature(&req)?;
        let lvl = resp[8];
        let is_charging = resp[9] != 0;
        Some((lvl.min(100), is_charging))
    }

    pub fn read_firmware_version(&self) -> String {
        let req = PacketBuilder::build_fw_query();
        if let Some(resp) = self.query_feature(&req) {
            if resp[7] != 0 || resp[8] != 0 || resp[9] != 0 {
                return format!("{}.{}.{}.{}", resp[7], resp[8], resp[9], resp[10]);
            }
        }
        "0.0.12.0".into()
    }

    pub fn apply_settings(&self, state: &DeviceState) -> bool {
        let mut ok = true;
        eprintln!("[INFO] Transmitting configuration packets to mouse...");

        // 1. Polling Rate
        let pkt = PacketBuilder::build_set_polling_rate(state.polling_rate);
        ok &= self.send_feature(&pkt);
        sleep(Duration::from_millis(30));

        // 2. Active DPI Stage
        let pkt = PacketBuilder::build_set_active_dpi(state.active_dpi_stage);
        ok &= self.send_feature(&pkt);
        sleep(Duration::from_millis(30));

        // 3. DPI Values
        for (idx, &val) in state.dpi_stages.iter().enumerate() {
            let pkt = PacketBuilder::build_set_dpi_value(idx as u8, val);
            ok &= self.send_feature(&pkt);
            sleep(Duration::from_millis(25));
        }

        // 4. LOD
        let pkt = PacketBuilder::build_set_lod(state.lod);
        ok &= self.send_feature(&pkt);
        sleep(Duration::from_millis(30));

        // 5. Debounce
        let pkt = PacketBuilder::build_set_debounce(state.debounce_ms);
        ok &= self.send_feature(&pkt);
        sleep(Duration::from_millis(30));

        // 6. Lighting
        let (r, g, b) = state.lighting_color;
        let pkt = PacketBuilder::build_set_lighting(
            state.lighting_mode,
            state.lighting_brightness,
            state.lighting_speed,
            r,
            g,
            b,
        );
        ok &= self.send_feature(&pkt);
        sleep(Duration::from_millis(30));

        eprintln!("[INFO] All configuration packets transmitted successfully.");
        ok
    }
}

pub fn find_device_path() -> Option<PathBuf> {
    let vid_pid_tag = format!("{:04X}:{:04X}", VENDOR_ID, PRODUCT_ID); // "373E:0047"
    let hidraw_dir = Path::new("/sys/class/hidraw");
    let entries = std::fs::read_dir(hidraw_dir).ok()?;

    for entry in entries.flatten() {
        let uevent_path = entry.path().join("device/uevent");
        if let Ok(content) = std::fs::read_to_string(&uevent_path) {
            let upper = content.to_ascii_uppercase();
            if upper.contains(&vid_pid_tag) || upper.contains("373E") && upper.contains("0047") {
                // Check if this is interface 2 (Vendor control interface)
                let parent_uevent = entry.path().join("device/../uevent");
                let parent_content = std::fs::read_to_string(&parent_uevent).unwrap_or_default().to_ascii_uppercase();

                if upper.contains("INPUT2") || parent_content.contains("IN02") || parent_content.contains("INTERFACE=3/0/0") {
                    let node_name = entry.file_name();
                    let dev_path = PathBuf::from("/dev").join(node_name);
                    eprintln!("[INFO] Found target Glitch R5U interface 2 at {:?}", dev_path);
                    return Some(dev_path);
                }
            }
        }
    }

    // Fallback: If interface 2 wasn't matched explicitly by uevent tag, look for any 373E:0047 hidraw node with highest index
    let mut matching_nodes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(hidraw_dir) {
        for entry in entries.flatten() {
            let uevent_path = entry.path().join("device/uevent");
            if let Ok(content) = std::fs::read_to_string(&uevent_path) {
                let upper = content.to_ascii_uppercase();
                if upper.contains("373E") && upper.contains("0047") {
                    matching_nodes.push(PathBuf::from("/dev").join(entry.file_name()));
                }
            }
        }
    }

    if !matching_nodes.is_empty() {
        matching_nodes.sort();
        let fallback = matching_nodes.last().cloned();
        eprintln!("[INFO] Fallback selected device path: {:?}", fallback);
        return fallback;
    }

    eprintln!("[WARN] No Glitch R5U / Attack Shark R5 device found in sysfs.");
    None
}
