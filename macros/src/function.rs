use proc_macro::{TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, Type, FnArg, Ident,};

/// the core logic in the "function" macro
/// 
/// it takes a function and its attributes.
pub fn function_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { mut sig, vis, block, attrs } = parse_macro_input!(item as ItemFn);

    let fn_name = &sig.ident;
    let generics = &sig.generics;
    let where_clause = &generics.where_clause;

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
    
    let new_inp_type = quote! { (#(#input_types),*) }; 
    // Create NameAndType structs for each input
    let input_types_str = input_args.iter()
        .map(
            |(name, ty)| 
            quote! { NameAndType { name: stringify!(#name).to_string(), ty: stringify!(#ty).to_string() } }
        )
        .collect::<Vec<_>>();
    
    let output_type = match &sig.output {
        ReturnType::Type(_, ty) => ty,
        ReturnType::Default => panic!("Output type must be of type Result<_, MiniModalError>"),
    };

    // phantom fields for generic types unused by the struct
    let phantom_fields = generics.params.iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                Some(quote! { #[allow(dead_code)] #ident: std::marker::PhantomData<#ident>, })
            },
            _ => None,
        });

    let remote_impl = generate_remote_impl(
        &fn_name,
        &input_idents,
        input_types_str,
        output_type,
        &new_inp_type,
    );

    let local_impl = generate_local_impl(
        sig.asyncness.is_some(),
        output_type,
        &new_inp_type,
        &input_idents,
        &block
    );

    let map_impl = generate_map_impl(&new_inp_type, &output_type);
    let map_async_impl = generate_map_async_impl(&new_inp_type);

    let stream_impl = generate_stream_impl(&new_inp_type);
    quote! {
        #vis struct #fn_name #generics #where_clause {
            #(#phantom_fields)*
        }

        impl #generics Function<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #local_impl
            #remote_impl
        }

        impl #generics BatchFunction<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #map_impl
            #map_async_impl
        }

        impl #generics StreamingFunction<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #stream_impl
        }

    }.into()
}

/// generates the impl for the map function if 
/// the input type is iterable else returns empty impl
fn generate_map_async_impl(
    new_inp_type: &TokenStream2,
) -> TokenStream2 {

    quote! {
        fn map_async(inputs: Vec<#new_inp_type>) -> Vec<Self::RemoteOutput> {
            inputs.into_iter().map(
                |x| {
                    Self::remote(x)
                }
            ).collect()
        }
    }
}

fn generate_map_impl(
    new_inp_type: &TokenStream2,
    output_type: &Type,
) -> TokenStream2 {
    quote! {
        fn map(inputs: Vec<#new_inp_type>) -> Pin<Box<dyn Future<Output = Vec<#output_type>> + Send>> {
            let futures = inputs.into_iter().map(|x| Self::remote(x));
            Box::pin(futures::future::join_all(futures))
        }
    }
}

fn generate_stream_impl(
    new_inp_type: &TokenStream2,
) -> TokenStream2 {
    quote! {
        type InputStream = Pin<Box<dyn Stream<Item = #new_inp_type> + Send>>;
        type OutputStream = Pin<Box<dyn Stream<Item = Self::RemoteOutput> + Send>>;
        fn map_stream(input: Self::InputStream) -> Self::OutputStream {
            Box::pin(
                input.map(|x| Self::remote(x))
            )
        }
    }
}

fn generate_local_impl(
    is_async: bool,
    output_type: &Type,
    new_inp_type: &TokenStream2,
    input_idents: &Vec<Ident>,
    block: &syn::Block,
) -> TokenStream2 {
    let new_input_ident = generate_new_input_ident(&input_idents);
    
    // Add this block to check the return type
    let block_return_type = infer_block_return_type(block);
    /* if !types_match(output_type, &block_return_type) {
        return syn::Error::new(
            Span::call_site(),
            format!("Function body returns {:?}, but the declared return type is {:?}", 
                    block_return_type, output_type)
        ).to_compile_error();
    } */

    if is_async {
        quote! {
            type LocalOutput = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
            fn local(#new_input_ident: #new_inp_type) -> Self::LocalOutput {
                Box::pin(async move { let (#(#input_idents),*) = #new_input_ident; #block })
            }
        }
    } else {
        quote! {
            type LocalOutput = #output_type;
            fn local(#new_input_ident: #new_inp_type) -> Self::LocalOutput {
                let (#(#input_idents),*) = #new_input_ident; #block
            }
        }
    }
}


fn generate_new_input_ident(input_idents: &Vec<Ident>) -> Ident {
    format_ident!(
        "{}", 
        input_idents.iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<String>>()
            .join("_")
    )
}

fn generate_remote_impl(
    name: &syn::Ident,
    args: &Vec<syn::Ident>,
    field_types: Vec<TokenStream2>,
    output_type: &syn::Type,
    new_inp_type: &TokenStream2,
) -> TokenStream2 {
    let new_input_ident = generate_new_input_ident(&args);
    let remote_block = generate_remote_block(name, args, field_types, output_type);

    quote! {
        type RemoteOutput = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
        fn remote(#new_input_ident: #new_inp_type) -> Self::RemoteOutput {
            Box::pin(async move { 
                let (#(#args),*) = #new_input_ident; 
                #remote_block
            })
        }
    }
}

fn generate_remote_block(
    name: &syn::Ident,
    args: &Vec<syn::Ident>,
    field_types: Vec<TokenStream2>,
    output_type: &syn::Type,
) -> TokenStream2 {
    quote! {
        use basemodules::MiniModalError;
        use minimodal_proto::proto::minimodal::{mini_modal_client::MiniModalClient, run_function_response::Result as RunFunctionResult, RunFunctionRequest};
        use tonic::Request;
        use serde_json;
        use minimodal_rs::utilities::serialize_inputs;
        use minimodal_rs::mount::mount_project;
        use minimodal_proto::proto::minimodal::NameAndType;

        let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
        
        mount_project(&mut client, vec![".git".to_string()])
            .await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?;

        let serialized_inputs = serialize_inputs(
            &[#(stringify!(#args)),*], 
            &[#(&(#args) as &dyn erased_serde::Serialize),*]
        )?;

        let request = Request::new(RunFunctionRequest {
            function_id: stringify!(#name).to_string(),
            serialized_inputs,
            field_types: vec![#(#field_types),*],
            output_type: stringify!(#output_type).to_string()
        });

        let response = client.run_function(request).await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?
            .into_inner()
            .result
            .ok_or_else(|| MiniModalError::OtherError("No result received".to_string()))?;

        match response {
            RunFunctionResult::Success(success) => {
                serde_json::from_str(&success)
                    .map_err(|e| MiniModalError::SerializationError(e.to_string()))
            }
            RunFunctionResult::Error(error) => Err(MiniModalError::FunctionError(error)),
        }
    }
}

// Add these helper functions
fn infer_block_return_type(block: &syn::Block) -> Type {
    // This is a simplified version. You might need a more sophisticated
    // analysis for complex blocks.
    if let Some(syn::Stmt::Expr(expr, _)) = block.stmts.last() {
        expr_to_type(expr)
    } else {
        syn::parse_quote!(())
    }
}

fn expr_to_type(expr: &syn::Expr) -> Type {
    match expr {
        syn::Expr::Call(call) => {
            // Assume the return type of the call is the type of the first argument
            if let syn::Expr::Path(path) = &*call.func {
                if let Some(segment) = path.path.segments.last() {
                    if segment.ident == "Ok" || segment.ident == "Err" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                                return t.clone();
                            }
                        }
                    }
                }
            }
        }
        // Add more cases as needed
        _ => {}
    }
    syn::parse_quote!(())
}

fn types_match(declared: &Type, actual: &Type) -> bool {
    // This is a simplified comparison. You might need to handle more cases.
    format!("{:?}", declared) == format!("{:?}", actual)
}