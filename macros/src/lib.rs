use proc_macro::TokenStream;
mod remote_function;

#[proc_macro_attribute]
pub fn remote_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    remote_function::remote_function_impl(_args, input)
}

