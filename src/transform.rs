use crate::types::{JsonItem, JsonItemType};
use jsonc_parser::cst::*;

pub fn transform(cst: &CstRootNode) -> JsonItem {
    match cst.root_value() {
        Some(root_node) => transform_node(&root_node),
        None => JsonItem::new(JsonItemType::Null),
    }
}

fn transform_node(node: &CstNode) -> JsonItem {
    match node {
        CstNode::Leaf(leaf) => transform_leaf(leaf),
        CstNode::Container(container) => transform_container(container),
    }
}

fn transform_leaf(leaf: &CstLeafNode) -> JsonItem {
    match leaf {
        CstLeafNode::StringLit(s) => {
            // Decoded strings need to be owned (escape sequences processed)
            let value = s.decoded_value().unwrap_or_default();
            JsonItem::new(JsonItemType::String).with_value(value)
        }
        CstLeafNode::NumberLit(n) => {
            // Numbers must use Display - jsonc-parser doesn't expose raw text access
            JsonItem::new(JsonItemType::Number).with_value(n.to_string())
        }
        CstLeafNode::BooleanLit(b) => {
            // Use static strings for literals
            let value = if b.value() { "true" } else { "false" };
            JsonItem::new(if b.value() {
                JsonItemType::True
            } else {
                JsonItemType::False
            })
            .with_value(value.to_string())
        }
        CstLeafNode::NullKeyword(_) => {
            // Use static string for null
            JsonItem::new(JsonItemType::Null).with_value("null".to_string())
        }
        CstLeafNode::Comment(c) => {
            // Comments need owned strings (from CST Display)
            let comment_text = c.to_string();
            let is_line_comment = comment_text.starts_with("//");
            let item_type = if is_line_comment {
                JsonItemType::LineComment
            } else {
                JsonItemType::BlockComment
            };
            JsonItem::new(item_type).with_value(comment_text)
        }
        // Skip whitespace, newlines
        CstLeafNode::Whitespace(_) | CstLeafNode::Newline(_) => {
            JsonItem::new(JsonItemType::BlankLine)
        }
        _ => JsonItem::new(JsonItemType::Null),
    }
}

fn transform_container(container: &CstContainerNode) -> JsonItem {
    match container {
        CstContainerNode::Array(array) => transform_array(array),
        CstContainerNode::Object(object) => transform_object(object),
        _ => JsonItem::new(JsonItemType::Null),
    }
}

fn transform_array(array: &CstArray) -> JsonItem {
    let mut json_item = JsonItem::new(JsonItemType::Array);
    let mut pending_prefix_comment: Option<String> = None;
    let mut pending_prefix_is_line_style: Option<bool> = None;

    for child in array.children() {
        // Skip tokens (brackets, commas, colons, etc.)
        if child.is_token() {
            continue;
        }

        // Skip whitespace and newlines (but not comments)
        if child.is_whitespace() || child.is_newline() {
            continue;
        }

        match child {
            CstNode::Leaf(leaf) => {
                match leaf {
                    CstLeafNode::Comment(c) => {
                        let comment_text = c.to_string();
                        let is_line_comment = comment_text.starts_with("//");

                        // Store as pending prefix comment for next element
                        pending_prefix_comment = Some(comment_text);
                        pending_prefix_is_line_style = Some(is_line_comment);
                    }
                    _ => {
                        let mut item = transform_leaf(&leaf);

                        // Attach pending prefix comment if available
                        attach_pending_comment(
                            &mut item,
                            &mut pending_prefix_comment,
                            &mut pending_prefix_is_line_style,
                        );

                        json_item.children.push(item);
                    }
                }
            }
            CstNode::Container(container) => {
                let mut item = transform_container(&container);

                // Attach pending prefix comment if available
                attach_pending_comment(
                    &mut item,
                    &mut pending_prefix_comment,
                    &mut pending_prefix_is_line_style,
                );

                json_item.children.push(item);
            }
        }
    }

    // Handle standalone comments at the end of array
    // (But skip if they're just whitespace/comment artifacts with no value)
    let has_non_comment_items = json_item
        .children
        .iter()
        .any(|c| !c.item_type.is_comment_or_blank());
    if has_non_comment_items || json_item.children.is_empty() {
        if let Some(comment) = pending_prefix_comment.take() {
            let item_type = if comment.starts_with("//") {
                JsonItemType::LineComment
            } else {
                JsonItemType::BlockComment
            };
            let comment_item = JsonItem::new(item_type).with_value(comment);
            json_item.children.push(comment_item);
        }
    }

    json_item
}

fn transform_object(object: &CstObject) -> JsonItem {
    let mut json_item = JsonItem::new(JsonItemType::Object);

    // Transform all properties and extract comments from within ObjectProp
    for prop in object.properties() {
        let name = prop
            .name()
            .and_then(|n| n.decoded_value().ok())
            .unwrap_or_default();
        let mut value = match prop.value() {
            Some(v) => transform_node(&v),
            None => JsonItem::new(JsonItemType::Null),
        };

        // Extract comments from ObjectProp children (middle and postfix comments)
        let mut found_middle_comment = false;
        for prop_child in prop.children() {
            // Skip tokens and whitespace
            if prop_child.is_token() || prop_child.is_whitespace() || prop_child.is_newline() {
                continue;
            }

            // Check for comments within the property
            if let Some(c) = as_comment(&prop_child) {
                let comment_text = c.to_string();
                let is_line = comment_text.starts_with("//");
                // If we haven't found value yet, this is a middle comment
                // Otherwise, it's a postfix comment
                if !found_middle_comment {
                    value.middle_comment = Some(comment_text);
                    found_middle_comment = true;
                } else {
                    value.postfix_comment = Some(comment_text);
                    value.is_post_comment_line_style = is_line;
                }
            }
        }

        // Only add property if it has a name
        if !name.is_empty() {
            json_item.children.push(value.with_name(name));
        }
    }

    // Check if object has any properties
    let has_properties = !json_item.children.is_empty();

    // Now, iterate over children to find and attach prefix/postfix comments
    let mut pending_prefix_comment: Option<String> = None;
    let mut pending_prefix_is_line_style: Option<bool> = None;
    let mut current_prop_index: usize = 0;

    for child in object.children() {
        // Skip whitespace and newlines
        if child.is_whitespace() || child.is_newline() {
            continue;
        }

        // Check for comment
        if let Some(c) = as_comment(&child) {
            let comment_text = c.to_string();
            let is_line_comment = comment_text.starts_with("//");

            // If we have properties, treat as prefix comment for next property
            if has_properties {
                pending_prefix_comment = Some(comment_text);
                pending_prefix_is_line_style = Some(is_line_comment);
            } else {
                // No properties - preserve as standalone comment
                let item_type = if comment_text.starts_with("//") {
                    JsonItemType::LineComment
                } else {
                    JsonItemType::BlockComment
                };
                let comment_item = JsonItem::new(item_type).with_value(comment_text);
                json_item.children.push(comment_item);
            }
        }

        // Check if it's an ObjectProp - marks a property, attach any pending comment
        if let CstNode::Container(CstContainerNode::ObjectProp(_)) = child {
            if current_prop_index < json_item.children.len() {
                attach_pending_comment(
                    &mut json_item.children[current_prop_index],
                    &mut pending_prefix_comment,
                    &mut pending_prefix_is_line_style,
                );
            }
            current_prop_index += 1;
        }

        // Skip tokens (after handling properties and comments)
        if child.is_token() {
            continue;
        }
    }

    json_item
}

fn attach_pending_comment(
    item: &mut JsonItem,
    pending_prefix_comment: &mut Option<String>,
    pending_prefix_is_line_style: &mut Option<bool>,
) {
    if let Some(comment) = pending_prefix_comment.take() {
        item.prefix_comment = Some(comment);
        if let Some(is_line) = pending_prefix_is_line_style.take() {
            item.is_post_comment_line_style = is_line;
        }
    }
}

fn as_comment(node: &CstNode) -> Option<&CstComment> {
    match node {
        CstNode::Leaf(CstLeafNode::Comment(c)) => Some(c),
        _ => None,
    }
}
