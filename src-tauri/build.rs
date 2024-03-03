fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=libvosk");
    }
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=vosk");
    }
    tauri_build::build()
}
