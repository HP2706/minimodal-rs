use std::fs;
use tonic::{transport::Server, Request, Response, Status};
use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RustFileResponse, 
    RunFunctionRequest, 
    RunFunctionResponse
};
use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
use minimodal_proto::proto::minimodal::mini_modal_server::{
    MiniModal, MiniModalServer
};
use base64; // Added for base64 decoding
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
use std::process::Command;
use std::path::Path;
use serde_json::{Value, json};

struct MiniModalService {
    project_dir_path: String,
}


impl MiniModalService {

    fn new(project_dir_path: String) -> MiniModalService {
        let service = MiniModalService {
            project_dir_path,
        };
        // build shadow dir
        service.build_shadow_dir();
        service
    }

    // store the shadow cargo project in server/project
    fn build_shadow_dir(&self) {
        let shadow_dir = self.project_dir_path.clone();
        if !Path::new(&shadow_dir).exists() {
            Command::new("cargo")
                .arg("new")
                .arg(shadow_dir)
                .output()
                .expect("Failed to create shadow cargo project");
        }
    }

    // add dependencies to the shadow cargo project
    fn add_dependencies(dependencies: Vec<String>) {
        Command::new("cargo")
            .arg("add")
            .args(dependencies)
            .output()
            .expect("Failed to add dependencies");
    }
}

#[tonic::async_trait]
impl MiniModal for MiniModalService {
    async fn send_rust_file(
        &self,
        request: Request<RustFileRequest>,
    ) -> Result<Response<RustFileResponse>, Status> {
        let rust_file = request.into_inner();
        let mut project_dir_path = self.project_dir_path.clone();
        
        let main_file_path = format!("{}/src/main.rs", project_dir_path);
        project_dir_path.push_str("/src/main.rs");

        // Decode the base64 encoded Rust file content
        let decoded_content = match general_purpose::STANDARD.decode(&rust_file.rust_file) {
            Ok(content) => content,
            Err(e) => {
                let error_message = format!("Error decoding base64 content: {}", e);
                eprintln!("{}", error_message);
                return Ok(Response::new(RustFileResponse { 
                    status: 1,
                    error_message,
                }));
            }
        };

        let dependencies = rust_file.dependencies;

        // Convert the decoded content to a string
        let rust_code = match String::from_utf8(decoded_content) {
            Ok(code) => code,
            Err(e) => {
                let error_message = format!("Error converting decoded content to string: {}", e);
                eprintln!("{}", error_message);
                return Ok(Response::new(RustFileResponse { 
                    status: 1,
                    error_message,
                }));
            }
        };

        // Write dependencies to shadow Cargo.toml
        let shadow_cargo_toml_path = format!("{}/Cargo.toml", self.project_dir_path);
        let mut cargo_toml_content = fs::read_to_string(&shadow_cargo_toml_path)
            .map_err(|e| {
                let error_message = format!("Error reading Cargo.toml: {}", e);
                eprintln!("{}", error_message);
                Status::internal(error_message)
            })?;

        // Find the [dependencies] section or add it if it doesn't exist
        if !cargo_toml_content.contains("[dependencies]") {
            cargo_toml_content.push_str("\n[dependencies]\n");
        } else {
            // If [dependencies] exists, ensure we're appending after it
            let deps_index = cargo_toml_content.find("[dependencies]").unwrap();
            cargo_toml_content.truncate(deps_index + 14); // 14 is the length of "[dependencies]"
            cargo_toml_content.push('\n');
        }

        // Append new dependencies
        cargo_toml_content.push_str(&dependencies.join("\n"));
        // Write updated content back to Cargo.toml
        fs::write(&shadow_cargo_toml_path, cargo_toml_content).map_err(|e| {
            let error_message = format!("Error writing to Cargo.toml: {}", e);
            eprintln!("{}", error_message);
            Status::internal(error_message)
        })?;

        match fs::write(&*main_file_path, rust_code) {
            Ok(_) => Ok(Response::new(RustFileResponse { 
                status: 0,
                error_message: "".to_string(),
            })),
            Err(e) => {
                let error_message = format!("Error writing file: {}", e);
                eprintln!("{}", error_message);
                Ok(Response::new(RustFileResponse { 
                    status: 1,
                    error_message,
                }))
            }
        }


    }

    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        let req = request.into_inner();
        println!("🏃‍ Running function: {}", req.function_id);
        let project_dir_path = self.project_dir_path.clone();

        println!("📦 Loading app: {}", project_dir_path);

        // Correctly construct the path to the main.rs file
        let main_file_path = format!("{}/src/main.rs", project_dir_path);
        println!("👉 Reading main file from {}", main_file_path);
        // Read the original Rust file
        let original_code = fs::read_to_string(&main_file_path)
            .map_err(|e| Status::internal(format!("Failed to read Rust file: {}", e)))?;

        let deserialized_inputs: Value = serde_json::from_str(&req.serialized_inputs)
            .map_err(|e| Status::internal(format!("Failed to deserialize inputs: {}", e)))?;
        // Modify the main function to return the result as JSON
        let deps = "use serde_json::{json, Value};\nuse anyhow::Error;";

        let main_code = format!(
            r#"//imports
    {}

    // Custom macro to print the result
    macro_rules! print_result {{
        ($result:expr) => {{
            let json_result = match $result {{
                Ok(value) => json!({{ "success": value }}),
                Err(e) => json!({{ "error": e.to_string() }}),
            }};
            println!("RESULT_START{{}}RESULT_END", json_result);
        }}
    }}

    //the original code
    {}
    #[tokio::main(flavor = "current_thread")]
    async fn main() -> () {{
        let inputs = serde_json::json!({});
        let result: {} = {}(inputs).await;
        print_result!(result);
    }}
    "#,
            deps,
            original_code,
            deserialized_inputs,
            req.output_type,
            req.function_id,
        );

        fs::write(main_file_path, main_code)
            .map_err(|e| Status::internal(format!("Failed to write temporary file: {}", e)))?;

        // Compile and run the code
        let output = std::process::Command::new("cargo")
            .current_dir(&project_dir_path)
            .arg("run")
            .output()
            .map_err(|e| Status::internal(format!("Failed to run cargo: {}", e)))?;
        
        if !output.status.success() {
            let error_message = format!("cargo run failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(Status::internal(error_message));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result = stdout
            .split("RESULT_START")
            .nth(1)
            .and_then(|s| s.split("RESULT_END").next())
            .ok_or_else(|| Status::internal(format!("Failed to parse output: {}", stdout)))?;

        let json_result: serde_json::Value = serde_json::from_str(result)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        // Create the response based on the JSON structure
        let response = if let Some(success) = json_result.get("success") {
            RunFunctionResponse {
                result: Some(RunFunctionResult::Success(success.to_string())),
            }
        } else if let Some(error) = json_result.get("error") {
            RunFunctionResponse {
                result: Some(RunFunctionResult::Error(error.to_string())),
            }
        } else {
            return Err(Status::internal("Invalid JSON result structure"));
        };

        Ok(Response::new(response))
    }
}


// run server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MiniModalService::new("src/server/project".to_string());

    println!("🎬 Starting up minimodal server");
    println!(" Listening on {}", addr);

    Server::builder()
        .add_service(MiniModalServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}