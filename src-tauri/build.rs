fn main() {
    println!("cargo:rustc-link-search=../src-tauri");
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=libvosk");
    }
    #[cfg(target_os = "macos")]
    {
        //println!("cargo:rustc-link-lib=vosk");
        println!("cargo:rustc-link-lib=static=vosk");
    }
    tauri_build::build()
}
