use proc_macro::TokenStream;
mod remote_function;
mod mount_macro;
mod utils;

#[proc_macro_attribute]
pub fn remote_function(_args: TokenStream, input: TokenStream) -> TokenStream {
    remote_function::remote_function_impl("remote_function".to_string(), _args, input)
}

//for debug
#[proc_macro_attribute]
pub fn mount(_args: TokenStream, input: TokenStream) -> TokenStream {
    mount_macro::mount_impl("mount".to_string(), _args, input)
}