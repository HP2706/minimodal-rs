use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Type, ItemFn, parse::Parse, parse::ParseStream};
use base64::{Engine as _, engine::general_purpose};
use syn::__private::ToTokens;
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
    let encoded_content = general_purpose::STANDARD.encode(&input.to_string());
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let fn_name = sig.ident.clone().to_string();

    let arg_names: Vec<_> = sig.inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    let return_type = match &sig.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => quote!(#ty).to_string(),
    };

    let return_type_str = return_type.to_string();
    let left_type = if let Some(left_type) = parse_result_type(&return_type_str.clone()) {
        syn::parse_str::<syn::Type>(&left_type).expect("Failed to parse left_type")
    } else {
        panic!("Invalid return type: {}", return_type_str);
    };
    let left_type = left_type.to_token_stream().to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
            use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
            use tonic::Request;
            use serde_json;
            use serde::{Serialize, Deserialize};
            // we define get_dependencies in minimodal_rs
            use minimodal_rs::utils::{get_dependencies, serialize_inputs}; 

            let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
            println!("left type: {}", #left_type);


            // 1. Send the current file to the remote machine
            let request = RustFileRequest {
                rust_file: #encoded_content.into(),
                dependencies: get_dependencies(),
            };
            let response = client.send_rust_file(request).await?;

            let serialized_inputs = serialize_inputs(
                &[#(stringify!(#arg_names)),*], 
                &[#(&(#arg_names) as &dyn erased_serde::Serialize),*]
            )?;
            // 2. send request to run a function
            let request = Request::new(RunFunctionRequest {
                function_id: #fn_name.to_string(),
                serialized_inputs,
                output_type: #return_type_str.to_string()
            });

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



pub fn parse_result_type(s: &str) -> Option<String> {
    let s = s.trim();
    let s = s.replace(" ", "");
    if s.starts_with("Result<") && s.ends_with(",Error>") {
        let inner = &s[7..s.len() - 7];
        Some(inner.to_string())
    } else {
        None
    }
}