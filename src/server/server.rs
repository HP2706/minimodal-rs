use crate::utilities::{_declare_values_from_json, write_bin_file};
use std::fs;
use std::pin::Pin;
use tonic::{transport::Server, Request, Response, Status};
use minimodal_proto::proto::minimodal::{
    MountProjectResponse,
    MountProjectRequest,
    FileEntry,
    RunFunctionRequest, 
    RunFunctionResponse,
};
use minimodal_proto::proto::minimodal::run_function_response::Response as RunFunctionResult;
use minimodal_proto::proto::minimodal::TaskResult;
use minimodal_proto::proto::minimodal::mount_project_response::Result as MountProjectResult;
use minimodal_proto::proto::minimodal::mini_modal_server::{
    MiniModal, MiniModalServer
};
use base64; // Added for base64 decoding
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
use std::process::Command;
use std::path::Path;
use serde_json::{Value, json};
use futures::stream::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use duct::cmd;
use std::io::{BufRead, BufReader, Lines};
pub struct MiniModalService {
    project_dir_path: String,
    tx: Option<mpsc::Sender<Result<RunFunctionResponse, Status>>>,
}

impl MiniModalService {
    pub fn new(project_dir_path: String) -> MiniModalService {
        let service = MiniModalService {
            project_dir_path,
            tx: None,
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
    type RunFunctionStream = Pin<Box<dyn Stream<Item = Result<RunFunctionResponse, Status>> + Send>>;

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
    ) -> Result<Response<Self::RunFunctionStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = mpsc::channel(4);
        let logger = Logger::new(&tx);

        logger.log(&format!("🏃‍ Running function: {}", req.function_id)).await;

        let project_dir_path = self.project_dir_path.clone();

        logger.log(&format!("📦 Loading app: {}", project_dir_path)).await;

        // Correctly construct the path to the main.rs file
        let original_main_file_path = format!("{}/src/original_main.rs", project_dir_path);
        logger.log(&format!("👉 Reading main file from {}", original_main_file_path)).await;

        // Read the original Rust file
        let original_code = fs::read_to_string(&original_main_file_path)
            .map_err(|e| Status::internal(format!("Failed to read file: {}", e)))?;

        let deserialized_inputs: Value = serde_json::from_str(&req.serialized_inputs)
            .map_err(|e| Status::internal(format!("Failed to deserialize inputs: {}", e)))?;
        // Modify the main function to return the result as JSON

        let str_field_types = req.field_types.iter().map(|field| (field.name.clone(), field.ty.clone())).collect::<Vec<(String, String)>>();

        let let_declarations = _declare_values_from_json(
            &deserialized_inputs, 
            &str_field_types
        ).map_err(|e| Status::internal(format!("Failed to declare values: {}", e)))?;

        let main_code = format_code(
            original_code, 
            deserialized_inputs, 
            let_declarations, 
            str_field_types, 
            &req
        );

        let name = uuid::Uuid::new_v4().to_string();
        logger.log(&format!("👉 Writing bin file to {}", project_dir_path)).await;

        let _ = write_bin_file(&name, &main_code, &project_dir_path.clone().into()) 
            .map_err(|e| Status::internal(format!("Failed to write bin file: {}", e)))?;
        
        // Compile and run the code using duct
        logger.log(&format!("project_dir_path: {}", project_dir_path)).await;
        let cmd = cmd!("cargo", "run", "--bin", &name)
            .dir(&project_dir_path)
            .stdout_capture()
            .stderr_capture();

        let output = cmd.run().map_err(|e| Status::internal(format!("Failed to run cargo: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let response_stream = ReceiverStream::new(rx);



            for line in stdout.lines() {
                if tx.send(Ok(RunFunctionResponse {
                    response: Some(RunFunctionResult::LogLine(line.to_string())),
                })).await.is_err() {
                    break;
                }
            }

            if !output.status.success() {
                let error_message = format!("cargo run failed: {}", stderr);
                println!("🔥 Error: {}", error_message);
                logger.log(&error_message).await;
            } else {
                let result = stdout
                    .split("RESULT_START")
                    .nth(1)
                    .and_then(|s| s.split("RESULT_END").next())
                    .unwrap_or_else(|| "");

                let json_result: serde_json::Value = serde_json::from_str(result).unwrap_or_else(|_| json!({}));

                let response = if let Some(success) = json_result.get("success") {
                    RunFunctionResponse {
                        response: Some(RunFunctionResult::Result(TaskResult {
                            success: true,
                            message: success.to_string(),
                        })),
                    }
                } else if let Some(error) = json_result.get("error") {
                    RunFunctionResponse {
                        response: Some(RunFunctionResult::Result(TaskResult {
                            success: false,
                            message: error.to_string(),
                        })),
                    }
                } else {
                    RunFunctionResponse {
                        response: Some(RunFunctionResult::Result(TaskResult {
                            success: false,
                            message: "Invalid JSON result structure".to_string(),
                        })),
                    }
                };

                let _ = tx.send(Ok(response)).await;
            }

        Ok(Response::new(Box::pin(response_stream) as Self::RunFunctionStream))
    }
}




fn format_code(
    original_code: String, 
    deserialized_inputs: Value, 
    let_declarations: String, 
    str_field_types: Vec<(String, String)>, 
    req: &RunFunctionRequest
) -> String {
    format!(
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
    )
}

struct Logger<'a> {
    tx: &'a mpsc::Sender<Result<RunFunctionResponse, Status>>,
}

impl<'a> Logger<'a> {
    pub fn new(tx: &'a mpsc::Sender<Result<RunFunctionResponse, Status>>) -> Logger<'a> {
        Logger { tx }
    }

    pub async fn log(&self, message: &str) {
        // print to console
        println!("{}", message);

        // send to client
        let _ = self.tx.send(Ok(RunFunctionResponse {
            response: Some(RunFunctionResult::LogLine(message.to_string())),
        })).await;
    }
}