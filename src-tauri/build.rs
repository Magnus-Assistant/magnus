fn main() {
    println!("cargo:rustc-link-lib=libvosk");
    tauri_build::build()
}
