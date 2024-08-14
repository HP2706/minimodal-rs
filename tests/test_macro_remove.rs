use minimodal_rs::parse_file::remove_macro;

pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_macro_comprehensive() {
        let test_cases = vec![
            (
                "Simple macro removal",
                "#[dummy_macro]\nfn test() {}\n",
                "fn test() {}\n",
                "dummy_macro"
            ),
            (
                "Macro with arguments",
                "#[dummy_macro(arg1, arg2 = \"value\")]\nfn test() {}\n",
                "fn test() {}\n",
                "dummy_macro"
            ),
            (
                "Multiple macros",
                "#[dummy_macro1]\n#[dummy_macro2]\nfn test() {}\n",
                "#[dummy_macro2]\nfn test() {}\n",
                "dummy_macro1"
            ),
            (
                "Nested macros",
                "#[outer(#[inner])]\nfn test() {}\n",
                "fn test() {}\n",
                "outer"
            ),
            (
                "Macro in string",
                "fn test() { let s = \"#[dummy_macro]\"; }\n",
                "fn test() { let s = \"#[dummy_macro]\"; }\n",
                "dummy_macro"
            ),
            (
                "Multiple occurrences",
                "#[dummy_macro]\nfn test1() {}\n#[dummy_macro]\nfn test2() {}\n",
                "fn test1() {}\nfn test2() {}\n",
                "dummy_macro"
            ),
            (
                "Non-existent macro",
                "#[existing_macro]\nfn test() {}\n",
                "#[existing_macro]\nfn test() {}\n",
                "non_existent_macro"
            ),
        ];

        for (name, input, expected, macro_name) in test_cases {
            let wrapped_input = format!("mod test_mod {{ {} }}", input);
            let wrapped_expected = format!("mod test_mod {{ {} }}", expected);
    
            let mut ast = match syn::parse_str::<syn::File>(&wrapped_input) {
                Ok(ast) => ast,
                Err(e) => panic!("Failed to parse input for case '{}': {}", name, e),
            };
            remove_macro(&mut ast, vec![macro_name.to_string()]);
            let result = prettyplease::unparse(&ast);
            
            // Normalize whitespace for comparison
            let normalized_result = normalize_whitespace(&result);
            let normalized_expected = normalize_whitespace(&wrapped_expected);
            
            assert_eq!(normalized_result, normalized_expected, "Case '{}' failed. Expected:\n{}\nGot:\n{}", name, wrapped_expected, result);
        }
    }
}