use proc_macro::{TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, format_ident};
use crate::function::impl_remote_block;
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, Type, FnArg, Ident,};
use crate::utils::extract_left_type;


pub fn function_experiment_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { mut sig, vis, block, attrs } = parse_macro_input!(item as ItemFn);

    let fn_name = &sig.ident;
    let generics = &sig.generics;
    let where_clause = &generics.where_clause;

    let input_args: Vec<(Ident, Type)> = sig.inputs.iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_ty) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                    Some((pat_ident.ident.clone(), (*pat_ty.ty).clone()))
                } else {
                    None
                }
            }
            _ => None
        })
        .collect();

    let (input_idents, input_types): (Vec<_>, Vec<_>) = input_args.iter().cloned().unzip();
    let new_inp_type = quote! { (#(#input_types),*) };
    
    let output_type = match &sig.output {
        ReturnType::Type(_, ty) => {
            check_output_type(ty);
            ty
        },
        ReturnType::Default => panic!("Output type must be of type Result<_, MiniModalError>"),
    };


    let remote_block = impl_remote_block(
        fn_name,
        input_idents.clone(),
        output_type
    );

    let is_async = sig.asyncness.is_some();

    //join the names of all input args
    let new_input_ident = format_ident!("{}", input_idents.iter().map(|ident| ident.to_string()).collect::<Vec<String>>().join("_"));

    let local_impl = if is_async {
        quote! {
            type Output = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
            fn local(#new_input_ident: #new_inp_type) -> Self::Output {
                Box::pin(async move { let (#(#input_idents),*) = #new_input_ident; #block })
            }
        }
    } else {
        quote! {
            type Output = #output_type;
            fn local(#new_input_ident: #new_inp_type) -> Self::Output {
                let (#(#input_idents),*) = #new_input_ident; #block
            }
        }
    };

    // New code to handle Result types
    let remote_impl = quote! {
        fn remote(#new_input_ident: #new_inp_type) -> Self::Output {
            Box::pin(async move { 
                let (#(#input_idents),*) = #new_input_ident; 
                #remote_block
            })
        }
    };

    //println!("remote_impl: {}", remote_impl);

    /// phantom fields for generic types unused by the struct
    let phantom_fields = generics.params.iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                Some(quote! { #[allow(dead_code)] #ident: std::marker::PhantomData<#ident>, })
            },
            _ => None,
        });

    let expanded = quote! {
        use std::pin::Pin;
        use std::future::Future;

        #vis struct #fn_name #generics #where_clause {
            #(#phantom_fields)*
        }

        impl #generics Function<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #local_impl
            #remote_impl
        }
    };

    println!("expanded: {}", expanded);

    TokenStream::from(expanded)
}


fn check_output_type(ty: &Type) {

    // Check if the output type is Result<_, MiniModalError>
    let is_valid_result_type = if let Type::Path(type_path) = &*ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            last_segment.ident == "Result" &&
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                args.args.len() == 2 &&
                if let syn::GenericArgument::Type(Type::Path(error_type)) = &args.args[1] {
                    error_type.path.is_ident("MiniModalError")
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if !is_valid_result_type {
        panic!("Output type must be Result<_, MiniModalError>");
    }

    ()
}
