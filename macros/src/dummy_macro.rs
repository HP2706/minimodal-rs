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
use syn::__private::ToTokens;
use crate::utils::extract_left_type;
use tonic::Request;

pub fn dummy_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    // we get the return type
    let return_type = match &sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    // Extract the Ok type if it's a Result
    let ok_type = extract_left_type(return_type.to_string());

    let request = Request::new(RunFunctionRequest {
        function_id: stringify!(#fn_name).to_string(),
        serialized_inputs: "".to_string(),
        output_type: stringify!(#return_type).to_string()
    });

    quote! {
        #(#attrs)*
        #vis #sig {
            let success = "[1, 2, 3]".to_string();
            let output_type_str : String = stringify!(#return_type).to_string();
            println!("ok_type: {:?}", stringify!(#ok_type));
            let result: #ok_type = serde_json::from_str(&success).expect("Failed to parse JSON");
            Ok(result)
        }
    }.into()
}