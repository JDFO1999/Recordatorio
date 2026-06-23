fn main() {
    let attrs = tauri_build::Attributes::new();
    if let Err(e) = tauri_build::try_build(attrs) {
        println!("cargo:warning=tauri_build failed (resources skipped): {}", e);
    }

    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_release = manifest_dir.join("target").join("release");

    // Copy libstdc++-6.dll (MinGW)
    let mingw_dll = r"C:\Users\Alkosto\mingw64\mingw64\bin\libstdc++-6.dll";
    if std::path::Path::new(mingw_dll).exists() {
        let _ = std::fs::copy(mingw_dll, target_release.join("libstdc++-6.dll"));
    }

    // Copy WebView2Loader.dll from webview2-com-sys build directory
    let build_dir = target_release.join("build");
    if let Ok(entries) = std::fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("webview2-com-sys-") {
                let dll = entry.path().join("out").join("x64").join("WebView2Loader.dll");
                if dll.exists() {
                    let _ = std::fs::copy(&dll, target_release.join("WebView2Loader.dll"));
                }
                break;
            }
        }
    }
}
