fn main() {
    mars_kernel::runtime_hook();
    mars_desktop_lib::run().expect("failed to run Mars desktop application");
}
