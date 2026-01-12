#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico"); // If you have an icon file
    res.set("ProductName", "Transmute");
    res.set("FileDescription", "Media Converter with GPU Acceleration");
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // No-op on other platforms
}
