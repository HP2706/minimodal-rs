use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream};
use base64::{Engine as _, engine::general_purpose};
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
    let str_fn = input.to_string();
    let encoded_content = general_purpose::STANDARD.encode(str_fn);
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let fn_name = sig.ident.clone().to_string();
    quote! {
        #(#attrs)*
        #vis #sig {
            use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
            use tonic::Request;
            // we define get_dependencies in minimodal_rs
            use minimodal_rs::utils::get_dependencies; 
            let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
            
            // 1. Send the current file to the remote machine
            let request = RustFileRequest {
                rust_file: #encoded_content.into(),
                dependencies: get_dependencies(),
            };
            let response = client.send_rust_file(request).await?;
            println!("SendRustFile Response: {:?}", response);

            // 2. send request to run a function
            let request = Request::new(RunFunctionRequest {
                function_id: #fn_name.to_string(),
                inputs: "".to_string(),
            });
            let response = client.run_function(request).await?;
            println!("RunFunction Response: {}", response.get_ref().result);
            Ok(())
        }
    }.into()
}
