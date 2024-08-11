use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream};

use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RustFileResponse, 
    RunFunctionRequest, 
    RunFunctionResponse
};


struct MacroInput {
    debug_arg: syn::Expr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroInput {
            debug_arg: input.parse()?,
        })
    }
}


pub fn remote_function_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let statements = block.stmts;
    let function_identifier = sig.ident.clone();
    
    quote! {
        #(#attrs)*
        #vis #sig {
            use base64::{Engine as _, engine::general_purpose};
            use tonic::Request;

            println!("running on {:?}", n_cpus);

            // 1. Send the current file to the remote machine
            let file_content = "
            fn main() {
                println!(\"Hello from remote!\");
            }
            ";
            let encoded_content = general_purpose::STANDARD.encode(file_content);
            let rust_file_request = RustFileRequest {
                rust_file: encoded_content,
            };
            let _response: RustFileResponse = tonic::client::Grpc::new(channel)
                .rust_file(Request::new(rust_file_request))
                .await?;

            /* // 2. Run the function on the remote machine
            let run_request = RunFunctionRequest {
                function_id: stringify!(#function_identifier).to_string(),
                inputs: format!("{:?}", (#(#statements),*)),
            };
            let response: RunFunctionResponse = tonic::client::Grpc::new(channel)
                .run_function(Request::new(run_request))
                .await?;

            // 3. Return the result
            let result = serde_json::from_str(&response.result).unwrap();
            println!("Result: {:?}", result); */
            Ok(())
        }
    }.into()
}