use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};
use crate::utils::extract_left_type;
use crate::args::{MacroArgs};
use proc_macro2::TokenStream as TokenStream2;

pub fn impl_remote_block(
    name: &syn::Ident,
    arg_names: Vec<syn::Ident>,
    output_type: &syn::Type
) -> TokenStream2 {

    quote! {
        use basemodules::MiniModalError;
        use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
        use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
        use minimodal_proto::proto::minimodal::RunFunctionRequest;
        use tonic::Request;
        use serde_json; 
        use serde::{Serialize, Deserialize};
        use minimodal_rs::utils::serialize_inputs;
        use minimodal_rs::mount::mount_project;

        let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
        let req = mount_project(vec![".git".to_string()])?;
        let _response = client.mount_project(req).await;

        let serialized_inputs = serialize_inputs(
            &[#(stringify!(#arg_names)),*], 
            &[#(&(#arg_names) as &dyn erased_serde::Serialize),*]
        )?;

        let request = Request::new(RunFunctionRequest {
            function_id: stringify!(#name).to_string(),
            serialized_inputs,
            output_type: stringify!(#output_type).to_string()
        });

        println!("request: {:?}", request);

        let response = client.run_function(request).await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?
            .get_ref().result.clone();


        match response {
            Some(RunFunctionResult::Success(success)) => {
                println!("recieved success: {}", success);
                let result: #output_type = match serde_json::from_str(&success) {
                    Ok(parsed) => Ok(parsed),
                    Err(e) => {
                        println!("Error parsing result: {:?} got {}", e, success);
                        return Err(MiniModalError::SerializationError(e.to_string()));
                    }
                };
                println!("Parsed result: {:?}", result);
                result
            }
            Some(RunFunctionResult::Error(error)) => {
                println!("Function failed with error: {}", error);
                Err(MiniModalError::FunctionError(error))?
            }
            None => {
                println!("No result received");
                Err(MiniModalError::OtherError("No result received".to_string()))?
            }
        }
    }.into()

}