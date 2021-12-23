#[cfg(target_os = "linux")]
pub fn os_native_name() -> &'static str {
    "natives-linux"
}

#[cfg(target_os = "windows")]
pub fn os_native_name() -> &'static str {
    "natives-windows"
}

#[cfg(target_os = "macos")]
pub fn os_native_name() -> &'static str {
    "natives-osx"
}
