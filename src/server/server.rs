use std::fs;
use tonic::{transport::Server, Request, Response, Status};
use minimodal_proto::proto::minimodal::{
    MountProjectResponse,
    MountProjectRequest,
    FileEntry,
    RunFunctionRequest, 
    RunFunctionResponse,
};
use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
use minimodal_proto::proto::minimodal::mount_project_response::Result as MountProjectResult;
use minimodal_proto::proto::minimodal::mini_modal_server::{
    MiniModal, MiniModalServer
};
use base64; // Added for base64 decoding
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
use std::process::Command;
use std::path::Path;
use serde_json::{Value, json};
use std::env;

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

fn generate_function_args(inputs: &serde_json::Value) -> String {
    match inputs {
        serde_json::Value::Object(map) => {
            map.iter()
                .map(|(_ , value)| format!("{}", serde_json::to_string(value).unwrap()))
                .collect::<Vec<String>>()
                .join(", ")
        }
        _ => "inputs.clone()".to_string(), // Fallback for non-object inputs
    }
}

#[tonic::async_trait]
impl MiniModal for MiniModalService {
    
    async fn mount_project(
        &self,
        request: Request<MountProjectRequest>,
    ) -> Result<Response<MountProjectResponse>, Status> {
        let req = request.into_inner();
        let project_dir_path = self.project_dir_path.clone();
        let shadow_dir = format!("{}", project_dir_path);

        for file_entry in req.files.into_iter() {
            let file_path = file_entry.file_path;
            let file_path = format!("{}/{}", shadow_dir, file_path);
            // Create intermediate directories if they don't exist
            if let Some(parent) = Path::new(&file_path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Status::internal(format!("Failed to create directories: {}", e)))?;
            }
            match fs::write(file_path, file_entry.content) {
                Ok(_) => (),
                Err(e) => return Err(Status::internal(format!("Failed to write file: {}", e))),
            }
        }

        Ok(Response::new(MountProjectResponse {
            result: Some(MountProjectResult::Success("Mounted project".to_string())),
        }))
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
        let original_main_file_path = format!("{}/src/original_main.rs", project_dir_path);
        println!("👉 Reading main file from {}", original_main_file_path);

        // Read the original Rust file
        let original_code = fs::read_to_string(&original_main_file_path)
            .map_err(|e| Status::internal(format!("Failed to read Rust file: {}", e)))?;

        let deserialized_inputs: Value = serde_json::from_str(&req.serialized_inputs)
            .map_err(|e| Status::internal(format!("Failed to deserialize inputs: {}", e)))?;
        // Modify the main function to return the result as JSON
        let deps = "use serde_json::{json, Value};\n";

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

    // the original code
    {}
    #[tokio::main(flavor = "current_thread")]
    async fn main() -> () {{
        let inputs: serde_json::Value = serde_json::json!({});
        
        let result: {} = match {}(
            {}
        ).await {{
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        }};
        
        print_result!(result);
    }}
    "#,
            deps,
            original_code,
            deserialized_inputs,
            req.output_type,
            req.function_id,
            generate_function_args(&deserialized_inputs),
        );
        let main_file_path = format!("{}/src/main.rs", project_dir_path);
        fs::write(&main_file_path, main_code)
            .map_err(|e| Status::internal(format!("Failed to write file: {}", e)))?;

        // Compile and run the code
        let output = std::process::Command::new("cargo")
            .current_dir(&project_dir_path)
            .args(&["run", "--bin", "minimodal-rs"])
            .output()
            .map_err(|e| Status::internal(format!("Failed to run cargo: {}", e)))?;
        
        if !output.status.success() {
            let error_message = format!("cargo run failed: {}", String::from_utf8_lossy(&output.stderr));
            println!("🔥 Error: {}", error_message);
            return Err(Status::internal(error_message));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result = stdout
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

        //remove the main.rs file
        fs::remove_file(main_file_path)
            .map_err(
                |e| 
                Status::internal(format!("Failed to remove temporary file: {}", e))
        )?;

        println!("🔥 Response: {:?}", response);
        Ok(Response::new(response))
    }
}

// run server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let args: Vec<String> = env::args().collect();
    let dirname = args.iter().position(|arg| arg == "-dirname")
        .and_then(|index| args.get(index + 1))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "src/server/shadow_dir".to_string());
    println!("🔧 Shadow dir: {}", dirname);
    let service = MiniModalService::new(dirname);

    println!("🎬 Starting up minimodal server");
    println!(" Listening on {}", addr);

    Server::builder()
        .add_service(MiniModalServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}