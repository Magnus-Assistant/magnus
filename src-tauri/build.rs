fn main() {
    println!("cargo:rustc-link-search=../src-tauri");
    println!("cargo:rustc-link-lib=vosk");
    tauri_build::build()
}
