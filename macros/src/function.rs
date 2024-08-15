use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream, Expr, LitStr, Ident, Token};
use base64::{Engine as _, engine::general_purpose};
use crate::utils::extract_left_type;
use minimodal_proto::proto::minimodal::{MountProjectRequest, MountProjectResponse};
use quote::{quote, ToTokens};
use anyhow::anyhow;
use basemodules::function::Function;
use once_cell::sync::Lazy;
use std::sync::Arc;


struct MacroArgs {
    debug: Option<Expr>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut debug = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "debug" => debug = Some(input.parse()?),
                _ => return Err(input.error("Unexpected attribute")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MacroArgs { debug })
    }
}

pub fn function_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let fn_name = &sig.ident;
    let fn_name_str = fn_name.to_string();
    let impl_fn_name = Ident::new(&format!("{}_impl", fn_name), sig.ident.span());

    let generics = &sig.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let debug = args.debug;

    let arg_types: Vec<_> = sig.inputs.iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(&pat_type.ty),
            _ => None
        })
        .collect();

    let input_type = if arg_types.len() == 1 {
        quote!(#(#arg_types)*)
    } else {
        quote!((#(#arg_types),*))
    };

    println!("input_type: {}", input_type);
    println!("arg_types: {:?}", arg_types);
    let return_type = match &sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    let mut new_sig = sig.clone();
    println!("ident before: {}", sig.ident);
    new_sig.ident = impl_fn_name.clone();
    println!("ident after: {}", new_sig.ident);
    let base_fn = quote! {
        #(#attrs)*
        #vis #new_sig {
            #block
        }
    };

    println!("new_sig.ident: {}", new_sig.ident);
    println!("input_type: {}", input_type);
    println!("return_type: {}", return_type);

    let output = quote! {
        #base_fn

        #[allow(non_upper_case_globals)]
        static #fn_name: Lazy<Function<#input_type, #return_type>> = Lazy::new(|| {
            Function::new(
                #fn_name_str,
                Arc::new(|a #ty_generics| #impl_fn_name #ty_generics (a))
            )
        });
    };

    println!("entire macro: {}", output);
    output.into()
}