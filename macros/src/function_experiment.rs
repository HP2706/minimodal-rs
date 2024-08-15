use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, GenericParam, TypeParam};

pub fn function_experiment_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_generics = &input.sig.generics;
    
    let generic_params = &fn_generics.params;
    let where_clause = &fn_generics.where_clause;

    // Extract input and output types
    let input_type = match fn_inputs.first() {
        Some(arg) => match arg {
            syn::FnArg::Typed(pat_type) => &pat_type.ty,
            _ => panic!("Unsupported argument type"),
        },
        None => panic!("Function must have at least one argument"),
    };

    let output_type = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        syn::ReturnType::Default => panic!("Function must have a return type"),
    };

    let expanded = quote! {
        static #fn_name: once_cell::sync::Lazy<
            for<#generic_params> Function<#input_type, #output_type>
        > = once_cell::sync::Lazy::new(|| {
            Function::new(
                stringify!(#fn_name).to_string(),
                |input: #input_type| -> #output_type #fn_body
            )
        });
    };

    expanded.into()
}