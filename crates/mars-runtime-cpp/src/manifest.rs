/// Generates a Conan manifest file from notebook dependencies.
pub fn generate_conanfile(dependencies: &std::collections::HashMap<String, String>) -> String {
    let mut conanfile = String::new();

    conanfile.push_str("[requires]\n");

    for (name, version) in dependencies {
        conanfile.push_str(name);
        conanfile.push('/');
        conanfile.push_str(version);
        conanfile.push('\n');
    }

    conanfile.push_str("\n[generators]\n");
    conanfile.push_str("CMakeDeps\n");
    conanfile.push_str("CMakeToolchain\n");

    conanfile.push_str("\n[options]\n");
    conanfile.push_str("*:shared=False\n");

    conanfile
}
