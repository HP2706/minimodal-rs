use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, Type, FnArg, Ident};
//use crate::function::impl_remote_block;

pub fn function_experiment_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { mut sig, vis, block, attrs } = parse_macro_input!(item as ItemFn);

    let fn_name = sig.ident.clone();
    let where_clause = &sig.generics.where_clause;
    let generics = &sig.generics;

    let input_args_tuple : Vec<(Ident, Type)> = sig.inputs.iter()
        .filter_map(|input| {
            if let FnArg::Typed(pat_ty) = input {
                if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                    Some((pat_ident.ident.clone(), *pat_ty.ty.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let (input_idents, input_types): (Vec<Ident>, Vec<&Type>) = input_args_tuple
        .iter()
        .map(|(ident, ty)| (ident.clone(), ty))
        .unzip();
    
    let new_inp_type = quote! { (#(#input_types),*) };
    
    let output_type = match &sig.output {
        ReturnType::Type(_, ty) => quote! { #ty },
        ReturnType::Default => quote! { () },
    };

    let is_async = sig.asyncness.is_some();

    let local_impl = if is_async {
        quote! {
            type LocalOutput = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
            
            fn local(a: #new_inp_type) -> Self::LocalOutput {
                Box::pin(async move {
                    let (#(#input_idents),*) = a;
                    #block
                })
            }
        }
    } else {
        quote! {
            type LocalOutput = #output_type;
            
            fn local(a: #new_inp_type) -> Self::LocalOutput {
                let (#(#input_idents),*) = a;
                #block
            }
        }
    };

    let phantom_fields = generics.params.iter().map(|param| {
        match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                quote! { #[allow(dead_code)] #ident: std::marker::PhantomData<#ident>, }
            },
            _ => quote!(),
        }
    });

    let struct_def = quote! {
        #vis struct #fn_name #generics #where_clause {
            #(#phantom_fields)*
        }
    }; 

    let expanded: TokenStream = quote! {
        use std::pin::Pin;
        use std::future::Future;
        #struct_def

        impl #generics Function<#new_inp_type, #output_type> for #fn_name #generics
        #where_clause
        {
            #local_impl
            //#remote_impl
        }

    }.into();

    println!("Expanded: {}", expanded);

    TokenStream::from(expanded)
}