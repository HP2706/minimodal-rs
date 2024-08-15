
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};
use crate::utils::extract_left_type;
use crate::args::{MacroArgs};

pub fn function_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let fn_name = sig.ident.clone().to_string();

        
    let arg_names: Vec<_> = sig.inputs.iter()
    .filter_map(|arg| match arg {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        }
        _ => None
    })
    .collect();

    let return_type = match &sig.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => quote!(#ty).to_string(),
    };
    let left_type = extract_left_type(return_type.to_string());

    quote! {
        #(#attrs)*
        #vis #sig {
            use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
            use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
            use minimodal_proto::proto::minimodal::RunFunctionRequest;
            use tonic::Request;
            use serde_json;
            use serde::{Serialize, Deserialize};
            // we define get_dependencies in minimodal_rs
            use minimodal_rs::utils::{serialize_inputs}; 
            use minimodal_rs::mount::mount_project;

            let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
            let req = mount_project(vec![".git".to_string()])?;
            let response = client.mount_project(req).await;

            let serialized_inputs = serialize_inputs(
                &[#(stringify!(#arg_names)),*], 
                &[#(&(#arg_names) as &dyn erased_serde::Serialize),*]
            )?;
            // 2. send request to run a function
            let request = Request::new(RunFunctionRequest {
                function_id: #fn_name.to_string(),
                serialized_inputs,
                output_type: #return_type.to_string()
            });

            println!("request: {:?}", request);

            let response = client.run_function(request).await?.get_ref().result.clone();

            match response {
                Some(RunFunctionResult::Success(success)) => {
                    println!("Function succeeded with result: {}", success);
                    let result: #left_type = serde_json::from_str(&success)?;
                    println!("Parsed result: {:?}", result);
                    Ok(result)
                }
                Some(RunFunctionResult::Error(error)) => {
                    println!("Function failed with error: {}", error);
                    Err(anyhow::anyhow!(error))
                }
                None => {
                    println!("No result received");
                    Err(anyhow::anyhow!("No result received"))
                }
            }
        }
    }.into()
}