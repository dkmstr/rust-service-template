#[cfg(target_os = "windows")]
use windows::core::Result;

mod launcher;

#[cfg(target_os = "windows")]
mod windows_service;

#[cfg(target_os = "windows")]
fn main() -> Result<()> {
    crate::windows_service::run_service()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn main() {
    // Linux specific code
    println!("Linux launcher");
}

#[cfg(target_os = "macos")]
fn main() {
    // MacOS specific code
    println!("macOS launcher");
}
