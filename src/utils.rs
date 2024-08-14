use serde::{Serialize, Deserialize};


pub fn serialize_inputs<'a>(
    arg_names: &[&str], 
    arg_values: &[&dyn erased_serde::Serialize]
) -> Result<String, serde_json::Error> {
    use serde_json::json;
    
    let mut map = serde_json::Map::new();
    for (name, value) in arg_names.iter().zip(arg_values.iter()) {
        map.insert(name.to_string(), json!(value));
    }
    
    serde_json::to_string(&map)
}

pub fn deserialize_inputs<'a, T: Serialize + Deserialize<'a>>(
    serialized_inputs: &'a str
) -> Result<T, serde_json::Error> {
    serde_json::from_str(serialized_inputs)
}

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
