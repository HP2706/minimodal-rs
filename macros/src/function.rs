use proc_macro::{TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, Type, FnArg, Ident,};

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
        ReturnType::Type(_, ty) => {
            check_output_type(ty);
            ty
        },
        ReturnType::Default => panic!("Output type must be of type Result<_, MiniModalError>"),
    };


    let remote_block = impl_remote_block(
        fn_name,
        input_idents.clone(),
        input_types_str,
        output_type
    );

    let is_async = sig.asyncness.is_some();

    //join the names of all input args
    let new_input_ident = format_ident!(
        "{}", 
        input_idents.iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>().join("_")
    );

    let local_impl = if is_async {
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
    };

    // New code to handle Result types
    let remote_impl = quote! {
        type RemoteOutput = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
        fn remote(#new_input_ident: #new_inp_type) -> Self::RemoteOutput {
            Box::pin(async move { 
                let (#(#input_idents),*) = #new_input_ident; 
                #remote_block
            })
        }
    };

    //println!("remote_impl: {}", remote_impl);

    /// phantom fields for generic types unused by the struct
    let phantom_fields = generics.params.iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                Some(quote! { #[allow(dead_code)] #ident: std::marker::PhantomData<#ident>, })
            },
            _ => None,
        });

    let expanded = quote! {
        #vis struct #fn_name #generics #where_clause {
            #(#phantom_fields)*
        }

        impl #generics Function<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #local_impl
            #remote_impl
        }
    };

    println!("expanded: {}", expanded);

    TokenStream::from(expanded)
}


fn check_output_type(ty: &Type) {
    // Check if the output type is Result<_, MiniModalError>
    let is_valid_result_type = if let Type::Path(type_path) = &*ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            last_segment.ident == "Result" &&
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                args.args.len() == 2 &&
                if let syn::GenericArgument::Type(Type::Path(error_type)) = &args.args[1] {
                    error_type.path.is_ident("MiniModalError")
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if !is_valid_result_type {
        panic!("Output type must be Result<_, MiniModalError>");
    }

    ()
}

pub fn impl_remote_block(
    name: &syn::Ident,
    args: Vec<syn::Ident>,
    field_types: Vec<TokenStream2>,
    output_type: &syn::Type
) -> TokenStream2 {

    quote! {
        use basemodules::MiniModalError;
        use minimodal_proto::proto::minimodal::mini_modal_client::MiniModalClient;
        use minimodal_proto::proto::minimodal::run_function_response::Result as RunFunctionResult;
        use minimodal_proto::proto::minimodal::RunFunctionRequest;
        use tonic::Request;
        use serde_json; 
        use serde::{Serialize, Deserialize};
        use minimodal_rs::utilities::serialize_inputs;
        use minimodal_rs::mount::mount_project;
        use minimodal_proto::proto::minimodal::NameAndType;

        let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
        let req = mount_project(vec![".git".to_string()])?;
        let _response = client.mount_project(req).await;

        let serialized_inputs = serialize_inputs(
            &[#(stringify!(#args)),*], 
            &[#(&(#args) as &dyn erased_serde::Serialize),*]
        )?;

        let request = Request::new(RunFunctionRequest {
            function_id: stringify!(#name).to_string(),
            serialized_inputs: serialized_inputs,
            field_types: vec![#(#field_types),*],
            output_type: stringify!(#output_type).to_string()
        });

        println!("request: {:?}", request);

        let response = client.run_function(request).await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?
            .get_ref().result.clone();

        match response {
            Some(RunFunctionResult::Success(success)) => {
                println!("recieved success: {}", success);
                let result: #output_type = match serde_json::from_str(&success) {
                    Ok(parsed) => Ok(parsed),
                    Err(e) => {
                        println!("Error parsing result: {:?} got {}", e, success);
                        return Err(MiniModalError::SerializationError(e.to_string()));
                    }
                };
                println!("Parsed result: {:?}", result);
                result
            }
            Some(RunFunctionResult::Error(error)) => {
                println!("Function failed with error: {}", error);
                Err(MiniModalError::FunctionError(error))?
            }
            None => {
                println!("No result received");
                Err(MiniModalError::OtherError("No result received".to_string()))?
            }
        }
    }.into()

}