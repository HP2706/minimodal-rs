use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use crate::macro_builder::MacroBuilder;

pub fn impl_stream_trait(
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

    quote! {
        impl #generics StreamingFunction<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            type InputStream = Pin<Box<dyn Stream<Item = #new_inp_type> + Send>>;
            type OutputStream = Pin<Box<dyn Stream<Item = Self::RemoteOutput> + Send>>;
            fn map_stream(input: Self::InputStream) -> Self::OutputStream {
                Box::pin(
                    input.map(|x| Self::remote(x))
                )
            }
        }
    }

}
