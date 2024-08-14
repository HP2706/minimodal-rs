use macros::remote_function;
use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RunFunctionRequest, 
};
use anyhow::Error;
use std::process::{Command, Child};
use std::time::Duration;
use tokio;
use tokio::time::sleep;

fn start_server() -> Child {
    Command::new("cargo")
        .args(["run", "--bin", "minimodal-server"])
        .spawn()
        .expect("Failed to start server")
}

#[tokio::test]
async fn test_remote_basic_function() {
    // Start the server
    let mut server = start_server();
    
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


