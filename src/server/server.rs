use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RustFileResponse, 
    RunFunctionRequest, 
    RunFunctionResponse
};
use minimodal_proto::proto::minimodal::mini_modal_server::{
    MiniModal, MiniModalServer
};
use base64; // Added for base64 decoding
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
struct MiniModalService {
    script_path: Arc<Mutex<String>>,
}

#[tonic::async_trait]
impl MiniModal for MiniModalService {
    async fn send_rust_file(
        &self,
        request: Request<RustFileRequest>,
    ) -> Result<Response<RustFileResponse>, Status> {
        let rust_file = request.into_inner();
        let mut script_path = self.script_path.lock().await;
        *script_path = "/tmp/app.rs".to_string();

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

        match fs::write(&*script_path, rust_code) {
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
        let script_path = self.script_path.lock().await.clone();

        println!("📦 Loading app: {}", script_path);

        // Read the original Rust file
        let original_code = fs::read_to_string(&script_path)
            .map_err(|e| Status::internal(format!("Failed to read Rust file: {}", e)))?;

        // Create a new file with a main function that calls the requested function
        let main_code = format!(
            r#"
{}

fn main() {{
    let result = {}({});
    println!("{{:?}}", result);
}}
"#,
            original_code,
            req.function_id,
            req.inputs
        );

        let temp_file_path = "/tmp/app_with_main.rs";
        fs::write(temp_file_path, main_code)
            .map_err(|e| Status::internal(format!("Failed to write temporary file: {}", e)))?;

        // Compile the new Rust file
        let output = std::process::Command::new("rustc")
            .arg(temp_file_path)
            .arg("-o")
            .arg("/tmp/app")
            .output()
            .expect("Failed to compile Rust file");

        if !output.status.success() {
            let error_message = format!(
                "🚨 Compilation failed. Status: {}, Stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", error_message);
            return Ok(Response::new(RunFunctionResponse {
                result: String::new(),
                error_message,
            }));
        }

        println!("🏃‍ Running function: {}", req.function_id);

        // Run the compiled binary
        let output = std::process::Command::new("/tmp/app")
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            println!("🏁 Result: {}", result);
            Ok(Response::new(RunFunctionResponse { 
                result,
                error_message: String::new(),
            }))
        } else {
            let error_message = format!(
                "🚨 Function execution failed. Status: {}, Stderr: {}, Stdout: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            println!("{}", error_message);
            Ok(Response::new(RunFunctionResponse {
                result: String::new(),
                error_message,
            }))
        }
    }
}


// run server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MiniModalService {
        script_path: Arc::new(Mutex::new(String::new())),
    };

    println!("🎬 Starting up minimodal server");
    println!(" Listening on {}", addr);

    Server::builder()
        .add_service(MiniModalServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}