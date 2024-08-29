use proc_macro::{TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, format_ident};
use syn::Type;
use crate::macro_builder::MacroBuilder;


/// generates the impl for the map function if 
/// the input type is iterable else returns empty impl
fn generate_map_async_impl(
    new_inp_type: &Type,
) -> TokenStream2 {

    quote! {
        fn map_async(inputs: Vec<#new_inp_type>) -> Vec<Self::RemoteOutput> {
            inputs.into_iter().map(
                |x| {
                    Self::remote(x)
                }
            ).collect()
        }
    }
}

fn generate_map_impl(
    new_inp_type: &Type,
    output_type: &Type,
) -> TokenStream2 {
    quote! {
        fn map(inputs: Vec<#new_inp_type>) -> Pin<Box<dyn Future<Output = Vec<#output_type>> + Send>> {
            let futures = inputs.into_iter().map(|x| Self::remote(x));
            Box::pin(futures::future::join_all(futures))
        }
    }
}

pub fn impl_map_trait(
    macro_builder: &MacroBuilder,
) -> TokenStream2 {

    let MacroBuilder {
        fn_name,
        generics,
        where_clause,
        new_inp_type,
        output_type,
        ..
    } = macro_builder;

    let map_impl = generate_map_impl(new_inp_type, output_type);
    let map_async_impl = generate_map_async_impl(new_inp_type);
    quote!{
        impl #generics BatchFunction<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #map_impl
            #map_async_impl
        }
    }.into()

}