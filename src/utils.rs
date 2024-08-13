


use std::fs;
use cargo_toml::{Manifest, Dependency};

/// Basic way to find the Cargo.toml and parse the dependencies
pub fn get_dependencies() -> Vec<String> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Search for Cargo.toml in current and parent directories
    while !current_dir.join("Cargo.toml").exists() {
        if !current_dir.pop() {
            panic!("Cargo.toml not found in any parent directory");
        }
    }

    let cargo_toml_path = current_dir.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    let manifest = Manifest::from_str(&cargo_toml_content)
        .expect("Failed to parse Cargo.toml");

    // Extract dependencies
    println!("Dependencies: {:?}", manifest.dependencies);
    let dependencies = manifest.dependencies
        .iter()
        .filter_map(|(name, dep)| {
            match dep {
                Dependency::Simple(version) => Some(format!("{}=\"{}\"", name, version)),
                Dependency::Detailed(detail) => {
                    if detail.path.is_some() || detail.git.is_some() {
                        None // Skip relative or git dependencies
                    } else {
                        detail.version.as_ref().map(|v| format!("{}=\"{}\"", name, v))
                    }
                },
                Dependency::Inherited(_) => None, // Skip inherited dependencies
            }
        })
        .collect::<Vec<String>>();
    dependencies
}