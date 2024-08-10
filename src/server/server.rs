use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use minimodal::{RustFileRequest, RustFileResponse, RunFunctionRequest, RunFunctionResponse};
pub mod minimodal {
    tonic::include_proto!("minimodal"); // The string specified here must match the proto package name
}

use crate::minimodal::mini_modal_server::{MiniModal, MiniModalServer};

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

        match fs::write(&*script_path, rust_file.rust_file) {
            Ok(_) => Ok(Response::new(RustFileResponse { status: 0 })),
            Err(e) => {
                eprintln!("Error: {}", e);
                Ok(Response::new(RustFileResponse { status: 1 }))
            }
        }
    }

    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        let req = request.into_inner();
        let script_path = self.script_path.lock().await.clone();

        println!("📦 Loading app: {}", script_path);

        // Compile the Rust file
        let output = std::process::Command::new("rustc")
            .arg(&script_path)
            .arg("-o")
            .arg("/tmp/app")
            .output()
            .expect("Failed to compile Rust file");

        if !output.status.success() {
            return Err(Status::internal("Failed to compile Rust file"));
        }

        println!("🏃‍ Running function: {}", req.function_id);

        // Run the compiled binary
        let output = std::process::Command::new("/tmp/app")
            .arg(&req.function_id)
            .arg(&req.inputs)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            println!("🏁 Result: {}", result);
            Ok(Response::new(RunFunctionResponse { result }))
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            Err(Status::internal(format!("Function execution failed: {}", error)))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MiniModalService {
        script_path: Arc::new(Mutex::new(String::new())),
    };

    println!("🎬 Starting up minimodal server");
    println!("👂 Listening on {}", addr);

    Server::builder()
        .add_service(MiniModalServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

// Add this test function at the end of the file
#[no_mangle]
pub extern "C" fn test_function(input: String) -> String {
    format!("Hello, {}! This is a test function.", input)
}