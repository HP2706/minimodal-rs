#!/bin/bash
# test_minimodal.bash

set -e  # Exit immediately if a command exits with a non-zero status.

# Create the test_function.rs file
cat << EOF > test_function.rs
#[no_mangle]
pub extern "C" fn test_function(input: String) -> String {
    format!("Hello, {}! This is a test function.", input)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <function_name> <input>", args[0]);
        std::process::exit(1);
    }
    
    let function_name = &args[1];
    let input = &args[2];
    
    if function_name == "test_function" {
        println!("{}", test_function(input.to_string()));
    } else {
        eprintln!("Unknown function: {}", function_name);
        std::process::exit(1);
    }
}
EOF

echo "Contents of test_function.rs:"
cat test_function.rs

echo "Encoding Rust file..."
RUST_FILE_CONTENT=$(base64 < test_function.rs | tr -d '\n')

echo "Sending Rust file..."
if ! SEND_RESULT=$(grpcurl -v -plaintext -import-path ./proto -proto minimodal.proto -d @ '[::1]:50051' minimodal.MiniModal/SendRustFile <<EOF
{
  "rust_file": "$RUST_FILE_CONTENT"
}
EOF
); then
    echo "Error sending Rust file: $SEND_RESULT"
    exit 1
fi

echo "Send result: $SEND_RESULT"

echo -e "\nRunning function..."
if ! RUN_RESULT=$(grpcurl -v -plaintext -import-path ./proto -proto minimodal.proto -d '{"function_id": "test_function", "inputs": "World"}' '[::1]:50051' minimodal.MiniModal/RunFunction); then
    echo "Error running function: $RUN_RESULT"
    exit 1
fi

echo "Run result: $RUN_RESULT"

# Clean up
rm test_function.rs