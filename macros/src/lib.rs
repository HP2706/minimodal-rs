use proc_macro::TokenStream;
mod function_trait;
mod utils;
mod args;
mod stream_trait;
mod map_trait;
mod core_function_impl;
mod macro_builder;

#[proc_macro_attribute]
pub fn function(_args: TokenStream, input: TokenStream) -> TokenStream {
    core_function_impl::function_impl( _args, input)
}
