use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RunFunctionRequest, 
};
use tonic::{IntoRequest, Request};
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use base64::{Engine as _, engine::general_purpose};
use macros::{remote_function, dummy_macro};
use anyhow::Error;
use minimodal_rs::utils::{extract_left_type, mount_project};
use quote::quote;

#[remote_function]
async fn lala<'a, T>(a: T) -> Result<Vec<i32>, Error> 
where
    T: serde::de::Deserialize<'a> + serde::Serialize,
{
    Ok(vec![1, 2, 3])
}

#[tokio::main]
async fn main() -> () {
    /* match lala::<i32>(1).await {
        Ok(r) => println!("Result: {:?}", r),
        Err(e) => println!("Error: {:?}", e),
    } */
    mount_project(vec![".git".to_string()]).unwrap();
}