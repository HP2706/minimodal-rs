use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::Ident;
use crate::macro_builder::MacroBuilder;

fn generate_local_impl(
    is_async: bool,
    macro_builder: &MacroBuilder,
) -> TokenStream2 {

    let MacroBuilder {
        new_inp_type, 
        output_type, 
        input_idents, 
        block,
        ..
    } = macro_builder;
    
    let new_input_ident = generate_new_input_ident(&input_idents);
    
    // Add this block to check the return type

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
    macro_builder: &MacroBuilder,
) -> TokenStream2 {

    let new_input_ident = generate_new_input_ident(&macro_builder.input_idents);

    let MacroBuilder { 
        fn_name, 
        new_inp_type, 
        output_type, 
        input_idents, 
        .. 
    } = macro_builder;

    let remote_block_body = quote! {
        use basemodules::MiniModalError;
        use minimodal_proto::proto::minimodal::{
            mini_modal_client::MiniModalClient, 
            run_function_response::Response, 
            RunFunctionResponse,
            RunFunctionRequest
        };
        use serde_json;
        use tonic::{Request, Status, Streaming};
        use serde_json;
        use minimodal_rs::utilities::serialize_inputs;
        use minimodal_rs::mount::mount_project;
        use minimodal_proto::proto::minimodal::NameAndType;

        let mut client = MiniModalClient::connect("http://[::1]:50051").await?;
        
        mount_project(&mut client, vec![".git".to_string()])
            .await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?;



        let request  = Request::new(RunFunctionRequest {
            function_id: stringify!(#fn_name).to_string(),
            serialized_inputs: "".to_string(),
            field_types: vec![],
            output_type: stringify!(#output_type).to_string()
        });

        let mut response_stream : Streaming<RunFunctionResponse> = 
            client.run_function(request).await
            .map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?
            .into_inner();

        while let Some(response) = response_stream.next().await {
            let response = response.map_err(|e| MiniModalError::from(anyhow::Error::from(e)))?;
            match response.response {
                Some(Response::Result(task_result)) => {
                    if task_result.success {
                        serde_json::from_str(&task_result.message)
                            .map_err(|e| MiniModalError::SerializationError(e.to_string()))
                    } else {
                        return Err(MiniModalError::FunctionError(task_result.message));
                    }
                }
                Some(Response::LogLine(line)) => {
                    println!("log: {}", line);
                }
                None => {
                    return Err(MiniModalError::OtherError("No result received".to_string()));
                }
            }
        }
        Err(MiniModalError::OtherError("Stream ended without result".to_string()))
    };

    quote! {
        type RemoteOutput = Pin<Box<dyn Future<Output = #output_type> + Send + 'static>>;
        fn remote(#new_input_ident: #new_inp_type) -> Self::RemoteOutput {
            Box::pin(async move { 
                let (#(#input_idents),*) = #new_input_ident; 
                #remote_block_body
            })
        }
    }
}

pub fn impl_function_trait(
    is_async: bool,
    macro_builder: &MacroBuilder,
) -> TokenStream2 {

    let MacroBuilder {
        fn_name, 
        generics, 
        where_clause, 
        new_inp_type, 
        output_type, 
        ..
    } = macro_builder;

    let remote_impl = generate_remote_impl(&macro_builder);
    let local_impl = generate_local_impl(is_async, &macro_builder);

    quote! {
        impl #generics Function<#new_inp_type, #output_type> for #fn_name #generics #where_clause {
            #local_impl
            #remote_impl
        }
    }.into()
}