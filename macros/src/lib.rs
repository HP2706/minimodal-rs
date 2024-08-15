use proc_macro::TokenStream;
mod function;
mod function_experiment;
mod mount_macro;
mod utils;
mod args;

#[proc_macro_attribute]
pub fn function(_args: TokenStream, input: TokenStream) -> TokenStream {
    function::function_impl( _args, input)
}

#[proc_macro_attribute]
pub fn function_experiment(_args: TokenStream, input: TokenStream) -> TokenStream {
    function_experiment::function_experiment_impl(_args, input)
}

//for debug
#[proc_macro_attribute]
pub fn mount(_args: TokenStream, input: TokenStream) -> TokenStream {
    mount_macro::mount_impl( _args, input)
}

/* ,
remote: Box::new(|args| Box::pin(async move {
let (#(#arg_names),*) = args;
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
let _ = client.mount_project(req).await;

let serialized_inputs = serialize_inputs(
    &[#(stringify!(#arg_names)),*],
    &[#(&(#arg_names) as &dyn erased_serde::Serialize),*]
)?;

let request = Request::new(RunFunctionRequest {
    function_id: #fn_name_str.to_string(),
    serialized_inputs,
    output_type: #return_type.to_string()
});

let response = client.run_function(request).await?.get_ref().result.clone();

match response {
    Some(RunFunctionResult::Success(success)) => {
        let result: #left_type = serde_json::from_str(&success)?;
        Ok(result)
    }
    Some(RunFunctionResult::Error(error)) => {
        Err(anyhow::anyhow!(error))
    }
    None => {
        Err(anyhow::anyhow!("No result received"))
    }
} 
}))*/