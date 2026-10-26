use std::process::Command;
use std::thread;
use std::time::Duration;
use glob::glob;

//Abstraction
pub fn sh(cmd: &str, err: &str) -> Result<String, String> {
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

pub fn lipc_set(service: &str, prop: &str, val: &str, flags: Option<&str>) -> Result<(), String> {
    let flag = flags.unwrap_or("");
    let command = format!("lipc-set-prop {flag} com.lab126.{service} {prop} {val}");

    sh(&command, &format!("Failed to set prop {service}, {prop}, {val}"))?;

    Ok(())
}

pub fn lipc_get(service: &str, prop: &str) -> Result<String, String> {
    let command = format!("lipc-get-prop com.lab126.{service} {prop}");
    let output = sh(&command, &format!("Failed to get prop {service}, {prop}"))?;

    let trimmed = output.trim();

    if !trimmed.is_empty() {
        Ok(trimmed.to_string())
    } else {
        Err(format!("Failed to get prop {service}, {prop}, returned empty string"))
    }
}

pub fn lipc_event(service: &str, prop: &str, flags: Option<&str>) -> Result<(), String> {
    let flag = flags.unwrap_or("");
    let command = format!("lipc-send-event {flag} com.lab126.{service} {prop}");

    sh(&command, &format!("Failed to send event {service}, {flag}"))?;

    Ok(())
}

pub fn boot_enable(service: &str) -> Result<(), String> {
    let command = format!("initctl start {service} >/dev/null 2>&1");

    sh(&command, &format!("Failed to enable boot service {service}"))?;
    thread::sleep(Duration::from_millis(500));
    Ok(())
}
pub fn rootfs_writeable() -> Result<(), String> {
    sh("mntroot rw", "Failed to mount the rootfs as writeable")?;
    Ok(())
}

pub fn rootfs_readonly() -> Result<(), String> {
    sh("mntroot ro", "Failed to mount the rootfs as read-only")?;
    Ok(())
}

pub fn read_path(path: &str) -> Result<String, String> {
    let path = glob::glob(path)
        .map_err(|e| e.to_string())?
        .next()
        .ok_or_else(|| "Path is nonexistent!".to_string())? 
        .map_err(|e| e.to_string())?;

    let physical = path.canonicalize().map_err(|e| e.to_string())?;

    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Failed to read {}: {}", physical.to_string_lossy(), e))
}

pub fn kill_process(process: &str) -> Result<(), String> {
    sh(&format!("pkill -9 -f {process}"), &format!("Failed to kill {process}!"))?;

    Ok(())
}

pub fn reboot() -> Result<(), String> {
    Command::new("sh")
        .args(["-c", "sleep 3 && reboot"])
        .spawn()
        .map_err(|e| format!("Failed to reboot Kindle: {e}"))?;

    //Yay, we're dead. Who freaking cares.
    Ok(())
}