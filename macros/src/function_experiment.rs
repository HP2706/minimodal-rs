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
    let mut where_clause = &input.sig.generics.where_clause;
    let generics = &input.sig.generics;

    let orig_output_type = match &input.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let (output_type, is_async, return_type) = match &input.sig.output {
        ReturnType::Default => (quote!(()), false, quote!(())),
        ReturnType::Type(_, ty) => {
            match &**ty {
                Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Future" => {
                    let inner_type = &type_path.path.segments.last().unwrap().arguments;
                    (quote!(std::pin::Pin<Box<dyn Future<Output = #inner_type> + Send + 'static>>), true, quote!(#inner_type))
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
    let generic_type_params = generics.params.iter().filter_map(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            Some(type_param.ident.clone())
        } else {
            None
        }
    }).collect::<Vec<syn::Ident>>();

    let expanded: TokenStream = quote! {
        #struct_def

        impl<#(#generic_type_params),*> Function<#(#generic_type_params),*, #orig_output_type> for #struct_name #generics
        #where_clause
        {
            #local_impl
        }

        #input
    }.into();

    println!("expanded: {}", expanded);

    TokenStream::from(expanded)
}

