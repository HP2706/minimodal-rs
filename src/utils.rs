use std::fs;
use cargo_toml::{Manifest, Dependency};
use serde::{Serialize, Deserialize};
use syn::{Type, parse_str};
use quote::ToTokens;

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

pub fn serialize_inputs<'a>(
    arg_names: &[&str], 
    arg_values: &[&dyn erased_serde::Serialize]
) -> Result<String, serde_json::Error> {
    use serde_json::json;
    
    let mut map = serde_json::Map::new();
    for (name, value) in arg_names.iter().zip(arg_values.iter()) {
        map.insert(name.to_string(), json!(value));
    }
    
    serde_json::to_string(&map)
}

pub fn deserialize_inputs<'a, T: Serialize + Deserialize<'a>>(
    serialized_inputs: &'a str
) -> Result<T, serde_json::Error> {
    serde_json::from_str(serialized_inputs)
}


pub fn extract_left_type(return_type: String) -> syn::Type {
    let parsed_type = syn::parse_str::<syn::Type>(&return_type)
        .expect(&format!("Failed to parse return type: {}", return_type));
    
    if let syn::Type::Path(type_path) = &parsed_type {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(left_type)) = args.args.first() {
                        return left_type.clone();
                    }
                }
            }
        }
    }
    
    parsed_type
}