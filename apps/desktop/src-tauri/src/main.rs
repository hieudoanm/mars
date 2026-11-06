fn main() {
    mars_kernel::runtime_hook();
    mars_tauri_lib::run().expect("failed to run Mars desktop application");
}
