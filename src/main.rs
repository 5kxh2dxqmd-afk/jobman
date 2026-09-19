slint::include_modules!();
use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use slint::SharedString;
use std::fs;
use std::env;

#[cfg(all(target_os = "linux", target_arch = "arm"))]
static INTER: &[u8] = include_bytes!("./inter.ttf");

#[cfg(all(target_os = "linux", target_arch = "arm"))]
static BASKERVILLE: &[u8] = include_bytes!("./libre-baskerville.ttf");

/*
    Kindle has no fonts, 
    on desktop just install Inter & Libre Baskerville during testing.
*/

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    let backend = slint_backend_kindle::install(INTER).expect("Failed to Install!");

    let app = AppWindow::new().expect("Failed to Create Window!");
    
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    backend.register_font_from_memory(BASKERVILLE).expect("Failed to Install Libre Baskerville!");

    let version = env!("CARGO_PKG_VERSION");
    app.set_jobman_version(version.into());
    match load_ssh_config() {
        Ok(config) => {
            app.set_kindle_ssh_ip(config.kindle_ip.into());
        }
        Err(e) => {
            app.set_kindle_ssh_ip("N/A".into());
            app.set_error(format!("Could not load config: {}", e).into());
            app.set_show_error(true);
        }
    }

    app.set_boot_mode(recovery_status());
    app.set_ota_status(ota_status());
    app.set_wifi_status(wifi_status());
    app.set_usb_ssh_status(usb_ssh_enabled());
    match battery_health() {
        Ok(health) => {
            app.set_battery_health(health);
        }

        Err(error_message) => {
            app.set_error(error_message.into());
            app.set_show_error(true);
        }
    }

    let app_weak = app.as_weak(); //No memory leaks
    let toggle_weak = app_weak.clone();

    app.on_toggle_ota(move || {
        let app = toggle_weak.unwrap();
        let status = app.get_ota_status();
        
        let result = if status {
            block_ota() 
        } else {
            enable_ota() 
        };

        if let Err(error_message) = result {
            app.set_error(error_message.into());
            app.set_show_error(true);
        }
    });

    let update_weak = app_weak.clone();
    app.on_update_environment(move || {
        let app = update_weak.unwrap();
        //For redraw 

        let run_weak = update_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            match update_env() {
                Ok(_) => {
                    let _ = Command::new("sh").args(["-c", "sleep 1 && reboot"]).spawn();
                    std::process::exit(0);
                }
                Err(error_message) => {
                    if let Some(ui) = run_weak.upgrade() {
                        ui.set_error(error_message.into());
                        ui.set_show_error(true);
                    }
                }
            }
        });
    });

    let usb_ssh_weak = app_weak.clone();
    app.on_usb_ssh_toggle(move || {
        let app = usb_ssh_weak.unwrap();

        let run_weak = usb_ssh_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = run_weak.upgrade() else { return };
            let status = app.get_usb_ssh_status();
        
            let result = if status {
                disable_usb_ssh() 
            } else {
                enable_usb_ssh()
            };

            app.set_usb_ssh_status(usb_ssh_enabled()); //Sync

            if let Err(error_message) = result {
                app.set_error(error_message.into());
                app.set_show_error(true);
            }
        });
    });

    app.on_quit(|| std::process::exit(0));
    app.run().expect("Event Loop Error!");
}

//UI backend
fn recovery_status() -> bool {
    env::args().any(|arg| arg == "--recovery" || arg == "-r")
}

fn ota_status() -> bool {
    Path::new("/usr/bin/otav3").try_exists().unwrap_or(false)
}

fn sh(cmd: &str, err: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| format!("{err} (Process Error: {e})"))?; 

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!("{err} (Exit Code: {})", output.status.code().unwrap_or(-1)))
    }
}

fn chattr_path() -> &'static str {
    if Path::new("/bin/chattr.e2fsprogs").exists() {
        "/bin/chattr.e2fsprogs"
    } else {
        "/bin/chattr"
    }
}

//Nothing here is ever ran in recovery!
fn block_ota() -> Result<(), String> {
    let chattr = chattr_path();

    sh("mntroot rw", "Failed to mount RootFS as writeable")?;

    sh(
        &format!("{chattr} -i /usr/bin/otaupd /usr/bin/otav3"),
        "Could not make active binaries mutable"
    )?;

    sh(
        "mv /usr/bin/otaupd /usr/bin/otaupd.bck && mv /usr/bin/otav3 /usr/bin/otav3.bck",
        "Failed renaming OTA binaries to backup files"
    )?;

    sh(
        &format!("{chattr} +i /usr/bin/otaupd.bck /usr/bin/otav3.bck"),
        "Could not make backup files immutable via chattr"
    )?;

    let _ = sh("mntroot ro", "");

    Command::new("sh")
        .args(["-c", "sleep 3 && reboot"])
        .spawn()
        .map_err(|e| format!("Failed to reboot Kindle: {e}"))?;

    Ok(()) 
}

fn enable_ota() -> Result<(), String> {
    let chattr = chattr_path();

    sh("mntroot rw", "Failed to mount RootFS as writeable")?;

    sh(
        &format!("{chattr} -i /usr/bin/otaupd.bck /usr/bin/otav3.bck"),
        "Could not make backup files mutable"
    )?;

    sh(
        "mv /usr/bin/otaupd.bck /usr/bin/otaupd && mv /usr/bin/otav3.bck /usr/bin/otav3",
        "Failed renaming OTA backup files to active binaries"
    )?; 

    sh(
        &format!("{chattr} +i /usr/bin/otaupd /usr/bin/otav3"),
        "Could not make active binaries immutable via chattr"
    )?;

    let _ = sh("mntroot ro", ""); 

    Command::new("sh")
        .args(["-c", "sleep 3 && reboot"])
        .spawn()
        .map_err(|e| format!("Failed to reboot Kindle: {e}"))?;

    Ok(()) 
}

fn wifi_status() -> bool {
    if let Ok(con) = sh("lipc-get-prop com.lab126.wifid cmState", "WiFi check failed") {
        con.trim() == "CONNECTED"
    } else {
        false
    }
}

fn update_env() -> Result<(), String> {
    sh("curl -L https://kindlemodding.org/jb.sh | RUN_MODE=2 sh", "Failed to curl and run jailbreak script")?;

    Ok(())
}

fn original_mah_round(rough: f64) -> f64 {
    let battery_intervals: Vec<f64> = vec![1350.0, 1300.0, 890.0, 245.0, 1000.0, 1500.0, 900.0, 1130.0, 1700.0, 1040.0, 3000.0, 1900.0, 2310.0, 4000.0];
    let max_interval = battery_intervals.iter().copied().max_by(f64::total_cmp).unwrap_or(0.0); //In case retrieved mAh is invalid/faulty/new device comes out, round down to the largest without failing

    battery_intervals
        .iter()
        .copied()
        .filter(|&x| x >= rough)
        .min_by(f64::total_cmp)
        .unwrap_or(max_interval)
}

fn battery_health() -> Result<i32, String> {
    if recovery_status() {
        return Ok(100) //Don't run commands on app launch in recovery environment -- will fail anyway
    }

    let mah = sh("gasgauge-info -m", "Failed to retrieve battery mAh")?;
    let capav = sh("lipc-get-prop com.lab126.powerd battLevel", "Failed to retrieve battery capacity")?;
    let original_mah = sh("cat /sys/class/power_supply/bd*_bat/charge_full_design", "Failed to read battery initial capacity")?;

    let mah: f64 = mah
        .split_whitespace()
        .next() //["num", "mAh"] <- first item (capacity)
        .ok_or("Could not parse battery capacity")?
        .parse()
        .map_err(|_| "Could not parse battery capacity".to_string())?;

    let mah = mah / 1000.0; //mAh from uAh

    let capav: f64 = capav
        .trim()
        .parse()
        .map_err(|_| "Could not parse battery percentage".to_string())?;

    let original_mah: f64 = original_mah
        .trim()
        .parse()
        .map_err(|_| "Could not parse original battery capacity".to_string())?;

    //Original mAh seems to be inaccurate... for some reason. Round it up to a known factory default
    let original_mah = original_mah / 1000.0; //Returned in uAh not mAh; convert
    let accurate = original_mah_round(original_mah);

    let current = (mah / capav) * 100.0; 
    let health = (current / accurate) * 100.0; //Get % from 0.xx

    Ok((health.round() as i32).clamp(0, 100))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct SshConfig {
    kindle_ip: String,
    password_override_enabled: bool,
    password: String,
    allow_password_login: bool,
    port: u16,
    window_size: Option<u64>, 
}

fn load_ssh_config() -> Result<SshConfig, String> {
    let config_path = Path::new("/mnt/us/jobman/ssh/etc/config.toml");
    let config_contents = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    let config: SshConfig = toml::from_str(&config_contents)
        .map_err(|e| format!("Syntax error in config file: {}", e))?;

    Ok(config)
}

fn usb_ssh_enabled() -> bool {
    if let Ok(status) = sh("lipc-get-prop com.lab126.volumd useUsbForNetwork", "g_ether status check failed") {
        status.trim() == "1"
    } else {
        false
    }
}

fn enable_usb_ssh() -> Result<(), String> {
    let config = load_ssh_config()?; 

    //Start USB networking
    sh("lipc-set-prop -i -- com.lab126.volumd useUsbForNetwork 1", "Failed to enable g_ether")?;
    sh("lipc-send-event -r 3 -d 2 com.lab126.hal usbUnconfigured", "Failed to run usbUnconfigured")?;
    sh("lipc-send-event -r 3 -d 2 com.lab126.hal usbPlugOut", "Failed to run usbPlugOut")?;

    let ip_cmd = format!("ifconfig usb0 {}", config.kindle_ip);
    sh(&ip_cmd, "Failed to set usb0 IP, uh-oh..!")?;

    //Create daemon options string
    let mut options = format!(" -R -H\"/mnt/us\" -p\"{}\" -l\"usb0\"", config.port);

    if !config.allow_password_login {
        options.push_str(" -s");
    }

    if config.allow_password_login && config.password_override_enabled {
        options.push_str(&format!(" -Y\"{}\"", config.password)); 
    }

    if let Some(size) = config.window_size {
        options.push_str(&format!(" -W\"{}\"", size));
    }

    let start_daemon_cmd = format!(
        "nohup /mnt/us/jobman/bin/dropbearmulti dropbear{} >/dev/null 2>&1 &", 
        options
    );
    sh(&start_daemon_cmd, "Failed to start dropbear daemon!")?;

    Ok(())
}

fn disable_usb_ssh() -> Result<(), String> {
    sh("ifconfig usb0 down", "Failed to bring usb0 interface down")?;


    sh("lipc-set-prop -i -- com.lab126.volumd useUsbForNetwork 0", "Failed to disable g_ether")?;
    sh("lipc-send-event -r 3 -d 2 com.lab126.hal usbUnconfigured", "Failed to run usbUnconfigured")?;
    sh("lipc-send-event -r 3 -d 2 com.lab126.hal usbPlugOut", "Failed to run usbPlugOut")?;

    //Stop daemon
    sh("pkill -9 -f \"dropbearmulti dropbear\"", "Failed to kill dropbear daemon!")?;
    Ok(())
}