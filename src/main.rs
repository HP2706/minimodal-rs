use remote_execute_macro::remote_execute;


#[remote_execute]
async fn my_remote_function() -> i32 {
    println!("This will be executed on the server!");
    42
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = my_remote_function().await?;
    println!("Result from server: {}", result);
    Ok(())
}