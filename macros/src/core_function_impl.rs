use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;
use syn::{parse_macro_input, ItemFn};
use crate::macro_builder::MacroBuilder;
use crate::stream_trait::impl_stream_trait;
use crate::map_trait::impl_map_trait;
use crate::function_trait::impl_function_trait;
/// the core logic in the "function" macro
/// 
/// it takes a function and its attributes.
pub fn function_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);   
    
    let vis = &item_fn.vis.clone();
    let is_async = item_fn.sig.asyncness.is_some();

    let macro_builder = MacroBuilder::new(item_fn);

    let cloned_generics = macro_builder.generics.clone();
    // phantom fields for generic types unused by the struct
    let phantom_fields = cloned_generics.params.iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                Some(quote! { #[allow(dead_code)] #ident: std::marker::PhantomData<#ident>, })
            },
            _ => None,
        });

    let stream_trait = impl_stream_trait(&macro_builder);
    let map_trait = impl_map_trait(&macro_builder);

    let function_trait = impl_function_trait(is_async, &macro_builder);

    let MacroBuilder {
        fn_name, 
        generics, 
        where_clause, 
        ..
    } = macro_builder;

    quote! {
        #vis struct #fn_name #generics #where_clause {
            #(#phantom_fields)*
        }

        #function_trait
        #stream_trait
        #map_trait

    }.into()
}
