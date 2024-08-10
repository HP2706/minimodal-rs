// ... existing imports ...

// Add these imports
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn remote_execute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;

    let expanded = quote! {
        async fn #fn_name() -> Result<_, Box<dyn std::error::Error>> {
            let client = RemoteExecutorClient::connect("http://[::1]:50051").await?;
            let serialized_fn = serde_closure::serialize(&|| async #fn_block)?;
            
            let request = Request::new(ExecuteRequest {
                function: serialized_fn,
            });

            let response = client.execute(request).await?;
            let result = serde_json::from_str(&response.into_inner().result)?;
            
            Ok(result)
        }
    };

    TokenStream::from(expanded)
}

// ... rest of the file ...