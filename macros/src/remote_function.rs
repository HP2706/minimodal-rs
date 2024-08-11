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
            use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
            use base64::{Engine as _, engine::general_purpose};
            use tonic::Request;
            let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
            // 1. Send the current file to the remote machine
            let file_content = "
            fn main() {
                println!(\"Hello from remote!\");
            }
            ";
            let encoded_content = general_purpose::STANDARD.encode(file_content);
            let rust_file_request = RustFileRequest {
                rust_file: encoded_content.into(),
            };
                    // Run a function
            let request = Request::new(RunFunctionRequest {
                function_id: "hello".to_string(),
                inputs: "".to_string(),
            });
            let response = client.run_function(request).await?;
            println!("RunFunction Response: {:?}", response);
            Ok(())
        }
    }.into()
}

