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
async fn lala<'a, T>(a: T) -> Result<Vec<i32>, Error> 
where
    T: serde::de::Deserialize<'a> + serde::Serialize,
{
    Ok(vec![1, 2, 3])
}


#[tokio::main]
async fn main() -> () {
    match lala::<i32>(1).await {
        Ok(r) => println!("Result: {:?}", r),
        Err(e) => println!("Error: {:?}", e),
    }
}


/* 
#[dummy_macro]
fn bla() -> Result<i32, Error> {
    Ok(1)
}

fn main() {
    bla().unwrap();
}

 */