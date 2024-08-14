// for removing macro attributes and items from a string
use syn::{visit_mut::VisitMut, Item, File};

pub struct MacroRemover {
    target_macros: Vec<String>,
}

impl VisitMut for MacroRemover {
    fn visit_item_mut(&mut self, i: &mut Item) {
        // Remove macro attributes
        match i {
            Item::Fn(item_fn) => {
                item_fn.attrs.retain(|attr| !self.target_macros.iter().any(|m| attr.path().is_ident(m)));
            },
            Item::Struct(item_struct) => {
                item_struct.attrs.retain(|attr| !self.target_macros.iter().any(|m| attr.path().is_ident(m)));
            },
            Item::Enum(item_enum) => {
                item_enum.attrs.retain(|attr| !self.target_macros.iter().any(|m| attr.path().is_ident(m)));
            },
            // Add more variants as needed
            _ => {},
        }
        
        // Remove macro items (unchanged)
        if let Item::Macro(item_macro) = i {
            if self.target_macros.iter().any(|m| item_macro.mac.path.is_ident(m)) {
                *i = Item::Verbatim(syn::parse_quote!()); // Use empty tokens as tokenstream
            }
        }
        syn::visit_mut::visit_item_mut(self, i);
    }
}

// ... rest of the file remains unchanged ...
pub fn remove_macro(ast: &mut File, target_macros: Vec<String>) -> () {
    let mut remover = MacroRemover { target_macros: target_macros };
    remover.visit_file_mut(ast);
}

pub fn remove_function(ast: &mut File, function_name: &str) -> () {
    ast.items.retain(|item| {
        if let Item::Fn(item_fn) = item {
            item_fn.sig.ident != function_name
        } else {
            true
        }
    });
}