use syn::{
    Generics, 
    WhereClause, 
    Type, 
    Ident, 
    Block, 
    FnArg, 
    ReturnType, 
    ItemFn,
    parse_quote
};

use proc_macro2::TokenStream;
pub struct MacroBuilder {
    pub fn_name: Ident,
    pub generics: Generics,
    pub where_clause: Option<WhereClause>,
    pub new_inp_type: Box<Type>,
    pub output_type: Box<Type>,
    pub input_idents: Vec<Ident>,
    pub block: Block,
    pub types_and_names: Vec<TokenStream>,
}

impl MacroBuilder {
    pub fn new(item: ItemFn) -> Self {
        let ItemFn { sig, block, .. } = item;

        let input_args: Vec<(Ident, Type)> = sig.inputs.iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(pat_ty) => {
                    if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                        Some((pat_ident.ident.clone(), (*pat_ty.ty).clone()))
                    } else {
                        None
                    }
                }
                _ => None
            })
            .collect();

        let (input_idents, input_types): (Vec<_>, Vec<_>) = input_args.iter().cloned().unzip();

        let types_and_names : Vec<TokenStream> = input_args
            .iter()
            .map(
                |(name, ty)| 
                quote::quote! { NameAndType { name: stringify!(#name).to_string(), ty: stringify!(#ty).to_string() } }
            )
            .collect::<Vec<_>>();

        let new_inp_type = parse_quote!((#(#input_types),*)); 
        // Create NameAndType structs for each input
        let output_type = match sig.output {
            ReturnType::Type(_, ty) => ty,
            ReturnType::Default => panic!("Output type must be of type Result<_, MiniModalError>"),
        };

        Self {
            fn_name: sig.ident,
            where_clause: sig.generics.where_clause.clone(),
            generics: sig.generics,
            new_inp_type,
            output_type,
            input_idents,
            block: *block,
            types_and_names: types_and_names,
        }

    }


}