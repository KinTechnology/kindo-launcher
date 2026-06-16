use anyhow::Result;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};
use std::{env, fs, thread};

const MAX_KINDO_REBOOT_ATTEMPT: u8 = 3;

struct KindoMonitor {
    root: PathBuf,
    runtime_dir: PathBuf,

    attempts: u8,
}

impl KindoMonitor {
    fn monitor_kindo(&self) -> Result<()> {
        let runtime_dir = &self.runtime_dir;

        if self.attempts > MAX_KINDO_REBOOT_ATTEMPT {
            panic!("Kindo has been restarted too many times, aborting");
        }

        loop {
            let is_stale = fs::metadata(runtime_dir.join("./kindo-app-keepalive"))
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .map(|dur| {
                    println!("Kindo keepalive mtime: {:?}", dur);
                    dur.as_secs() > 20
                })
                .unwrap_or(true);

            if is_stale {
                println!("Kindo is stale, restarting...");
                self.kill_kindo();
                let _ = self.run_kindo();
            }
            println!("Kindo is alive, sleeping...");

            thread::sleep(Duration::from_secs(5));
        }
    }

    fn kill_kindo(&self) {
        let _ = Command::new("killall").args(["-q", "kindo-app"]).status();
    }

    fn run_kindo(&self) -> Result<Child> {
        let kindo_bin_path =
            self.root.join("./vertex/dist/mac-arm64/kindo-app.app/Contents/MacOS/kindo-app");

        let path_str = kindo_bin_path.to_str().unwrap();

        let mut cmd = Command::new(path_str);
        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to start kindo-app");

        return Ok(child);
    }
}

fn main() {
    let app_env = std::env::var("APP_ENV").unwrap_or(String::from("production"));
    println!("APP_ENV: {}", app_env);

    let runtime_dir =
        PathBuf::from(std::env::var("RUNTIME_DIRECTORY").unwrap_or(String::from("/tmp")));

    let root = env::current_dir().unwrap().parent().unwrap().to_path_buf();

    let monitor = KindoMonitor {
        root,
        runtime_dir,
        attempts: 0,
    };

    monitor.kill_kindo();
    let _ = monitor.run_kindo().unwrap();
    monitor.monitor_kindo().unwrap();
}
