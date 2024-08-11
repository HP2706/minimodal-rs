use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream};

struct MacroInput {
    debug_arg: syn::Expr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroInput {
            debug_arg: input.parse()?,
        })
    }
}

pub fn log_duration_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroInput);
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let statements = block.stmts;
    let function_identifier = sig.ident.clone();
    let debug_arg = args.debug_arg;

    quote!(
        #(#attrs)*
        #vis #sig {
            let __start = std::time::Instant::now();
            
            let __result = {
                #(#statements)*
            };

            println!("{} took {}μs, Debug: {:?}", 
                stringify!(#function_identifier), 
                __start.elapsed().as_micros(),
                #debug_arg
            );
            
            __result
        }
    ).into()
}