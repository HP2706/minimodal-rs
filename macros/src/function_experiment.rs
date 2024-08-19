use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, Type};

pub fn function_experiment_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;
    let mut struct_name = format_ident!("_{}", func_name.to_string());
    let vis = &input.vis;
    let block = &input.block;
    let inputs = &input.sig.inputs;
    let where_clause = &input.sig.generics.where_clause;
    let generics = &input.sig.generics;


    let (output_type, is_async, return_type) = match &input.sig.output {
        ReturnType::Default => (quote!(()), false, quote!(())),
        ReturnType::Type(_, ty) => {
            match &**ty {
                Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Future" => {
                    let inner_type = &type_path.path.segments.last().unwrap().arguments;
                    (quote!(#ty), true, quote!(#inner_type))
                },
                _ => (quote!(std::future::Ready<#ty>), false, quote!(#ty)),
            }
        }
    };

    let local_impl = if is_async {
        quote! {
            type Output = #output_type;

            fn local(#inputs) -> Self::Output {
                Box::pin(async move #block)
            }
        }
    } else {
        quote! {
            type Output = std::future::Ready<#return_type>;

            fn local(#inputs) -> Self::Output {
                std::future::ready(#block)
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
        #vis struct #struct_name #generics #where_clause {
            #(#phantom_fields)*
        }
    }; 

    // we collect A, B, .. in <A, B, ..>
    
    let mut generic_type_params = generics.params.iter().filter_map(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            Some(type_param.ident.clone())
        } else {
            None
        }
    }).collect::<Vec<syn::Ident>>();

    let (generic_type_params, additional_where_clause) = if generic_type_params.len() == 1 {
        let new_type = format_ident!("__B");
        let existing_type = &generic_type_params[0];
        (
            vec![existing_type.clone(), new_type.clone()],
            quote! { #new_type: std::convert::From<#existing_type> }
        )
    } else {
        (generic_type_params, quote!())
    };

    let where_clause = if let Some(existing_where) = where_clause {
        quote! { #existing_where #additional_where_clause }
    } else if !additional_where_clause.is_empty() {
        quote! { where #additional_where_clause }
    } else {
        quote!()
    };

    let expanded: TokenStream = quote! {
        #struct_def

        impl<#(#generic_type_params),*> Function<#(#generic_type_params),*> for #struct_name #generics
        #where_clause
        {
            #local_impl
        }

        #input
    }.into();

    println!("expanded: {}", expanded);

    TokenStream::from(expanded)
}

