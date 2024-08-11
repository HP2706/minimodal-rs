use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream};

use minimodal_proto::proto::minimodal::{
    RustFileRequest, 
    RustFileResponse, 
    RunFunctionRequest, 
    RunFunctionResponse
};

struct MacroInput {
    n_cpus: syn::Expr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroInput {
            n_cpus: input.parse()?,
        })
    }
}

pub fn remote_function_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroInput);
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { sig, vis, block, attrs } = input;

    let statements = block.stmts;
    let function_identifier = sig.ident.clone();
    let n_cpus = args.n_cpus;
    
    // the code should 
    // 1. send the current file to the remot machine via: 
    // {
    //   "rust_file": "$RUST_FILE_CONTENT"(base64 encoded)
    // }
    // 2. grpcurl -v -plaintext -import-path ./proto -proto minimodal.proto -d '{"function_id": "test_function", "inputs": "World"}' '[::1]:50051' minimodal.MiniModal/RunFunction
    // 3. return the result
    quote!(
        
    ).into()
}