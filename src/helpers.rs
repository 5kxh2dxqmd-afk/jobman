use std::process::Command;

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
    Ok(())
}

//TODO: kill process func, read [cat] file func.