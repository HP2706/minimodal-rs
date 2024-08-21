#[path = "test_utils.rs"]
mod test_utils;

use macros::function;
use std::process::{Command, Child};
use std::time::Duration;
use tokio;
use tokio::time::sleep;
use basemodules::MiniModalError;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::future::Future;
use basemodules::function::Function;
use std::fmt::Debug;
use rstest::*;

async fn process_call<F>(call: F) -> bool
where
    F: Future<Output = Result<Vec<i32>, MiniModalError>>,
{
    match call.await {
        Ok(r) => true,
        Err(e) => false,
    }
}

#[fixture]
async fn server() -> Child {
    let server = test_utils::start_server(None).expect("Failed to start server");
    sleep(Duration::from_secs(2)).await;
    server
}

#[function]
async fn lala<T>(a: T) -> Result<Vec<i32>, MiniModalError> 
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static,
{
    Ok(vec![1, 2, 3])
}

#[rstest]
#[case::remote(lala::remote)]
#[case::local(lala::local)]
#[tokio::test]
async fn test_lala_function<F>(#[future] server: Child, #[case] func: F)
where
    F: Fn(i32) -> Pin<Box<dyn Future<Output = Result<Vec<i32>, MiniModalError>> + Send>>,
{
    let result = process_call(func(1)).await;
    assert!(result);
    server.await.kill().expect("Failed to kill server process");
}