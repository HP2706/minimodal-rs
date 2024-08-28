use syn::{parse::Parse, parse::ParseStream, Expr, Ident, Token};
use darling::FromMeta;
use proc_macro::TokenStream;

#[derive(FromMeta)]
pub struct MacroArgs {
    #[darling(default)]
    pub debug: bool,
}

impl MacroArgs {
    pub fn parse(input: TokenStream) -> syn::Result<Self> {
        let args = darling::ast::NestedMeta::parse_meta_list(input.into())
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;
        
        Self::from_list(&args)
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))
    }
}