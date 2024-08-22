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
use polars::prelude::*;

async fn process_call<F, O>(call: F) -> bool
where
    F: Future<Output = Result<O, MiniModalError>>,
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

#[function]
async fn multi_arg(a: i32, b: i32) -> Result<Vec<i32>, MiniModalError> {
    Ok(vec![a, b])
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarsDataFrame(DataFrame);

#[function]
async fn df_test_deserialize(df: PolarsDataFrame) -> Result<PolarsDataFrame, MiniModalError> {
    println!("🔥 Result: {:?}", df);
    Ok(df)
}

#[rstest]
#[case::local((lala::<i32>::local, 1))]
#[case::remote((lala::<i32>::remote, 1))]
#[case::remote((df_test_deserialize::remote, PolarsDataFrame(DataFrame::new(vec![Series::new("col1", vec![1, 2, 3])]).unwrap())))]
#[case::local((df_test_deserialize::local, PolarsDataFrame(DataFrame::new(vec![Series::new("col1", vec![1, 2, 3])]).unwrap())))]
#[case::remote((multi_arg::remote, (1, 2)))]
#[case::local((multi_arg::local, (1, 2)))]
#[tokio::test]
async fn test_function<I, O, F>(#[future] server: Child, #[case] func_input: (F, I))
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static,
    F: Fn(I) -> Pin<Box<dyn Future<Output = Result<O, MiniModalError>> + Send + 'static>> + Send + 'static
{
    let result = process_call((func_input.0)(func_input.1)).await;
    assert!(result);
    server.await.kill().expect("Failed to kill server process");
}
