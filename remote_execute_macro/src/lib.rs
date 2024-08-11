use proc_macro::TokenStream;

mod log_duration;
mod remote_function;

#[proc_macro_attribute]
pub fn log_duration(_args: TokenStream, input: TokenStream) -> TokenStream {
    log_duration::log_duration_impl(_args, input)
}


