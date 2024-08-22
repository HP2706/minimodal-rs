#[path = "test_utils.rs"]
mod test_utils;

use minimodal_rs::utils::{declare_values_from_json, check_code_compiles};
use rstest::*;
use serde_json::json;
use uuid::Uuid;


#[rstest]
#[case::simple_types(
    json!({"int_field": 42, "string_field": "hello".to_string()}),
    vec![("int_field", "i32"), ("string_field", "String")]
)]
#[case::complex_types(
    json!({"tuple_field": [1, "two".to_string()], "hashmap_field": {"key": "value".to_string()}}),
    vec![("tuple_field", "(i32, String)"), ("hashmap_field", "std::collections::HashMap<String, String>")]
)]
#[case::nested_types(
    json!({"nested": {"vec_field": [1, 2, 3], "option_field": null}}),
    vec![("nested", "Nested")]
)]
#[case::array_types(
    json!({"array_field": [1, 2, 3, 4, 5]}),
    vec![("array_field", "[i32; 5]")]
)]
#[tokio::test]
async fn test_declare_values_from_json_compiles(
    #[case] input_json: serde_json::Value,
    #[case] type_declarations: Vec<(&str, &str)>,
) {
    let type_declarations: Vec<(String, String)> = type_declarations
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let output = declare_values_from_json(input_json, type_declarations).unwrap();

    // Generate a unique function name for each test case

    let type_def = "
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Nested {
    vec_field: Vec<i32>,
    option_field: Option<String>,
}
";

    let (compiles, error_message) = check_code_compiles(format!("{type_def}\n{output}")).unwrap();
    
    assert!(compiles, "Code failed to compile: {:?}", error_message);
}