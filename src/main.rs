use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RunFunctionRequest, 
};
use tonic::{IntoRequest, Request};
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use base64::{Engine as _, engine::general_purpose};
use macros::remote_function;

#[remote_function]
async fn lala() -> Result<(), Box<dyn std::error::Error>> {
    println!("lala");
    Ok(())
}

#[tokio::main]
async fn main() {
    
    lala().await.unwrap();
}