fn main() {
    let attrs = tauri_build::Attributes::new();
    if let Err(e) = tauri_build::try_build(attrs) {
        println!("cargo:warning=tauri_build failed (resources skipped): {}", e);
    }
}
