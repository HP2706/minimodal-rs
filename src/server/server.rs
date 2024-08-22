use crate::utilities::{_declare_values_from_json, write_bin_file};
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

pub struct MiniModalService {
    project_dir_path: String,
}


impl MiniModalService {

    pub fn new(project_dir_path: String) -> MiniModalService {
        let service = MiniModalService {
            project_dir_path,
        };
        // build shadow dir
        service.build_shadow_dir();
        service
    }

    // store the shadow cargo project in server/project
    pub fn build_shadow_dir(&self) {
        let shadow_dir = self.project_dir_path.clone();
        if !Path::new(&shadow_dir).exists() {
            Command::new("cargo")
                .arg("new")
                .arg(shadow_dir)
                .output()
                .expect("Failed to create shadow cargo project");
        }
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

        let str_field_types = req.field_types.iter().map(|field| (field.name.clone(), field.ty.clone())).collect::<Vec<(String, String)>>();

        let let_declarations = _declare_values_from_json(
            &deserialized_inputs, 
            &str_field_types
        ).map_err(|e| Status::internal(format!("Failed to declare values: {}", e)))?;

        let main_code = format!(
            r#"//imports
    
    {original_code}

    // Custom macro to print the result
    macro_rules! print_result {{
        ($result:expr) => {{
            let json_result = match $result {{
                Ok(value) => serde_json::json!({{ "success": value }}),
                Err(e) => serde_json::json!({{ "error": e.to_string() }}),
            }};
            println!("RESULT_START{{}}RESULT_END", json_result);
        }}
    }}

    // the original code
    #[tokio::main(flavor = "current_thread")]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {{
        let inputs: serde_json::Value = serde_json::json!({deserialized_inputs});
        
        {declarations}
        let result: {output_type} = match {function_id}(
            {args}
        ).await {{
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        }};
        
        print_result!(result);
        Ok(())
    }}
    "#,
            original_code=original_code,
            deserialized_inputs=deserialized_inputs,
            declarations=let_declarations,
            args=format!("{}", str_field_types.iter().map(|field| format!("{}", field.0)).collect::<Vec<String>>().join(", ")),
            output_type=req.output_type,
            function_id=req.function_id,
        );

        let name = uuid::Uuid::new_v4().to_string();
        println!("👉 Writing bin file to {}", project_dir_path);
        let temp_file_path = write_bin_file(&name, &main_code, &project_dir_path.clone().into()) 
            .map_err(|e| Status::internal(format!("Failed to write bin file: {}", e)))?;
        
        // Compile and run the code
        println!("project_dir_path: {}", project_dir_path);
        let output = std::process::Command::new("cargo")
            .current_dir(&project_dir_path)
            .args(&["run", "--bin", &name])
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

        /* //remove the temp.rs file
        fs::remove_file(temp_file_path)
            .map_err(
                |e| 
                Status::internal(format!("Failed to remove temporary file: {}", e))
        )?; */

        println!("🔥 Response: {:?}", response);
        Ok(Response::new(response))
    }
}
