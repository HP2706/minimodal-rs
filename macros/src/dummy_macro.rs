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
use crate::utils::parse_result_type;

pub fn dummy_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

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



    quote! {
        #(#attrs)*
        #vis #sig {
            let success = "1".to_string();
            println!("success: {:?}", success);
            println!("left_type: {:?}", stringify!(#left_type));
            let result: #left_type = serde_json::from_str(&success)?;
            println!("result: {:?}", result);
            return Ok(result);
        }   
    }.into()
}