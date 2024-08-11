use remote_execute_macro::log_duration;

#[derive(Debug)]
struct Image {
    secret : String
}

#[log_duration(Image {secret : "secret".to_string()})]
fn hello(a : i32) -> () {
    println!("Hello, world!");
}
fn main() {
    hello(1);
}