use std::fs;
use cargo_toml::{Manifest, Dependency};
use serde::{Serialize, Deserialize};
use syn::{meta, parse_str, Type};
use anyhow::Error;
use base64::{engine::general_purpose, Engine as _};
use cargo_metadata::MetadataCommand;
use ignore::WalkBuilder;
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use std::path::PathBuf;
use tonic::Request;
use std::collections::HashMap;
use std::borrow::Cow;

pub fn build_cargo_toml(cargo_toml_content : &mut String , metadata: &cargo_metadata::Metadata) -> Result<(), Error> {
    let dependencies = get_dependencies_from_cargo_toml(&metadata);

    if !cargo_toml_content.contains("[dependencies]") {
        *cargo_toml_content = format!("{}\n[dependencies]\n", cargo_toml_content.clone());
    } else {
        if let Some(deps_index) = cargo_toml_content.find("[dependencies]") {
            *cargo_toml_content = format!("{}[dependencies]\n", &cargo_toml_content[..deps_index]);
        }
    }
    // Append new dependencies
    *cargo_toml_content = format!("{}{}", cargo_toml_content, dependencies.join("\n"));
    Ok(())
}

pub fn get_project_structure(filter_entries : Vec<String>) -> Result<HashMap<String, String>, Error> {
    let metadata = MetadataCommand::new()
        .exec()?;

    // Walk through project files
    let walker = WalkBuilder::new(&metadata.workspace_root)
        .hidden(false)
        .git_ignore(true)
        .build();

    let mut hashmap  = HashMap::new();

    for entry in walker.filter_map(Result::ok).filter_map(|entry| {
        let relative_path = entry.path().strip_prefix(&metadata.workspace_root).ok()?;
        let relative_path_str = relative_path.to_string_lossy().to_string();
        
        if filter_entries.iter().any(|filter| relative_path_str.starts_with(filter)) {
            None
        } else {
            Some(entry)
        }
    }) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&metadata.workspace_root)?;
        if filter_entries.contains(&relative_path.to_string_lossy().to_string()) {
            continue;
        }
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(&metadata.workspace_root)?;
            let content = std::fs::read(path)?;
            let content = String::from_utf8_lossy(&content).to_string();
            hashmap.insert(relative_path.to_string_lossy().to_string(), content);
        }
    }

    let cargo_toml_keys: Vec<String> = hashmap.keys()
        .filter(|k| k.ends_with("Cargo.toml"))
        .cloned()
        .collect();

    if cargo_toml_keys.is_empty() || cargo_toml_keys.len() > 1 {
        return Err(anyhow::anyhow!(
            format!("No Cargo.toml or multiple Cargo.toml found in the project: {}", cargo_toml_keys.len())
        ));
    }
    
    // Write dependencies to shadow Cargo.toml
    let cargo_toml_key = &cargo_toml_keys[0];
    let cargo_toml_content = match hashmap.get_mut(cargo_toml_key) {
        Some(content) => content,
        None => return Err(anyhow::anyhow!(format!("Cargo.toml key {} not found in keys {}", cargo_toml_key , hashmap.keys().map(|k| k.clone()).collect::<Vec<String>>().join(", "))))
    };
    build_cargo_toml(cargo_toml_content, &metadata)?;
    Ok(hashmap)
}

pub fn mount_project(filter_entries : Vec<String>) -> Result<HashMap<String, String>, Error> {
    let hashmap = get_project_structure(filter_entries)?;
    //TODO : implement some sort of caching here:
    Ok(hashmap)
}

pub fn get_dependencies_from_cargo_toml(metadata: &cargo_metadata::Metadata) -> Vec<String> {
    return metadata.packages.iter()
        .map(|p| format!("{}={}", p.name.clone(), p.version.to_string()))
        .collect::<Vec<String>>();    
}

/// Basic way to find the Cargo.toml and parse the dependencies
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