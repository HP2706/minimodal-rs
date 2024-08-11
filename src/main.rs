use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RunFunctionRequest, 
};
use tonic::{IntoRequest, Request};
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use base64::{Engine as _, engine::general_purpose};



async fn test() -> Result<(), Box<dyn std::error::Error>> {
    println!("running test!");
    let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
    let file_content = r#"
fn hello() {
    println!("Hello from remote!");
}
    "#;
    let encoded_content = general_purpose::STANDARD.encode(file_content);
    println!("Encoded content: {}", encoded_content);
    let rust_file = encoded_content.into();
    println!("Rust file: {:?}", rust_file);
    let request = Request::new(RustFileRequest { rust_file: rust_file });
    let response = client.send_rust_file(request).await?;
    println!("SendRustFile Response: {:?}", response);

    // Run a function
    let request = Request::new(RunFunctionRequest {
        function_id: "hello".to_string(),
        inputs: "".to_string(),
    });
    let response = client.run_function(request).await?;
    println!("RunFunction Response: {:?}", response);


    Ok(())
}


#[tokio::main]
async fn main() {
    test().await;
}