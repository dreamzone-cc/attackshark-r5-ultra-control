use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("glitch-r5u.sock")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/glitch-r5u-{}.sock", uid))
    }
}

pub enum InstanceCheck {
    Primary(UnixListener),
    AlreadyRunning,
}

pub fn check_or_become_primary() -> InstanceCheck {
    let path = socket_path();

    // 1. Try connecting to an existing active socket
    if let Ok(mut stream) = UnixStream::connect(&path) {
        let _ = stream.write_all(b"SHOW\n");
        eprintln!("[INFO] Another instance is already running. Signaled existing instance to focus/show. Exiting cleanly.");
        return InstanceCheck::AlreadyRunning;
    }

    // 2. If connection failed, remove any stale socket file
    let _ = std::fs::remove_file(&path);

    // 3. Bind to socket as the primary instance
    match UnixListener::bind(&path) {
        Ok(listener) => {
            eprintln!("[INFO] Primary instance registered on socket: {:?}", path);
            InstanceCheck::Primary(listener)
        }
        Err(e) => {
            eprintln!("[WARN] Failed to bind instance socket ({:?}): {}. Retrying once...", path, e);
            let _ = std::fs::remove_file(&path);
            if let Ok(listener) = UnixListener::bind(&path) {
                InstanceCheck::Primary(listener)
            } else {
                InstanceCheck::AlreadyRunning
            }
        }
    }
}

pub fn spawn_ipc_server<F>(listener: UnixListener, on_show: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let on_show = Arc::new(on_show);
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 64];
                if let Ok(n) = stream.read(&mut buf) {
                    let cmd = String::from_utf8_lossy(&buf[..n]);
                    if cmd.contains("SHOW") {
                        on_show();
                        let _ = stream.write_all(b"OK\n");
                    }
                }
            }
        }
    });
}
