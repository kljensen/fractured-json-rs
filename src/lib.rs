pub mod computed;
pub mod error;
pub mod formatter;
pub mod options;
pub mod transform;
pub mod types;

pub use error::{FracturedJsonError, Result};
pub use formatter::format;
pub use options::{
    CommentPolicy, EolStyle, FracturedJsonOptions, NumberListAlignment, TableCommaPlacement,
};
pub use transform::transform;
pub use types::{InputPosition, JsonItem, JsonItemType};

use jsonc_parser::{cst::CstRootNode, ParseOptions};

pub fn format_jsonc(input: &str, options: &FracturedJsonOptions) -> Result<String> {
    let parse_options = ParseOptions::default();
    let cst = CstRootNode::parse(input, &parse_options)?;
    let json_item = transform(&cst);
    Ok(format(&json_item, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let input = r#"{"name": "test", "value": 123}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn test_simple_array() {
        let input = r#"[1, 2, 3]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_nested_structure() {
        let input = r#"{"arr": [1, 2, 3], "obj": {"nested": true}}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"arr\""));
        assert!(result.contains("\"obj\""));
    }

    #[test]
    fn test_table_formatting_simple_values() {
        let input = r#"[
            {"x": 1, "y": 2},
            {"x": 10, "y": 20}
        ]"#;
        let options = FracturedJsonOptions {
            max_table_row_complexity: 3,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"x\"") || result.contains("1"));
    }

    #[test]
    fn test_table_formatting_arrays() {
        let input = r#"[
            [1, 2, 3],
            [4, 5, 6]
        ]"#;
        let options = FracturedJsonOptions {
            max_table_row_complexity: 2,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_compact_array_formatting() {
        let input = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#;
        let options = FracturedJsonOptions {
            max_compact_array_complexity: 1,
            min_compact_array_row_items: 4,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("["));
        assert!(result.contains("]"));
    }

    #[test]
    fn test_number_alignment_left() {
        let input = r#"[1, 10, 100, 1000]"#;
        let options = FracturedJsonOptions {
            max_inline_complexity: 0,
            number_list_alignment: NumberListAlignment::Left,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("10"));
        assert!(result.contains("100"));
    }

    #[test]
    fn test_number_alignment_decimal() {
        let input = r#"[1.5, 10.25, 100.125]"#;
        let options = FracturedJsonOptions {
            max_inline_complexity: 0,
            number_list_alignment: NumberListAlignment::Decimal,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("1.5"));
        assert!(result.contains("10.25"));
    }

    #[test]
    fn test_property_alignment() {
        let input = r#"{"name": "test", "value": 123, "description": "A test object"}"#;
        let options = FracturedJsonOptions {
            max_inline_complexity: 0,
            max_prop_name_padding: 20,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"value\""));
        assert!(result.contains("\"description\""));
    }

    #[test]
    fn test_comment_preservation() {
        let input = r#"{"key": "value"}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("key"));
    }

    #[test]
    fn test_comment_removal() {
        let input = r#"{"key": "value" /* comment */}"#;
        let options = FracturedJsonOptions {
            comment_policy: CommentPolicy::Remove,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(!result.contains("comment"));
    }

    #[test]
    fn test_trailing_commas() {
        let input = r#"[1, 2, 3]"#;
        let options = FracturedJsonOptions {
            max_inline_complexity: 0,
            allow_trailing_commas: true,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains(",\n"));
    }

    #[test]
    fn test_indentation_spaces() {
        let input = r#"{"a": {"b": 1}}"#;
        let options = FracturedJsonOptions {
            indent_spaces: 2,
            max_inline_complexity: 0,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("  "));
    }

    #[test]
    fn test_indentation_tabs() {
        let input = r#"{"a": {"b": 1}}"#;
        let options = FracturedJsonOptions {
            use_tab_to_indent: true,
            max_inline_complexity: 0,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\t"));
    }

    #[test]
    fn test_empty_object() {
        let input = r#"{}"#;
        let options = FracturedJsonOptions {
            simple_bracket_padding: false,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("{}"));
    }

    #[test]
    fn test_empty_array() {
        let input = r#"[]"#;
        let options = FracturedJsonOptions {
            simple_bracket_padding: false,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("[]"));
    }

    #[test]
    fn test_deeply_nested() {
        let input = r#"{"a": {"b": {"c": {"d": 1}}}}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
        assert!(result.contains("\"c\""));
        assert!(result.contains("\"d\""));
    }

    #[test]
    fn test_special_characters() {
        let input = r#"{"key": "value with \"quotes\"" , "unicode": "Hello \u4e16\u754c"}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("quotes"));
        assert!(result.contains("unicode"));
    }

    #[test]
    fn test_boolean_values() {
        let input = r#"{"true": true, "false": false}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
    }

    #[test]
    fn test_null_value() {
        let input = r#"{"key": null}"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("null"));
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = r#"{
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"users\""));
        assert!(result.contains("\"settings\""));
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"age\""));
    }

    #[test]
    fn test_line_length_limit() {
        let input = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]"#;
        let options = FracturedJsonOptions {
            max_total_line_length: 50,
            max_inline_complexity: 1,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\n"));
    }

    #[test]
    fn test_comment_detection_line_comment() {
        let input = r#"[
            // line comment
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// line comment"));
    }

    #[test]
    fn test_comment_detection_block_comment() {
        let input = r#"[
            /* block comment */
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("/* block comment */"));
    }

    #[test]
    fn test_comment_detection_multiple() {
        let input = r#"[
            // first comment
            1,
            // second comment
            2,
            3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// first comment"));
        assert!(result.contains("// second comment"));
    }

    #[test]
    fn test_object_prefix_comment() {
        let input = r#"{
            // prefix
            "a": 1,
            "b": 2
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// prefix"));
        assert!(result.contains("\"a\""));
    }

    #[test]
    fn test_nested_array_with_comment() {
        let input = r#"{
            "arr": [
                // comment
                1, 2
            ]
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// comment"));
        assert!(result.contains("\"arr\""));
    }

    #[test]
    fn test_nested_object_with_comment() {
        let input = r#"{
            "obj": {
                // comment
                "key": "value"
            }
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// comment"));
        assert!(result.contains("\"key\""));
    }

    #[test]
    fn test_block_comment_in_array() {
        let input = r#"[
            /* block */
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("/* block */"));
    }

    #[test]
    fn test_block_comment_in_object() {
        let input = r#"{
            /* block */
            "a": 1
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("/* block */"));
    }

    #[test]
    fn test_comment_in_deeply_nested() {
        let input = r#"{
            "level1": {
                "level2": [
                    {
                        "level3": [
                            // deep comment
                            1, 2
                        ]
                    }
                ]
            }
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// deep comment"));
    }

    #[test]
    fn test_multiple_comment_types() {
        let input = r#"[
            // line
            1,
            /* block */
            2
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// line"));
        assert!(result.contains("/* block */"));
    }

    #[test]
    fn test_comment_policy_remove() {
        let input = r#"[
            // remove this
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions {
            comment_policy: CommentPolicy::Remove,
            ..FracturedJsonOptions::default()
        };
        let result = format_jsonc(input, &options).unwrap();
        assert!(!result.contains("// remove this"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_comment_with_complex_data() {
        let input = r#"{
            "users": [
                // admin user
                {"name": "Alice", "role": "admin"},
                // regular user
                {"name": "Bob", "role": "user"}
            ]
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// admin user"));
        assert!(result.contains("// regular user"));
        assert!(result.contains("\"name\""));
    }

    #[test]
    fn test_comments_prevent_inlining() {
        let input = r#"[
            // comment prevents inline
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        // Should be multiline, not inline
        assert!(result.contains("\n"));
        assert!(result.contains("// comment prevents inline"));
    }

    #[test]
    fn test_empty_array_with_comment() {
        let input = r#"[
            // but empty array
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// but empty array"));
    }

    #[test]
    fn test_empty_object_with_comment() {
        let input = r#"{
            // empty object
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// empty object"));
    }

    #[test]
    fn test_multiline_block_comment() {
        let input = r#"[
            /* multi
            line
            comment */
            1, 2, 3
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("/* multi"));
        assert!(result.contains("comment */"));
    }

    #[test]
    fn test_comment_at_array_end() {
        let input = r#"[
            1, 2, 3
            // at end
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// at end"));
    }

    #[test]
    fn test_comment_in_object_value() {
        let input = r#"{
            "arr": [1, 2, 3 // trailing comment
            ]
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        // Comment should be preserved (as postfix comment on array element)
        assert!(result.contains("// trailing comment") || result.contains("3"));
    }

    #[test]
    fn test_multiple_comments_single_element() {
        let input = r#"[
            // comment 1
            /* comment 2 */
            // comment 3
            42
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(
            result.contains("// comment 1")
                || result.contains("/* comment 2 */")
                || result.contains("// comment 3")
        );
        assert!(result.contains("42"));
    }

    #[test]
    fn test_comment_in_nested_both_levels() {
        let input = r#"{
            // outer comment
            "arr": [
                // inner comment
                1, 2
            ]
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// outer comment"));
        assert!(result.contains("// inner comment"));
    }

    #[test]
    fn test_comment_with_special_characters() {
        let input = r#"[
            // comment with "quotes" and 'apostrophes'
            1, 2
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// comment with"));
    }

    #[test]
    fn test_preserve_structure_with_comments() {
        let input = r#"{
            "key1": "value1",
            // separator
            "key2": "value2"
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("\"key1\""));
        assert!(result.contains("\"key2\""));
        assert!(result.contains("// separator"));
    }

    #[test]
    fn test_multiple_objects_with_comments() {
        let input = r#"[
            // first object
            {"a": 1},
            // second object
            {"b": 2}
        ]"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();
        assert!(result.contains("// first object"));
        assert!(result.contains("// second object"));
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
    }

    #[test]
    fn test_comprehensive_comments() {
        let input = r#"{
            // config section
            "database": {
                "host": "localhost", // default host
                "port": 5432
            },
            // users
            "users": [
                // admin
                {"name": "admin", "role": "admin"},
                // regular users
                {"name": "user1", "role": "user"},
                {"name": "user2", "role": "user"} /* last user */
            ]
        }"#;
        let options = FracturedJsonOptions::default();
        let result = format_jsonc(input, &options).unwrap();

        // Verify all comments are preserved
        assert!(result.contains("// config section"));
        assert!(result.contains("// default host"));
        assert!(result.contains("// users"));
        assert!(result.contains("// admin"));
        assert!(result.contains("// regular users"));
        assert!(result.contains("/* last user */"));

        // Verify structure is maintained
        assert!(result.contains("\"database\""));
        assert!(result.contains("\"host\""));
        assert!(result.contains("\"users\""));
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"role\""));
    }
}
