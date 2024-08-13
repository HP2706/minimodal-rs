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


pub fn dummy_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

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

    quote! {
        #(#attrs)*
        #vis #sig {
            use serde_json;
            
            let args = serde_json::json!({
                #(stringify!(#arg_names): #arg_names),*
            });
            let serialized_args = serde_json::to_string(&args)?;
            println!("serialized_args: {:?}", serialized_args);
            Ok(())
        }
    }.into()
}