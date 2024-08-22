use serde::{Serialize, Deserialize};
use std::process::Command;
use std::path::PathBuf;
use basemodules::MiniModalError;
use anyhow::Result;

pub fn _declare_values_from_json(
    json: &serde_json::Value, 
    arg_types: &Vec<(String,String)>
) -> Result<String, MiniModalError> {
    let mut values = Vec::new();
    let json_obj = json.as_object().ok_or(MiniModalError::SerializationError("json is not an object".to_string()))?;
    
    for (name, value_type) in arg_types.iter() {
        let value = json_obj
            .get(name)
            .ok_or(MiniModalError::SerializationError(format!("key {} not found in json", name)))?;
        
        let declaration = format!(
            "let {}: {} = serde_json::from_value(serde_json::json!({}))?;", 
            name, value_type, value
        );
        values.push(declaration);
    }
    
    Ok(values.join("\n"))
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

pub fn check_code_compiles(code: String) -> Result<(bool, Option<String>)> {
    let wrapped_code = format!(r#"
        use serde_json;
        fn main() -> Result<(), Box<dyn std::error::Error>> {{
            {}
            Ok(())
        }}
        "#,
        code
    );

    let name = uuid::Uuid::new_v4().to_string();

    let current_dir = std::env::current_dir()?;
    let temp_file_path = write_bin_file(&name, &wrapped_code, &current_dir)?;
    let compile_output = Command::new("cargo")
        .args(["run", "--bin", &name])
        .output()?;

    std::fs::remove_file(temp_file_path)?;
    
    if compile_output.status.success() {
        Ok((true, None))
    } else {
        Ok((false, Some(String::from_utf8_lossy(&compile_output.stderr).to_string())))
    }
}

/// Writes a binary file to the src/bin directory
/// name: the name of the file
/// code: the code to write to the file
pub fn write_bin_file(name: &str, code: &str, project_dir_path: &PathBuf) -> Result<PathBuf> {
    let bin_dir = project_dir_path.join("src").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let temp_file_path = bin_dir.join(format!("{}.rs", name));
    println!("temp_file_path: {}", temp_file_path.display());

    match std::fs::write(&temp_file_path, code) {
        Ok(_) => Ok(temp_file_path),
        Err(e) => {
            let error_message = format!("Error writing file {}: {}", temp_file_path.display(), e);
            println!("🔥 Error: {}", error_message);
            Err(anyhow::anyhow!(error_message))
        }
    }
}