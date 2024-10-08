use std::fs;
use cargo_toml::{Manifest, Dependency};
use anyhow::Error;
use cargo_metadata::MetadataCommand;
use ignore::WalkBuilder;
use minimodal_proto::proto::minimodal::{
    MountProjectRequest, 
    FileEntry, 
    MountProjectResponse, 
    mini_modal_client::MiniModalClient
};
use tonic::transport::Channel;
use std::path::PathBuf;
use std::collections::HashMap;
use toml;
use crate::parse_file::{remove_macro, remove_function};

pub fn build_cargo_toml(
    cargo_toml_content : &mut Vec<u8>,
) -> Result<(), Error> {
    let mut cargo_toml_content_str = std::str::from_utf8_mut(cargo_toml_content).unwrap().to_string();

    let mut manifest = Manifest::from_str(&cargo_toml_content_str)
        .expect("Failed to parse Cargo.toml");

    let modified_toml = toml::to_string(&manifest)?;

    // Update the cargo_toml_content with the new TOML string
    *cargo_toml_content = modified_toml.into_bytes();
    Ok(())
}

pub fn handle_main_rs(work_space_root : PathBuf) -> Result<Vec<u8>, Error> {
    let main_rs_path = work_space_root.join("src/main.rs");
    let content = if main_rs_path.exists() {
        match fs::read(&main_rs_path) {
            Ok(content) => {
                Ok(content)
            },
            Err(e) => {
                Err(anyhow::anyhow!(format!("Failed to read src/main.rs: {}", e)))
            }
        } 
    } else {
        Err(anyhow::anyhow!("src/main.rs does not exist"))
    }?;

    let new_content_str = String::from_utf8(content).unwrap();
    let mut ast = syn::parse_file(&new_content_str).unwrap();

    //TODO find a way to avoid manually adding the macro names here
    remove_macro(
        &mut ast, 
        vec!["function", "mount", "function_experiment"].iter().map(|s| s.to_string()).collect()
    );

    remove_function(
        &mut ast, 
        "main"
    );

    Ok(prettyplease::unparse(&ast).into_bytes())
}

pub fn get_project_structure(filter_entries : Vec<String>) -> Result<HashMap<String, Vec<u8>>, Error> {
    let metadata = MetadataCommand::new()
        .exec()?;

    // Walk through project files
    let walker = WalkBuilder::new(&metadata.workspace_root)
        .hidden(false)
        .git_ignore(true)
        .build();

    let mut hashmap : HashMap<String, Vec<u8>> = HashMap::new();

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
            hashmap.insert(relative_path.to_string_lossy().to_string(), content);
        }
    }
    
    hashmap.insert("src/original_main.rs".to_string(), handle_main_rs(metadata.workspace_root.into())?);


    let cargo_toml_content = match hashmap.get_mut(&"Cargo.toml".to_string()) {
        Some(cargo_toml_content) => cargo_toml_content,
        None => return Err(
            anyhow::anyhow!(
                format!(
                    "Cargo.toml not found in the project: used key: \"Cargo.toml\" out of all keys: {:?}", 
                    hashmap.keys()
                )
            )
        ),
    };

    //pass mutable reference of cargo_toml_content to build_cargo_toml
    build_cargo_toml(cargo_toml_content)?;
    Ok(hashmap)
}


pub async fn mount_project(
    client: &mut MiniModalClient<Channel>,
    filter_entries: Vec<String>,
) -> Result<MountProjectResponse, Error> {
    let hashmap = get_project_structure(filter_entries.clone())?;

    let files: Vec<FileEntry> = hashmap.into_iter()
        .map(|(file_path, content)| FileEntry {
            file_path,
            content,
        })
        .collect();

    let request = MountProjectRequest {
        files,
    };

    match client.mount_project(request).await {
        Ok(response) => {
            Ok(response.into_inner())
        },
        Err(e) => {
            Err(anyhow::anyhow!(format!("Failed to mount project: {}", e)))
        }
    }
}
