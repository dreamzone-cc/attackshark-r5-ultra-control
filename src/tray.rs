use ksni::{Icon, MenuItem, ToolTip, Tray};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub struct ControlTray {
    pub battery_level: Arc<AtomicI32>,
    pub active_dpi: Arc<AtomicI32>,
    pub on_open: Arc<dyn Fn() + Send + Sync>,
    pub on_refresh: Arc<dyn Fn() + Send + Sync>,
    pub on_set_dpi_stage: Arc<dyn Fn(u8) + Send + Sync>,
    pub on_quit: Arc<dyn Fn() + Send + Sync>,
}

impl Tray for ControlTray {
    fn id(&self) -> String {
        "attackshark-r5-ultra-control".into()
    }

    fn title(&self) -> String {
        "Attack Shark R5 Ultra".into()
    }

    fn icon_name(&self) -> String {
        "input-mouse".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let level = self.battery_level.load(Ordering::Relaxed);
        let width = 32;
        let height = 32;
        let mut data = Vec::with_capacity(width * height * 4);

        let (r, g, b) = if level < 0 {
            (147, 153, 178) // Gray
        } else if level > 60 {
            (166, 227, 161) // Green
        } else if level > 20 {
            (249, 226, 175) // Yellow
        } else {
            (243, 139, 168) // Red
        };

        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - 16;
                let dy = y as i32 - 16;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= 12 * 12 {
                    if dist_sq >= 9 * 9 {
                        data.push(255); // Alpha
                        data.push(30);  // R
                        data.push(30);  // G
                        data.push(46);  // B
                    } else {
                        data.push(255);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                    }
                } else {
                    data.push(0);
                    data.push(0);
                    data.push(0);
                    data.push(0);
                }
            }
        }

        vec![Icon {
            width: width as i32,
            height: height as i32,
            data,
        }]
    }

    fn tool_tip(&self) -> ToolTip {
        let level = self.battery_level.load(Ordering::Relaxed);
        let dpi = self.active_dpi.load(Ordering::Relaxed);
        let desc = if level >= 0 {
            format!("Attack Shark R5 Ultra\nBattery: {}%\nDPI: {}", level, dpi)
        } else {
            "Attack Shark R5 Ultra\nStatus: Offline / Sleep".into()
        };

        ToolTip {
            title: "Attack Shark R5 Ultra Control Center".into(),
            description: desc,
            icon_name: "input-mouse".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        (self.on_open)();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "🖱️ Attack Shark R5 Ultra".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "📊 Open Control Center".into(),
                activate: Box::new(|this: &mut Self| {
                    (this.on_open)();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "🔄 Refresh Device Status".into(),
                activate: Box::new(|this: &mut Self| {
                    (this.on_refresh)();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            SubMenu {
                label: "🎯 Quick DPI Switcher".into(),
                submenu: vec![
                    StandardItem {
                        label: "Stage 1 (800 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(0); }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Stage 2 (1600 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(1); }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Stage 3 (2400 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(2); }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Stage 4 (3200 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(3); }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Stage 5 (5000 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(4); }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Stage 6 (12000 DPI)".into(),
                        activate: Box::new(|this: &mut Self| { (this.on_set_dpi_stage)(5); }),
                        ..Default::default()
                    }.into(),
                ],
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "❌ Quit Daemon".into(),
                activate: Box::new(|this: &mut Self| {
                    (this.on_quit)();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
