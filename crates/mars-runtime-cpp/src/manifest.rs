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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_conanfile() {
        let mut deps = HashMap::new();
        deps.insert("nlohmann_json".to_string(), "3.11.2".to_string());
        deps.insert("eigen".to_string(), "3.4.0".to_string());

        let conanfile = generate_conanfile(&deps);

        assert!(conanfile.contains("[requires]"));
        assert!(conanfile.contains("nlohmann_json/3.11.2"));
        assert!(conanfile.contains("eigen/3.4.0"));
        assert!(conanfile.contains("[generators]"));
        assert!(conanfile.contains("CMakeDeps"));
    }

    #[test]
    fn test_empty_dependencies() {
        let deps = HashMap::new();
        let conanfile = generate_conanfile(&deps);

        assert!(conanfile.contains("[requires]"));
        assert!(conanfile.contains("[generators]"));
    }
}
