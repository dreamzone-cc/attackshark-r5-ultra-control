pub const VENDOR_ID: u16 = 0x373E;
pub const PRODUCT_ID: u16 = 0x0047;
pub const REPORT_SIZE: usize = 65;

pub const HIDIOCSFEATURE_65: libc::c_ulong = 0xC0414806;
pub const HIDIOCGFEATURE_65: libc::c_ulong = 0xC0414807;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingRate {
    Hz125 = 125,
    Hz250 = 250,
    Hz500 = 500,
    Hz1000 = 1000,
    Hz2000 = 2000,
    Hz4000 = 4000,
    Hz8000 = 8000,
}

impl PollingRate {
    pub fn from_hz(hz: i32) -> Self {
        match hz {
            125 => PollingRate::Hz125,
            250 => PollingRate::Hz250,
            500 => PollingRate::Hz500,
            2000 => PollingRate::Hz2000,
            4000 => PollingRate::Hz4000,
            8000 => PollingRate::Hz8000,
            _ => PollingRate::Hz1000,
        }
    }

    pub fn to_code(&self) -> u8 {
        match self {
            PollingRate::Hz125 => 0,
            PollingRate::Hz250 => 1,
            PollingRate::Hz500 => 2,
            PollingRate::Hz1000 => 3,
            PollingRate::Hz2000 => 4,
            PollingRate::Hz4000 => 5,
            PollingRate::Hz8000 => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodSetting {
    Low1mm = 0,
    High2mm = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    Off = 0,
    Static = 1,
    Breathing = 2,
    Neon = 3,
    Wave = 4,
}

#[derive(Debug, Clone)]
pub struct DeviceState {
    pub battery_level: u8,
    pub is_charging: bool,
    pub active_profile: u8,
    pub active_dpi_stage: u8,
    pub dpi_stages: [u16; 6],
    pub polling_rate: PollingRate,
    pub lod: LodSetting,
    pub debounce_ms: u8,
    pub motion_sync: bool,
    pub ripple_control: bool,
    pub sleep_timeout_mins: u8,
    pub lighting_mode: LightingMode,
    pub lighting_brightness: u8,
    pub lighting_speed: u8,
    pub lighting_color: (u8, u8, u8),
    pub firmware_version: String,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            battery_level: 100,
            is_charging: false,
            active_profile: 1,
            active_dpi_stage: 1,
            dpi_stages: [800, 1600, 2400, 3200, 5000, 12000],
            polling_rate: PollingRate::Hz1000,
            lod: LodSetting::Low1mm,
            debounce_ms: 4,
            motion_sync: true,
            ripple_control: false,
            sleep_timeout_mins: 5,
            lighting_mode: LightingMode::Neon,
            lighting_brightness: 80,
            lighting_speed: 5,
            lighting_color: (137, 180, 250),
            firmware_version: "0.0.12.0".into(),
        }
    }
}

// Packet Builders
pub struct PacketBuilder;

impl PacketBuilder {
    pub fn build_battery_query() -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x02;
        buf[5] = 0x00;
        buf[6] = 0x83; // Opcode: Battery
        buf
    }

    pub fn build_fw_query() -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x10;
        buf[5] = 0x00;
        buf[6] = 0x81; // Opcode: FW Version
        buf
    }

    pub fn build_settings_query(cmd: u8) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x02;
        buf[5] = 0x00;
        buf[6] = cmd;
        buf
    }

    pub fn build_set_polling_rate(rate: PollingRate) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x01; // func
        buf[5] = 0x00;
        buf[6] = 0x01; // sub / opcode
        buf[7] = rate.to_code();
        buf
    }

    pub fn build_set_active_dpi(stage_idx: u8) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x1A; // DPI Func
        buf[5] = 0x00;
        buf[6] = 0x01;
        buf[7] = stage_idx;
        buf
    }

    pub fn build_set_dpi_value(stage_idx: u8, dpi: u16) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x1A;
        buf[5] = 0x00;
        buf[6] = 0x02;
        buf[7] = stage_idx;
        buf[8] = ((dpi >> 8) & 0xFF) as u8;
        buf[9] = (dpi & 0xFF) as u8;
        buf
    }

    pub fn build_set_lod(lod: LodSetting) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x01;
        buf[5] = 0x00;
        buf[6] = 0x08; // LOD opcode
        buf[7] = lod as u8;
        buf
    }

    pub fn build_set_debounce(ms: u8) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x01;
        buf[5] = 0x00;
        buf[6] = 0x0B; // Debounce opcode
        buf[7] = ms.min(30);
        buf
    }

    pub fn build_set_lighting(mode: LightingMode, brightness: u8, speed: u8, r: u8, g: u8, b: u8) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x00;
        buf[1] = 0xA1;
        buf[2] = 0x00;
        buf[3] = 0x02;
        buf[4] = 0x13; // Lighting func
        buf[5] = 0x00;
        buf[6] = 0x01;
        buf[7] = mode as u8;
        buf[8] = speed.clamp(1, 10);
        buf[9] = brightness.min(100);
        buf[10] = r;
        buf[11] = g;
        buf[12] = b;
        buf
    }
}
