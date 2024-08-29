use syn::Type;

// Add these helper functions
pub fn infer_block_return_type(block: &syn::Block) -> Type {
    // This is a simplified version. You might need a more sophisticated
    // analysis for complex blocks.
    if let Some(syn::Stmt::Expr(expr, _)) = block.stmts.last() {
        expr_to_type(expr)
    } else {
        syn::parse_quote!(())
    }
}

pub fn expr_to_type(expr: &syn::Expr) -> Type {
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

pub fn types_match(declared: &Type, actual: &Type) -> bool {
    // This is a simplified comparison. You might need to handle more cases.
    format!("{:?}", declared) == format!("{:?}", actual)
}