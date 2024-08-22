use serde::{Serialize, Deserialize};
use std::process::Command;
use std::io::Write;
use basemodules::MiniModalError;
use anyhow::Result;
use tempfile::NamedTempFile;
use std::env;

pub fn declare_values_from_json(json: serde_json::Value, arg_types: Vec<(String,String)>) -> Result<String, MiniModalError> {
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

    let current_dir = std::env::current_dir()?;
    // Create the src/bin directory if it doesn't exist
    let bin_dir = current_dir.join("src").join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let temp_file_path = current_dir.join(bin_dir.join("temp.rs"));
    
    // Delete the contents of the file if it exists
    if temp_file_path.exists() {
        std::fs::write(&temp_file_path, "fn main() {}")?;
    }
    
    // Write the new code to the file
    std::fs::write(&temp_file_path, wrapped_code)?;

    let compile_output = Command::new("cargo")
        .args(["run", "--bin", "temp"])
        .output()?;

    //std::fs::remove_file(temp_file_path)?;
    
    if compile_output.status.success() {
        Ok((true, None))
    } else {
        Ok((false, Some(String::from_utf8_lossy(&compile_output.stderr).to_string())))
    }
}