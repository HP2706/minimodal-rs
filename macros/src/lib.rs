use proc_macro::TokenStream;
mod remote_function;
mod dummy_macro;
mod utils;

#[proc_macro_attribute]
pub fn remote_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    remote_function::remote_function_impl(_args, input)
}

#[proc_macro_attribute]
pub fn dummy_macro(_args: TokenStream, input: TokenStream) -> TokenStream {
    dummy_macro::dummy_impl(_args, input)
}