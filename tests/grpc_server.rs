#[path = "test_utils.rs"]
mod test_utils;
use std::time::Duration;
use tokio;
use tokio::time::sleep;
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use minimodal_rs::mount::mount_project;

#[tokio::test]
async fn test_grpc_server() {
    let server_name = "test_mount_dir".to_string();

    let mut server = match test_utils::start_server(Some(&server_name)) {
        Ok(child) => child,
        Err(e) => panic!("Failed to start server: {}", e),
    };
    sleep(Duration::from_secs(2)).await;

    let mut client = MiniModalClient::connect("http://[::1]:50051").await.unwrap();
    let req = mount_project(&mut client, vec![".git".to_string(), "minimodal_proto".to_string(), "macros".to_string(), "src/server".to_string()]).await.unwrap();
    server.kill().expect("Failed to kill server");
}