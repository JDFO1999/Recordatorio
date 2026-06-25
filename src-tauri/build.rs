fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_release = manifest_dir.join("target").join("release");

    // Copy WebView2Loader.dll from webview2-com-sys build directory (before tauri_build)
    let build_dir = target_release.join("build");
    if let Ok(entries) = std::fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("webview2-com-sys-") {
                let dll = entry.path().join("out").join("x64").join("WebView2Loader.dll");
                if dll.exists() {
                    // Copy to target/release/ for cargo run development
                    let _ = std::fs::copy(&dll, target_release.join("WebView2Loader.dll"));
                    // Copy to src-tauri/ so NSIS places it next to the exe
                    let _ = std::fs::copy(&dll, manifest_dir.join("WebView2Loader.dll"));
                }
                break;
            }
        }
    }

    // Copy libstdc++-6.dll and libgcc_s_seh-1.dll from MinGW to src-tauri/ (for NSIS installer)
    let mingw_bin = std::path::PathBuf::from(r"C:\Users\Alkosto\mingw64\mingw64\bin");
    for dll_name in ["libstdc++-6.dll", "libgcc_s_seh-1.dll"] {
        let src = mingw_bin.join(dll_name);
        if src.exists() {
            let _ = std::fs::copy(&src, target_release.join(dll_name));
            let _ = std::fs::copy(&src, manifest_dir.join(dll_name));
        }
    }

    let attrs = tauri_build::Attributes::new();
    if let Err(e) = tauri_build::try_build(attrs) {
        println!("cargo:warning=tauri_build failed (resources skipped): {}", e);
    }
}
