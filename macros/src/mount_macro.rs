use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn mount_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    quote! {
        #(#attrs)*
        #vis #sig {
            use minimodal_proto::proto::minimodal::FileEntry;
            use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
            use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
            use tonic::Request;
            use serde_json;
            use syn::parse_file;
            use serde::{Serialize, Deserialize};
            use minimodal_rs::utils::serialize_inputs;
            use minimodal_rs::mount::{mount_project, handle_main_rs};
            use std::fs;

            let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
            let req = mount_project(vec![".git".to_string()])?;
            let response = client.mount_project(req).await;
            println!("Original code: {}", stringify!(#block));
            
            // Call the original function
            let result = async move { #block };
            result.await
        }
    }.into()
}