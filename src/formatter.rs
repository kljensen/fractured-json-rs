use crate::computed::ItemRef;
use crate::options::{FracturedJsonOptions, NumberListAlignment, TableCommaPlacement};
use crate::types::{JsonItem, JsonItemType};
use std::borrow::Cow;

pub fn format(item: &JsonItem, options: &FracturedJsonOptions) -> String {
    // Build computed tree with references (no clone!)
    let computed = ItemRef::from_root(item, options);

    // Estimate capacity: minimum length + 20% for indentation/line breaks + base buffer
    let estimated_capacity =
        computed.minimum_total_length() + computed.minimum_total_length() / 5 + 100;
    let mut buffer = String::with_capacity(estimated_capacity);
    format_item(&computed, options, 0, &mut buffer);

    buffer
}

fn format_item(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    match item.item_type() {
        JsonItemType::Array => {
            if should_inline(item, options, indent) {
                format_inline_array(item, options, indent, buffer);
            } else if should_format_as_compact_array(item, options, indent) {
                format_compact_array(item, options, indent, buffer);
            } else if should_format_as_table(item, options, indent) {
                format_table_array(item, options, indent, buffer);
            } else {
                format_expanded_array(item, options, indent, buffer);
            }
        }
        JsonItemType::Object => {
            if should_inline(item, options, indent) {
                format_inline_object(item, options, indent, buffer);
            } else if should_format_as_table(item, options, indent) {
                format_table_object(item, options, indent, buffer);
            } else {
                format_expanded_object(item, options, indent, buffer);
            }
        }
        JsonItemType::String => {
            write_indent(options, indent, buffer);
            write_quotes(item.value(), buffer);
        }
        JsonItemType::Number => {
            write_indent(options, indent, buffer);
            buffer.push_str(item.value());
        }
        JsonItemType::True => {
            write_indent(options, indent, buffer);
            buffer.push_str("true");
        }
        JsonItemType::False => {
            write_indent(options, indent, buffer);
            buffer.push_str("false");
        }
        JsonItemType::Null => {
            write_indent(options, indent, buffer);
            buffer.push_str("null");
        }
        JsonItemType::LineComment => {
            if options.comment_policy == crate::options::CommentPolicy::Preserve {
                write_indent(options, indent, buffer);
                buffer.push_str(item.value());
            }
        }
        JsonItemType::BlockComment => {
            if options.comment_policy == crate::options::CommentPolicy::Preserve {
                write_indent(options, indent, buffer);
                buffer.push_str(item.value());
            }
        }
        JsonItemType::BlankLine => {
            buffer.push_str(options.eol_string());
        }
    }
}

fn should_inline(item: &ItemRef, options: &FracturedJsonOptions, indent: usize) -> bool {
    if item.requires_multiple_lines() {
        return false;
    }

    if item.has_comments() {
        return false;
    }

    if item.complexity() > options.max_inline_complexity {
        return false;
    }

    if item.minimum_total_length() + indent * options.indent_spaces > options.max_total_line_length
    {
        return false;
    }

    true
}

fn should_format_as_compact_array(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    _indent: usize,
) -> bool {
    if item.item_type() != JsonItemType::Array {
        return false;
    }

    if item.is_empty() {
        return false;
    }

    if item.complexity() > options.max_compact_array_complexity {
        return false;
    }

    if item.has_comments() {
        return false;
    }

    if item.children.len() < options.min_compact_array_row_items {
        return false;
    }

    true
}

fn should_format_as_table(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    _indent: usize,
) -> bool {
    if item.item_type() == JsonItemType::Array {
        // Don't use table format if comments are present
        if item.requires_multiple_lines() {
            return false;
        }

        if item.is_empty() {
            return false;
        }

        if item.complexity() > options.max_table_row_complexity {
            return false;
        }

        if !can_format_as_table_array(item, options) {
            return false;
        }

        return true;
    }

    if item.item_type() == JsonItemType::Object {
        if item.is_empty() {
            return false;
        }

        if item.complexity() > options.max_table_row_complexity {
            return false;
        }

        if !can_format_as_table_object(item, options) {
            return false;
        }

        return true;
    }

    false
}

fn can_format_as_table_array(item: &ItemRef, _options: &FracturedJsonOptions) -> bool {
    if item.is_empty() {
        return false;
    }

    let first_type = get_item_type_for_table(&item.children[0]);
    if first_type.is_none() {
        return false;
    }

    item.children
        .iter()
        .all(|c| get_item_type_for_table(c) == first_type)
}

fn can_format_as_table_object(item: &ItemRef, _options: &FracturedJsonOptions) -> bool {
    if item.is_empty() {
        return false;
    }

    let first_props = get_property_names(&item.children[0]);
    if first_props.is_empty() {
        return false;
    }

    item.children
        .iter()
        .all(|c| get_property_names(c).as_slice() == first_props.as_slice())
}

fn get_item_type_for_table(item: &ItemRef) -> Option<JsonItemType> {
    match item.item_type() {
        JsonItemType::Object => {
            if item.is_empty() {
                None
            } else {
                Some(JsonItemType::Object)
            }
        }
        JsonItemType::Array => {
            if item.is_empty() {
                None
            } else {
                Some(JsonItemType::Array)
            }
        }
        JsonItemType::String
        | JsonItemType::Number
        | JsonItemType::True
        | JsonItemType::False
        | JsonItemType::Null => Some(item.item_type()),
        _ => None,
    }
}

fn get_property_names<'a>(item: &'a ItemRef<'a>) -> Vec<&'a str> {
    if item.item_type() != JsonItemType::Object {
        return Vec::new();
    }
    item.children.iter().map(|c| c.name()).collect()
}

fn format_number_aligned(
    value: &str,
    options: &FracturedJsonOptions,
    other_values: &[&str],
) -> String {
    match options.number_list_alignment {
        NumberListAlignment::None => value.to_string(),
        NumberListAlignment::Left => {
            if let Some(max_len) = other_values.iter().map(|v| v.len()).max() {
                format!("{:<width$}", value, width = max_len)
            } else {
                value.to_string()
            }
        }
        NumberListAlignment::Decimal => {
            let max_integer_part = other_values
                .iter()
                .filter_map(|v| v.split('.').next())
                .map(|s| s.len())
                .max()
                .unwrap_or(value.len());

            if value.contains('.') {
                let parts: Vec<&str> = value.split('.').collect();
                if parts.len() == 2 {
                    format!(
                        "{: >width$}.{}",
                        parts[0],
                        parts[1],
                        width = max_integer_part
                    )
                } else {
                    value.to_string()
                }
            } else {
                format!("{: >width$}", value, width = max_integer_part)
            }
        }
    }
}

fn write_indent(options: &FracturedJsonOptions, indent: usize, buffer: &mut String) {
    if options.use_tab_to_indent {
        for _ in 0..indent {
            buffer.push('\t');
        }
    } else {
        write_spaces(buffer, indent * options.indent_spaces);
    }
}

/// Efficiently writes spaces to buffer without allocations.
/// Uses a pre-allocated string slice and slices it as needed.
fn write_spaces(buffer: &mut String, count: usize) {
    const SPACES: &str = "                                "; // 32 spaces
    let mut remaining = count;

    while remaining >= 32 {
        buffer.push_str(SPACES);
        remaining -= 32;
    }

    if remaining > 0 {
        buffer.push_str(&SPACES[..remaining]);
    }
}

fn write_quotes(s: &str, buffer: &mut String) {
    buffer.push('"');
    buffer.push_str(&escape_string(s));
    buffer.push('"');
}

fn write_quoted_property_name(s: &str, buffer: &mut String) {
    buffer.push('"');
    buffer.push_str(s);
    buffer.push('"');
}

fn escape_string(s: &str) -> Cow<'_, str> {
    // Check if any character needs escaping - enable zero-copy for common case
    let needs_escape = s.chars().any(|c| {
        matches!(c, '\\' | '"' | '\n' | '\t' | '\r' | '\x08' | '\x0c') || c <= '\u{001f}'
    });

    if !needs_escape {
        return Cow::Borrowed(s); // Zero-copy for strings without special chars
    }

    // Count characters that need escaping to estimate capacity more accurately
    let escape_count = s
        .chars()
        .filter(|&c| {
            matches!(c, '\\' | '"' | '\n' | '\t' | '\r' | '\x08' | '\x0c') || c <= '\u{001f}'
        })
        .count();

    // Simple escapes add 1 char (e.g., \n), unicode escapes add 5 chars (\uXXXX)
    // Estimate: original length + escapes + some buffer for unicode escapes
    let estimated_capacity = s.len() + escape_count + (escape_count / 4);
    let mut result = String::with_capacity(estimated_capacity);

    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            _ => {
                // Escape control characters (0x00-0x1F) without format! allocation
                if c <= '\u{001f}' {
                    const HEX: &[u8] = b"0123456789abcdef";
                    let code = c as usize;
                    result.push_str("\\u00");
                    result.push(HEX[(code >> 4) & 0xf] as char);
                    result.push(HEX[code & 0xf] as char);
                } else {
                    result.push(c);
                }
            }
        }
    }
    Cow::Owned(result)
}

fn write_prefix_comment(item: &ItemRef, options: &FracturedJsonOptions, buffer: &mut String) {
    if options.comment_policy != crate::options::CommentPolicy::Preserve {
        return;
    }

    if let Some(comment) = item.prefix_comment() {
        if options.comment_padding {
            buffer.push(' ');
        }
        buffer.push_str(comment);
        if options.comment_padding {
            buffer.push(' ');
        }
    }
}

fn write_middle_comment(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    if options.comment_policy != crate::options::CommentPolicy::Preserve {
        return;
    }

    if let Some(comment) = item.middle_comment() {
        buffer.push_str(options.eol_string());
        write_indent(options, indent, buffer);
        buffer.push_str(comment);
        buffer.push_str(options.eol_string());
        write_indent(options, indent, buffer);
    }
}

fn write_postfix_comment(item: &ItemRef, options: &FracturedJsonOptions, buffer: &mut String) {
    if options.comment_policy != crate::options::CommentPolicy::Preserve {
        return;
    }

    if let Some(comment) = item.postfix_comment() {
        if options.comment_padding {
            buffer.push(' ');
        }
        buffer.push_str(comment);
    }
}

fn calculate_items_per_row(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
) -> usize {
    if item.is_empty() {
        return 1;
    }

    let max_width = options.max_total_line_length - (indent + 1) * options.indent_spaces;
    let avg_item_width: usize = item
        .children
        .iter()
        .map(|c| c.minimum_total_length())
        .sum::<usize>()
        / item.children.len();

    if avg_item_width == 0 {
        return 1;
    }

    let items_per_row = max_width / avg_item_width.max(1);
    items_per_row.max(1)
}

fn format_compact_array(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('[');
    buffer.push_str(options.eol_string());

    let items_per_row = calculate_items_per_row(item, options, indent);
    let mut current_row = 0;

    for (i, child) in item.children.iter().enumerate() {
        let row_idx = i / items_per_row;

        if row_idx > current_row {
            buffer.push(',');
            buffer.push_str(options.eol_string());
            current_row = row_idx;
        }

        write_indent(options, indent + 1, buffer);

        write_prefix_comment(child, options, buffer);
        format_inline_value(child, options, 0, buffer);
        write_postfix_comment(child, options, buffer);

        if i < item.children.len() - 1 {
            buffer.push(',');
            if options.comma_padding {
                buffer.push(' ');
            }
        }

        buffer.push_str(options.eol_string());
    }

    write_indent(options, indent, buffer);
    buffer.push(']');
}

fn calculate_table_column_widths(
    items: &[&ItemRef],
    indent: usize,
    options: &FracturedJsonOptions,
) -> Vec<usize> {
    if items.is_empty() {
        return Vec::new();
    }

    let column_count = get_column_count(items[0]);
    let mut widths = vec![0; column_count];

    for item in items {
        // Calculate widths directly without allocating Vec<String>
        match item.item_type() {
            JsonItemType::Object => {
                for (col_idx, child) in item.children.iter().enumerate() {
                    let value_len = calculate_value_display_length(child) + 2; // +2 for quotes
                    let name_len = child.name_length();
                    // "name": value (simplified - doesn't account for escaping)
                    let total_len = name_len + value_len + 4; // +4 for quotes, colon, space
                    let indent_len = total_len + indent * options.indent_spaces;
                    if col_idx < widths.len() {
                        widths[col_idx] = widths[col_idx].max(indent_len);
                    }
                }
            }
            JsonItemType::Array => {
                for (col_idx, child) in item.children.iter().enumerate() {
                    let value_len = calculate_value_display_length(child) + 2; // +2 for brackets
                    let indent_len = value_len + indent * options.indent_spaces;
                    if col_idx < widths.len() {
                        widths[col_idx] = widths[col_idx].max(indent_len);
                    }
                }
            }
            _ => {
                let value_len = calculate_value_display_length(item) + 2; // +2 for quotes/brackets
                let indent_len = value_len + indent * options.indent_spaces;
                if !widths.is_empty() {
                    widths[0] = widths[0].max(indent_len);
                }
            }
        }
    }

    widths
}

/// Calculate display length without allocating (conservative estimate for strings)
fn calculate_value_display_length(item: &ItemRef) -> usize {
    match item.item_type() {
        JsonItemType::String => {
            // Estimate escaped length (most strings don't need escaping)
            // Conservative: assume 20% overhead for escaping max
            let base_len = item.value().len();
            std::cmp::min(base_len * 2, base_len + base_len / 5 + 10)
        }
        JsonItemType::Number | JsonItemType::True | JsonItemType::False | JsonItemType::Null => {
            item.value().len()
        }
        JsonItemType::Object => 2, // {}
        JsonItemType::Array => 2,  // []
        _ => 0,
    }
}

fn get_column_count(item: &ItemRef) -> usize {
    match item.item_type() {
        JsonItemType::Object => item.children.len(),
        JsonItemType::Array => item.children.len(),
        _ => 1,
    }
}

fn get_column_values(item: &ItemRef) -> Vec<String> {
    match item.item_type() {
        JsonItemType::Object => item.children.iter().map(format_value_simple).collect(),
        JsonItemType::Array => item.children.iter().map(format_value_simple).collect(),
        _ => vec![format_value_simple(item)],
    }
}

fn format_value_simple(item: &ItemRef) -> String {
    match item.item_type() {
        JsonItemType::String => {
            let escaped = escape_string(item.value());
            format!("\"{}\"", escaped)
        }
        JsonItemType::Number => item.value().to_string(),
        JsonItemType::True => "true".to_string(),
        JsonItemType::False => "false".to_string(),
        JsonItemType::Null => "null".to_string(),
        JsonItemType::Object => "{}".to_string(),
        JsonItemType::Array => "[]".to_string(),
        _ => String::new(),
    }
}

fn format_table_array(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('[');
    buffer.push_str(options.eol_string());

    let items: Vec<&ItemRef> = item.children.iter().collect();
    let column_widths = calculate_table_column_widths(&items, indent + 1, options);

    for (i, child) in items.iter().enumerate() {
        write_indent(options, indent + 1, buffer);

        let values = get_column_values(child);
        let is_last = i == items.len() - 1;

        for (col_idx, value) in values.iter().enumerate() {
            buffer.push_str(value);

            if col_idx < values.len() - 1 {
                let padding = column_widths[col_idx].saturating_sub(value.len());
                write_spaces(buffer, padding);
                buffer.push(',');
                if options.comma_padding {
                    buffer.push(' ');
                }
            }
        }

        if !is_last && options.table_comma_placement == TableCommaPlacement::EndOfLine {
            buffer.push(',');
        }

        buffer.push_str(options.eol_string());
    }

    write_indent(options, indent, buffer);
    buffer.push(']');
}

fn format_table_object(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('{');
    buffer.push_str(options.eol_string());

    let max_name_len = item
        .children
        .iter()
        .map(|c| c.name_length())
        .max()
        .unwrap_or(0);

    let padding = max_name_len.min(options.max_prop_name_padding);

    // Check if object has any non-comment items
    let has_properties = item
        .children
        .iter()
        .any(|c| !c.item_type().is_comment_or_blank());

    for (i, child) in item.children.iter().enumerate() {
        // Preserve standalone comment items when object has no properties
        if !has_properties && child.item_type().is_comment() {
            write_indent(options, indent + 1, buffer);
            format_item(child, options, indent + 1, buffer);
            buffer.push_str(options.eol_string());
            continue;
        }

        // Skip standalone comment items in objects
        if has_properties && child.item_type().is_comment_or_blank() {
            continue;
        }

        write_indent(options, indent + 1, buffer);

        write_prefix_comment(child, options, buffer);

        if options.colon_before_prop_name_padding {
            write_quoted_property_name(child.name(), buffer);
            buffer.push(':');
        } else {
            write_quoted_property_name(child.name(), buffer);
            let name_padding = padding.saturating_sub(child.name_length());
            write_spaces(buffer, name_padding);
            buffer.push(':');
        }

        if options.colon_padding {
            buffer.push(' ');
        }

        write_middle_comment(child, options, indent + 1, buffer);
        format_item(child, options, indent + 1, buffer);
        write_postfix_comment(child, options, buffer);

        let is_last = i == item.children.len() - 1;
        if !is_last || options.allow_trailing_commas {
            buffer.push(',');
            if options.comma_padding {
                buffer.push(' ');
            }
        }

        buffer.push_str(options.eol_string());
    }

    write_indent(options, indent, buffer);
    buffer.push('}');
}

fn format_inline_array(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('[');

    if !item.is_empty() {
        if options.nested_bracket_padding {
            buffer.push(' ');
        }

        for (i, child) in item.children.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
                if options.comma_padding {
                    buffer.push(' ');
                }
            }

            write_prefix_comment(child, options, buffer);
            format_inline_value(child, options, 0, buffer);
            write_postfix_comment(child, options, buffer);
        }

        if options.nested_bracket_padding {
            buffer.push(' ');
        }
    } else if options.simple_bracket_padding {
        buffer.push(' ');
    }

    buffer.push(']');
}

fn format_expanded_array(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('[');
    buffer.push_str(options.eol_string());

    let should_align_numbers = options.number_list_alignment != NumberListAlignment::None
        && item
            .children
            .iter()
            .all(|c| c.item_type() == JsonItemType::Number);

    let all_numbers: Vec<&str> = if should_align_numbers {
        item.children.iter().map(|c| c.value()).collect()
    } else {
        Vec::new()
    };

    for (i, child) in item.children.iter().enumerate() {
        write_indent(options, indent + 1, buffer);

        write_prefix_comment(child, options, buffer);

        if should_align_numbers && child.item_type() == JsonItemType::Number {
            let aligned = format_number_aligned(child.value(), options, &all_numbers);
            buffer.push_str(&aligned);
        } else {
            format_item(child, options, indent + 1, buffer);
        }

        write_postfix_comment(child, options, buffer);

        let is_last = i == item.children.len() - 1;
        if !is_last || (options.allow_trailing_commas && !item.is_empty()) {
            buffer.push(',');
            if options.comma_padding {
                buffer.push(' ');
            }
        }

        buffer.push_str(options.eol_string());
    }

    write_indent(options, indent, buffer);
    buffer.push(']');
}

fn format_inline_object(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('{');

    if !item.is_empty() {
        // Check if object has any non-comment items
        let has_properties = item
            .children
            .iter()
            .any(|c| !c.item_type().is_comment_or_blank());

        if options.nested_bracket_padding {
            buffer.push(' ');
        }

        for (i, child) in item.children.iter().enumerate() {
            // Preserve standalone comment items when object has no properties
            if !has_properties && child.item_type().is_comment() {
                if i > 0 {
                    buffer.push(',');
                    if options.comma_padding {
                        buffer.push(' ');
                    }
                }
                format_inline_value(child, options, 0, buffer);
                continue;
            }

            // Skip standalone comment items in objects
            if has_properties && child.item_type().is_comment_or_blank() {
                continue;
            }

            if i > 0 {
                buffer.push(',');
                if options.comma_padding {
                    buffer.push(' ');
                }
            }

            write_prefix_comment(child, options, buffer);
            write_quoted_property_name(child.name(), buffer);
            buffer.push(':');

            if options.colon_padding {
                buffer.push(' ');
            }

            write_middle_comment(child, options, indent, buffer);
            format_inline_value(child, options, 0, buffer);
            write_postfix_comment(child, options, buffer);
        }

        if options.nested_bracket_padding {
            buffer.push(' ');
        }
    } else if options.simple_bracket_padding {
        buffer.push(' ');
    }

    buffer.push('}');
}

fn format_expanded_object(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    write_indent(options, indent, buffer);
    buffer.push('{');
    buffer.push_str(options.eol_string());

    // Check if object has any non-comment items
    let has_properties = item
        .children
        .iter()
        .any(|c| !c.item_type().is_comment_or_blank());

    for (i, child) in item.children.iter().enumerate() {
        // Preserve standalone comment items when object has no properties
        if !has_properties && child.item_type().is_comment() {
            write_indent(options, indent + 1, buffer);
            format_item(child, options, indent + 1, buffer);
            buffer.push_str(options.eol_string());
            continue;
        }

        // Skip standalone comment items in objects
        if has_properties && child.item_type().is_comment_or_blank() {
            continue;
        }

        write_indent(options, indent + 1, buffer);

        write_prefix_comment(child, options, buffer);
        write_quoted_property_name(child.name(), buffer);
        buffer.push(':');

        if options.colon_padding {
            buffer.push(' ');
        }

        write_middle_comment(child, options, indent + 1, buffer);
        format_item(child, options, indent + 1, buffer);
        write_postfix_comment(child, options, buffer);

        let is_last = i == item.children.len() - 1;
        if !is_last || (options.allow_trailing_commas && !item.is_empty()) {
            buffer.push(',');
            if options.comma_padding {
                buffer.push(' ');
            }
        }

        buffer.push_str(options.eol_string());
    }

    write_indent(options, indent, buffer);
    buffer.push('}');
}

fn format_inline_value(
    item: &ItemRef,
    options: &FracturedJsonOptions,
    indent: usize,
    buffer: &mut String,
) {
    match item.item_type() {
        JsonItemType::Array | JsonItemType::Object => {
            format_item(item, options, indent, buffer);
        }
        JsonItemType::String => {
            write_quotes(item.value(), buffer);
        }
        JsonItemType::Number => {
            buffer.push_str(item.value());
        }
        JsonItemType::True => {
            buffer.push_str("true");
        }
        JsonItemType::False => {
            buffer.push_str("false");
        }
        JsonItemType::Null => {
            buffer.push_str("null");
        }
        JsonItemType::LineComment | JsonItemType::BlockComment => {
            if options.comment_policy == crate::options::CommentPolicy::Preserve {
                buffer.push_str(item.value());
            }
        }
        JsonItemType::BlankLine => {}
    }
}
