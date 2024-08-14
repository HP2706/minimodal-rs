

pub fn extract_left_type(return_type: String) -> syn::Type {
    let parsed_type = syn::parse_str::<syn::Type>(&return_type)
        .expect(&format!("Failed to parse return type: {}", return_type));
    
    if let syn::Type::Path(type_path) = &parsed_type {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(left_type)) = args.args.first() {
                        return left_type.clone();
                    }
                }
            }
        }
    }
    
    parsed_type
}
