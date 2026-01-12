#[cfg(target_os = "windows")]
fn main() {
    use std::path::Path;

    let mut res = winres::WindowsResource::new();

    // Only set icon if it exists
    if Path::new("icon.ico").exists() {
        res.set_icon("icon.ico");
    }

    res.set("ProductName", "Transmute");
    res.set("FileDescription", "Media Converter with GPU Acceleration");
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // No-op on other platforms
}
