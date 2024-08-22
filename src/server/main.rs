use tokio;
use tonic::transport::Server;
use std::env;
use std::process::Command;
use minimodal_proto::proto::minimodal::mini_modal_server::MiniModalServer;
use minimodal_rs::server::server::MiniModalService;

// Function to kill process using the port
fn kill_process_on_port(port: u16) -> Result<(), std::io::Error> {
    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output()?;

    if !output.stdout.is_empty() {
        let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Command::new("kill").arg("-9").arg(&pid).output()?;
        println!("Killed process {} using port {}", pid, port);
    }

    Ok(())
}

// run server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    // Kill process on port 50051 if active
    kill_process_on_port(50051)?;

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