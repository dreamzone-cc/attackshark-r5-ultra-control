slint::include_modules!();

mod autostart;
mod config;
mod device;
mod protocol;
mod single_instance;
mod tray;

use config::{load_config, save_config, AppConfig};
use device::R5Device;
use ksni::blocking::TrayMethods;
use protocol::{DeviceState, LightingMode, LodSetting, PacketBuilder, PollingRate};
use slint::{Color, ComponentHandle};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn sync_ui_from_state(ui: &AppWindow, state: &DeviceState, connected: bool) {
    ui.set_battery_level(state.battery_level as i32);
    ui.set_battery_text(format!("{}%", state.battery_level).into());
    ui.set_is_charging(state.is_charging);
    ui.set_connection_status(if connected { "Connected (2.4G)" } else { "Disconnected" }.into());
    ui.set_active_profile(state.active_profile as i32);
    ui.set_active_dpi_stage(state.active_dpi_stage as i32);
    ui.set_dpi_stage_1(state.dpi_stages[0] as i32);
    ui.set_dpi_stage_2(state.dpi_stages[1] as i32);
    ui.set_dpi_stage_3(state.dpi_stages[2] as i32);
    ui.set_dpi_stage_4(state.dpi_stages[3] as i32);
    ui.set_dpi_stage_5(state.dpi_stages[4] as i32);
    ui.set_dpi_stage_6(state.dpi_stages[5] as i32);
    ui.set_polling_rate(state.polling_rate as i32);
    ui.set_lod_setting(state.lod as i32);
    ui.set_debounce_ms(state.debounce_ms as i32);
    ui.set_motion_sync_enabled(state.motion_sync);
    ui.set_ripple_control_enabled(state.ripple_control);
    ui.set_sleep_timeout_mins(state.sleep_timeout_mins as i32);
    ui.set_lighting_mode(state.lighting_mode as i32);
    ui.set_lighting_brightness(state.lighting_brightness as i32);
    ui.set_lighting_speed(state.lighting_speed as i32);
    ui.set_lighting_color_r(state.lighting_color.0 as i32);
    ui.set_lighting_color_g(state.lighting_color.1 as i32);
    ui.set_lighting_color_b(state.lighting_color.2 as i32);
    ui.set_firmware_version(state.firmware_version.clone().into());
    ui.set_autostart_enabled(autostart::is_enabled());

    if !connected {
        ui.set_status_color(Color::from_argb_u8(255, 243, 139, 168)); // Red
    } else if state.battery_level > 60 {
        ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161)); // Green
    } else if state.battery_level > 20 {
        ui.set_status_color(Color::from_argb_u8(255, 249, 226, 175)); // Yellow
    } else {
        ui.set_status_color(Color::from_argb_u8(255, 243, 139, 168)); // Red
    }
}

fn sync_state_from_ui(ui: &AppWindow, state: &mut DeviceState) {
    state.active_dpi_stage = ui.get_active_dpi_stage() as u8;
    state.dpi_stages[0] = ui.get_dpi_stage_1() as u16;
    state.dpi_stages[1] = ui.get_dpi_stage_2() as u16;
    state.dpi_stages[2] = ui.get_dpi_stage_3() as u16;
    state.dpi_stages[3] = ui.get_dpi_stage_4() as u16;
    state.dpi_stages[4] = ui.get_dpi_stage_5() as u16;
    state.dpi_stages[5] = ui.get_dpi_stage_6() as u16;
    state.polling_rate = PollingRate::from_hz(ui.get_polling_rate());
    state.lod = if ui.get_lod_setting() == 1 { LodSetting::High2mm } else { LodSetting::Low1mm };
    state.debounce_ms = ui.get_debounce_ms() as u8;
    state.motion_sync = ui.get_motion_sync_enabled();
    state.ripple_control = ui.get_ripple_control_enabled();
    state.sleep_timeout_mins = ui.get_sleep_timeout_mins() as u8;
    state.lighting_mode = match ui.get_lighting_mode() {
        0 => LightingMode::Off,
        1 => LightingMode::Static,
        2 => LightingMode::Breathing,
        4 => LightingMode::Wave,
        _ => LightingMode::Neon,
    };
    state.lighting_brightness = ui.get_lighting_brightness() as u8;
    state.lighting_speed = ui.get_lighting_speed() as u8;
    state.lighting_color = (
        ui.get_lighting_color_r() as u8,
        ui.get_lighting_color_g() as u8,
        ui.get_lighting_color_b() as u8,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Single-Instance Check via Unix Socket
    let instance_check = single_instance::check_or_become_primary();
    let listener = match instance_check {
        single_instance::InstanceCheck::Primary(l) => l,
        single_instance::InstanceCheck::AlreadyRunning => {
            return Ok(());
        }
    };

    eprintln!("[INFO] Initializing Glitch R5U...");

    let main_window = AppWindow::new()?;
    let ui_weak = main_window.as_weak();

    // 2. Spawn Single-Instance IPC Listener (brings window to front if launched again)
    let ui_weak_ipc = ui_weak.clone();
    single_instance::spawn_ipc_server(listener, move || {
        let _ = ui_weak_ipc.upgrade_in_event_loop(|ui| {
            let _ = ui.show();
        });
    });

    // Prevent close from destroying window, just hide
    let ui_weak_close = ui_weak.clone();
    main_window.on_close_clicked(move || {
        if let Some(ui) = ui_weak_close.upgrade() {
            let _ = ui.hide();
        }
    });

    main_window.window().on_close_requested(|| {
        slint::CloseRequestResponse::HideWindow
    });

    // Custom TitleBar Window Drag Handler
    let ui_weak_drag = ui_weak.clone();
    main_window.on_window_dragged(move |dx, dy| {
        if let Some(ui) = ui_weak_drag.upgrade() {
            let win = ui.window();
            let scale = win.scale_factor();
            let p_dx = (dx * scale) as i32;
            let p_dy = (dy * scale) as i32;
            if p_dx != 0 || p_dy != 0 {
                let pos = win.position();
                win.set_position(slint::PhysicalPosition::new(pos.x + p_dx, pos.y + p_dy));
            }
        }
    });

    // Shared State
    let mut current_state = DeviceState::default();
    let saved_cfg = load_config();
    current_state.active_profile = saved_cfg.active_profile;
    current_state.active_dpi_stage = saved_cfg.active_dpi_stage;
    current_state.dpi_stages = saved_cfg.dpi_stages;
    current_state.polling_rate = PollingRate::from_hz(saved_cfg.polling_rate_hz);
    current_state.lod = if saved_cfg.lod_code == 1 { LodSetting::High2mm } else { LodSetting::Low1mm };
    current_state.debounce_ms = saved_cfg.debounce_ms;
    current_state.motion_sync = saved_cfg.motion_sync;
    current_state.ripple_control = saved_cfg.ripple_control;
    current_state.sleep_timeout_mins = saved_cfg.sleep_timeout_mins;
    current_state.lighting_mode = match saved_cfg.lighting_mode_code {
        0 => LightingMode::Off,
        1 => LightingMode::Static,
        2 => LightingMode::Breathing,
        4 => LightingMode::Wave,
        _ => LightingMode::Neon,
    };
    current_state.lighting_brightness = saved_cfg.lighting_brightness;
    current_state.lighting_speed = saved_cfg.lighting_speed;
    current_state.lighting_color = (
        saved_cfg.lighting_color_r,
        saved_cfg.lighting_color_g,
        saved_cfg.lighting_color_b,
    );

    // Initial Hardware Query
    let mut is_connected = false;
    if let Some(dev) = R5Device::open() {
        is_connected = true;
        eprintln!("[INFO] Hardware connected on {:?}", dev.path);
        if let Some((lvl, is_chg)) = dev.read_battery() {
            eprintln!("[INFO] Initial battery read: {}% (Charging: {})", lvl, is_chg);
            current_state.battery_level = lvl;
            current_state.is_charging = is_chg;
        }
        current_state.firmware_version = dev.read_firmware_version();
    } else {
        eprintln!("[WARN] Mouse device not detected on startup.");
    }

    let shared_state = Arc::new(Mutex::new(current_state.clone()));
    let battery_atomic = Arc::new(AtomicI32::new(current_state.battery_level as i32));
    let dpi_atomic = Arc::new(AtomicI32::new(current_state.dpi_stages[current_state.active_dpi_stage as usize] as i32));

    sync_ui_from_state(&main_window, &current_state, is_connected);
    if is_connected {
        main_window.set_status_banner("Ready - Device connected and synchronized".into());
        main_window.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
    } else {
        main_window.set_status_banner("Mouse device not connected".into());
        main_window.set_status_color(Color::from_argb_u8(255, 243, 139, 168));
    }

    // Setup System Tray
    let ui_weak_tray = ui_weak.clone();
    let on_open: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        let _ = ui_weak_tray.upgrade_in_event_loop(|ui| {
            if ui.window().is_visible() {
                let _ = ui.hide();
            } else {
                let _ = ui.show();
            }
        });
    });

    let state_tray_refresh = Arc::clone(&shared_state);
    let ui_weak_refresh = ui_weak.clone();
    let battery_atomic_ref = Arc::clone(&battery_atomic);
    let on_refresh: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        if let Some(dev) = R5Device::open() {
            if let Some((lvl, chg)) = dev.read_battery() {
                let mut st = state_tray_refresh.lock().unwrap();
                st.battery_level = lvl;
                st.is_charging = chg;
                battery_atomic_ref.store(lvl as i32, Ordering::Relaxed);
                let st_clone = st.clone();
                let _ = ui_weak_refresh.upgrade_in_event_loop(move |ui| {
                    sync_ui_from_state(&ui, &st_clone, true);
                    ui.set_status_banner("Status refreshed from device".into());
                    ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
                });
            }
        }
    });

    let state_tray_dpi = Arc::clone(&shared_state);
    let ui_weak_dpi = ui_weak.clone();
    let dpi_atomic_ref = Arc::clone(&dpi_atomic);
    let on_set_dpi_stage: Arc<dyn Fn(u8) + Send + Sync> = Arc::new(move |stage_idx| {
        let stage = stage_idx.min(5);
        let mut connected = false;
        if let Some(dev) = R5Device::open() {
            connected = true;
            let pkt = PacketBuilder::build_set_active_dpi(stage);
            let _ = dev.send_feature(&pkt);
        }
        let mut st = state_tray_dpi.lock().unwrap();
        st.active_dpi_stage = stage;
        let new_dpi = st.dpi_stages[stage as usize] as i32;
        dpi_atomic_ref.store(new_dpi, Ordering::Relaxed);
        let st_clone = st.clone();
        let _ = ui_weak_dpi.upgrade_in_event_loop(move |ui| {
            sync_ui_from_state(&ui, &st_clone, connected);
        });
    });

    let on_quit: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
        eprintln!("[INFO] Quitting Glitch R5U daemon.");
        let _ = slint::quit_event_loop();
    });

    let tray_item = tray::ControlTray {
        battery_level: Arc::clone(&battery_atomic),
        active_dpi: Arc::clone(&dpi_atomic),
        on_open: Arc::clone(&on_open),
        on_refresh: Arc::clone(&on_refresh),
        on_set_dpi_stage: Arc::clone(&on_set_dpi_stage),
        on_quit: Arc::clone(&on_quit),
    };

    eprintln!("[INFO] Spawning KDE Plasma System Tray StatusNotifierItem...");
    let tray_handle = tray_item.spawn()?;

    // UI Callbacks
    let state_ui_refresh = Arc::clone(&shared_state);
    let ui_weak_btn_refresh = ui_weak.clone();
    let tray_handle_btn = tray_handle.clone();
    let battery_btn_atomic = Arc::clone(&battery_atomic);
    main_window.on_refresh_all(move || {
        if let Some(dev) = R5Device::open() {
            let mut st = state_ui_refresh.lock().unwrap();
            if let Some((lvl, chg)) = dev.read_battery() {
                st.battery_level = lvl;
                st.is_charging = chg;
                battery_btn_atomic.store(lvl as i32, Ordering::Relaxed);
            }
            st.firmware_version = dev.read_firmware_version();
            tray_handle_btn.update(|_| {});
            if let Some(ui) = ui_weak_btn_refresh.upgrade() {
                sync_ui_from_state(&ui, &st, true);
                ui.set_status_banner("Hardware refreshed successfully".into());
                ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
            }
        } else {
            if let Some(ui) = ui_weak_btn_refresh.upgrade() {
                ui.set_connection_status("Disconnected".into());
                ui.set_status_banner("Mouse device not connected".into());
                ui.set_status_color(Color::from_argb_u8(255, 243, 139, 168));
            }
        }
    });

    let state_apply = Arc::clone(&shared_state);
    let ui_weak_apply = ui_weak.clone();
    let tray_handle_apply = tray_handle.clone();
    let dpi_apply_atomic = Arc::clone(&dpi_atomic);
    main_window.on_apply_settings(move || {
        if let Some(ui) = ui_weak_apply.upgrade() {
            let mut st = state_apply.lock().unwrap();
            sync_state_from_ui(&ui, &mut st);
            dpi_apply_atomic.store(st.dpi_stages[st.active_dpi_stage as usize] as i32, Ordering::Relaxed);

            if let Some(dev) = R5Device::open() {
                eprintln!("[INFO] Applying settings via {:?}", dev.path);
                if dev.apply_settings(&st) {
                    ui.set_connection_status("Connected (2.4G)".into());
                    ui.set_status_banner("Settings applied to mouse successfully".into());
                    ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
                } else {
                    ui.set_status_banner("Warning: Write incomplete".into());
                    ui.set_status_color(Color::from_argb_u8(255, 249, 226, 175));
                }
            } else {
                eprintln!("[ERROR] Apply settings failed: Mouse device not found.");
                ui.set_connection_status("Disconnected".into());
                ui.set_status_banner("Mouse device not connected".into());
                ui.set_status_color(Color::from_argb_u8(255, 243, 139, 168));
            }
            tray_handle_apply.update(|_| {});
        }
    });

    let state_sync = Arc::clone(&shared_state);
    let ui_weak_sync = ui_weak.clone();
    main_window.on_sync_onboard(move || {
        if let Some(ui) = ui_weak_sync.upgrade() {
            let mut st = state_sync.lock().unwrap();
            sync_state_from_ui(&ui, &mut st);

            let cfg = AppConfig {
                active_profile: st.active_profile,
                active_dpi_stage: st.active_dpi_stage,
                dpi_stages: st.dpi_stages,
                polling_rate_hz: st.polling_rate as i32,
                lod_code: st.lod as u8,
                debounce_ms: st.debounce_ms,
                motion_sync: st.motion_sync,
                ripple_control: st.ripple_control,
                sleep_timeout_mins: st.sleep_timeout_mins,
                lighting_mode_code: st.lighting_mode as u8,
                lighting_brightness: st.lighting_brightness,
                lighting_speed: st.lighting_speed,
                lighting_color_r: st.lighting_color.0,
                lighting_color_g: st.lighting_color.1,
                lighting_color_b: st.lighting_color.2,
            };
            let _ = save_config(&cfg);

            if let Some(dev) = R5Device::open() {
                let _ = dev.apply_settings(&st);
                ui.set_connection_status("Connected (2.4G)".into());
                ui.set_status_banner("Settings synced and saved to onboard memory".into());
                ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
            } else {
                ui.set_connection_status("Disconnected".into());
                ui.set_status_banner("Saved locally (Mouse not connected)".into());
                ui.set_status_color(Color::from_argb_u8(255, 249, 226, 175));
            }
        }
    });

    let state_dpi_btn = Arc::clone(&shared_state);
    let ui_weak_dpi_btn = ui_weak.clone();
    let tray_handle_dpi = tray_handle.clone();
    let dpi_atomic_btn = Arc::clone(&dpi_atomic);
    main_window.on_set_active_dpi(move |stage_idx| {
        let stage = (stage_idx as u8).min(5);
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_active_dpi(stage);
            let _ = dev.send_feature(&pkt);
        }
        let mut st = state_dpi_btn.lock().unwrap();
        st.active_dpi_stage = stage;
        let new_dpi = st.dpi_stages[stage as usize] as i32;
        dpi_atomic_btn.store(new_dpi, Ordering::Relaxed);
        tray_handle_dpi.update(|_| {});
        if let Some(ui) = ui_weak_dpi_btn.upgrade() {
            ui.set_active_dpi_stage(stage as i32);
            ui.set_status_banner(format!("Switched to DPI Stage {} ({} DPI)", stage + 1, new_dpi).into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let ui_weak_polling = ui_weak.clone();
    main_window.on_set_polling_rate_value(move |rate_val| {
        if let Some(dev) = R5Device::open() {
            let rate = PollingRate::from_hz(rate_val);
            let pkt = PacketBuilder::build_set_polling_rate(rate);
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_polling.upgrade() {
            ui.set_polling_rate(rate_val);
            ui.set_status_banner(format!("Polling rate set to {} Hz", rate_val).into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let ui_weak_lod = ui_weak.clone();
    main_window.on_set_lod_value(move |lod_val| {
        let lod = if lod_val == 1 { LodSetting::High2mm } else { LodSetting::Low1mm };
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_lod(lod);
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_lod.upgrade() {
            ui.set_lod_setting(lod_val);
            ui.set_status_banner(format!("LOD set to {}", if lod_val == 1 { "2.0 mm" } else { "1.0 mm" }).into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    // RGB Lighting Callbacks
    let state_lighting_mode = Arc::clone(&shared_state);
    let ui_weak_lighting_mode = ui_weak.clone();
    main_window.on_set_lighting_mode_value(move |mode_val| {
        let mode = match mode_val {
            0 => LightingMode::Off,
            1 => LightingMode::Static,
            2 => LightingMode::Breathing,
            4 => LightingMode::Wave,
            _ => LightingMode::Neon,
        };
        let mut st = state_lighting_mode.lock().unwrap();
        st.lighting_mode = mode;
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_lighting(
                mode,
                st.lighting_brightness,
                st.lighting_speed,
                st.lighting_color.0,
                st.lighting_color.1,
                st.lighting_color.2,
            );
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_lighting_mode.upgrade() {
            ui.set_lighting_mode(mode_val);
            ui.set_status_banner("RGB Lighting mode updated".into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let state_lighting_bright = Arc::clone(&shared_state);
    let ui_weak_lighting_bright = ui_weak.clone();
    main_window.on_set_lighting_brightness_value(move |bright_val| {
        let b = (bright_val as u8).min(100);
        let mut st = state_lighting_bright.lock().unwrap();
        st.lighting_brightness = b;
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_lighting(
                st.lighting_mode,
                b,
                st.lighting_speed,
                st.lighting_color.0,
                st.lighting_color.1,
                st.lighting_color.2,
            );
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_lighting_bright.upgrade() {
            ui.set_lighting_brightness(bright_val);
            ui.set_status_banner(format!("RGB Brightness set to {}%", b).into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let state_lighting_speed = Arc::clone(&shared_state);
    let ui_weak_lighting_speed = ui_weak.clone();
    main_window.on_set_lighting_speed_value(move |speed_val| {
        let s = (speed_val as u8).clamp(1, 10);
        let mut st = state_lighting_speed.lock().unwrap();
        st.lighting_speed = s;
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_lighting(
                st.lighting_mode,
                st.lighting_brightness,
                s,
                st.lighting_color.0,
                st.lighting_color.1,
                st.lighting_color.2,
            );
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_lighting_speed.upgrade() {
            ui.set_lighting_speed(speed_val);
            ui.set_status_banner(format!("RGB Speed set to {}x", s).into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let state_lighting_color = Arc::clone(&shared_state);
    let ui_weak_lighting_color = ui_weak.clone();
    main_window.on_set_lighting_color_preset(move |r, g, b| {
        let (cr, cg, cb) = (r as u8, g as u8, b as u8);
        let mut st = state_lighting_color.lock().unwrap();
        st.lighting_color = (cr, cg, cb);
        if let Some(dev) = R5Device::open() {
            let pkt = PacketBuilder::build_set_lighting(
                st.lighting_mode,
                st.lighting_brightness,
                st.lighting_speed,
                cr,
                cg,
                cb,
            );
            let _ = dev.send_feature(&pkt);
        }
        if let Some(ui) = ui_weak_lighting_color.upgrade() {
            ui.set_lighting_color_r(r);
            ui.set_lighting_color_g(g);
            ui.set_lighting_color_b(b);
            ui.set_status_banner("RGB Color updated".into());
            ui.set_status_color(Color::from_argb_u8(255, 166, 227, 161));
        }
    });

    let ui_weak_factory = ui_weak.clone();
    main_window.on_reset_factory(move || {
        let def = DeviceState::default();
        if let Some(dev) = R5Device::open() {
            let _ = dev.apply_settings(&def);
        }
        if let Some(ui) = ui_weak_factory.upgrade() {
            sync_ui_from_state(&ui, &def, true);
            ui.set_status_banner("Reset to Factory Defaults completed".into());
            ui.set_status_color(Color::from_argb_u8(255, 249, 226, 175));
        }
    });

    main_window.on_autostart_toggled(|enabled| {
        let _ = autostart::set_enabled(enabled);
    });

    main_window.on_quit_clicked(|| {
        let _ = slint::quit_event_loop();
    });

    // Periodic Background Polling Timer (every 60s)
    let state_timer = Arc::clone(&shared_state);
    let tray_handle_timer = tray_handle.clone();
    let battery_timer_atomic = Arc::clone(&battery_atomic);
    let ui_weak_timer = ui_weak.clone();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(60),
        move || {
            if let Some(dev) = R5Device::open() {
                if let Some((lvl, chg)) = dev.read_battery() {
                    let mut st = state_timer.lock().unwrap();
                    st.battery_level = lvl;
                    st.is_charging = chg;
                    battery_timer_atomic.store(lvl as i32, Ordering::Relaxed);
                    tray_handle_timer.update(|_| {});
                    if let Some(ui) = ui_weak_timer.upgrade() {
                        ui.set_battery_level(lvl as i32);
                        ui.set_battery_text(format!("{}%", lvl).into());
                        ui.set_is_charging(chg);
                        ui.set_connection_status("Connected (2.4G)".into());
                    }
                }
            } else {
                if let Some(ui) = ui_weak_timer.upgrade() {
                    ui.set_connection_status("Disconnected".into());
                }
            }
        },
    );

    // Initial show
    main_window.show()?;

    eprintln!("[INFO] Running Slint event loop until quit...");
    slint::run_event_loop_until_quit()?;
    eprintln!("[INFO] Glitch R5U event loop terminated cleanly.");

    Ok(())
}
