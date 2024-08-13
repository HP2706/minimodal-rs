use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RunFunctionRequest, 
};
use tonic::{IntoRequest, Request};
use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
use base64::{Engine as _, engine::general_purpose};
use macros::{remote_function, dummy_macro};
use anyhow::Error;

#[remote_function]
async fn lala<'a, T>(a: T) -> Result<i32, Error> 
where
    T: serde::de::Deserialize<'a> + serde::Serialize,
{
    Ok(1)
}


#[tokio::main]
async fn main() -> () {
    match lala::<i32>(1).await {
        Ok(r) => println!("Result: {:?}", r),
        Err(e) => println!("Error: {:?}", e),
    }
}
/* 
use syn::parse_str;
use syn::__private::ToTokens;
fn main() {
    let left_type = "i32";
    let parsed_type = parse_str::<syn::Type>(&left_type).expect("Failed to parse left_type");
    
    let parsed_type = parsed_type.to_token_stream().to_string();
    println!("Parsed type: {:?}", parsed_type);
}





 */