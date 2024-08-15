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
        let args = Self::parse_meta_list(input)?;
        let opts = Self::from_nested_meta(&args)?;
        Ok(MacroArgs { debug: opts.debug })
    }

    fn parse_meta_list(input: TokenStream) -> syn::Result<Vec<darling::ast::NestedMeta>> {
        darling::ast::NestedMeta::parse_meta_list(input.into())
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))
    }

    fn from_nested_meta(args: &[darling::ast::NestedMeta]) -> syn::Result<Self> {
        Self::from_list(args)
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))
    }
}