#[path = "test_utils.rs"]
mod test_utils;

use macros::remote_function;
use minimodal_proto::proto::minimodal::{
    RunFunctionRequest, 
};
use anyhow::Error;
use std::process::{Command, Child};
use std::time::Duration;
use tokio;
use tokio::time::sleep;




#[tokio::test]
async fn test_remote_basic_function() {
    // Start the server
    let mut server = test_utils::start_server(None);

    // Give the server some time to start up
    sleep(Duration::from_secs(2)).await;

    #[remote_function]
    async fn lala<'a, T>(a: T) -> Result<Vec<i32>, Error> 
    where
        T: serde::de::Deserialize<'a> + serde::Serialize,
    {
        Ok(vec![1, 2, 3])
    }

    match lala::<i32>(1).await {
        Ok(r) => println!("Result: {:?}", r),
        Err(e) => println!("Error: {:?}", e),
    }

    // Shutdown the server
    server.kill().expect("Failed to kill server process");
}