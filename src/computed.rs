use crate::options::FracturedJsonOptions;
use crate::types::{JsonItem, JsonItemType};

// Character size constants for length calculations
const QUOTES_SIZE: usize = 2;
const BRACKETS_SIZE: usize = 2;
const COLON_SIZE: usize = 1;

/// Computed values for a JsonItem (no mutation needed)
#[derive(Debug, Clone)]
pub struct ComputedItem {
    pub complexity: u32,
    pub minimum_total_length: usize,
    pub requires_multiple_lines: bool,
    pub name_length: usize,
    pub value_length: usize,
}

/// Reference to original item + its computed values and children
#[derive(Debug, Clone)]
pub struct ItemRef<'a> {
    pub item: &'a JsonItem,
    pub computed: ComputedItem,
    pub children: Vec<ItemRef<'a>>,
}

impl<'a> ItemRef<'a> {
    pub fn from_root(root: &'a JsonItem, options: &FracturedJsonOptions) -> Self {
        Self::compute_recursive(root, options)
    }

    fn compute_recursive(item: &'a JsonItem, options: &FracturedJsonOptions) -> Self {
        // Compute children first
        let children: Vec<_> = item
            .children
            .iter()
            .map(|c| Self::compute_recursive(c, options))
            .collect();

        // Compute this node's values
        let complexity = compute_complexity_impl(item, &children);
        let minimum_total_length = compute_lengths_impl(item, &children, options);
        let requires_multiple_lines = compute_requires_multiple_lines_impl(item, &children);

        let computed = ComputedItem {
            complexity,
            minimum_total_length,
            requires_multiple_lines,
            name_length: item.name.len(),
            value_length: item.value.len(),
        };

        Self {
            item,
            computed,
            children,
        }
    }

    /// Convenience accessors
    pub fn item_type(&self) -> JsonItemType {
        self.item.item_type
    }

    pub fn name(&self) -> &str {
        &self.item.name
    }

    pub fn value(&self) -> &str {
        &self.item.value
    }

    pub fn has_comments(&self) -> bool {
        self.item.has_comments()
    }

    pub fn is_empty(&self) -> bool {
        self.item.is_empty()
    }

    pub fn prefix_comment(&self) -> Option<&str> {
        self.item.prefix_comment.as_deref()
    }

    pub fn middle_comment(&self) -> Option<&str> {
        self.item.middle_comment.as_deref()
    }

    pub fn postfix_comment(&self) -> Option<&str> {
        self.item.postfix_comment.as_deref()
    }

    pub fn is_post_comment_line_style(&self) -> bool {
        self.item.is_post_comment_line_style
    }

    pub fn name_length(&self) -> usize {
        self.computed.name_length
    }

    pub fn value_length(&self) -> usize {
        self.computed.value_length
    }

    pub fn complexity(&self) -> u32 {
        self.computed.complexity
    }

    pub fn minimum_total_length(&self) -> usize {
        self.computed.minimum_total_length
    }

    pub fn requires_multiple_lines(&self) -> bool {
        self.computed.requires_multiple_lines
    }
}

// Pure functions for computation (no mutation)
fn compute_complexity_impl(item: &JsonItem, children: &[ItemRef<'_>]) -> u32 {
    match item.item_type {
        JsonItemType::Array | JsonItemType::Object => {
            children
                .iter()
                .map(|c| c.computed.complexity)
                .max()
                .unwrap_or(0)
                + 1
        }
        _ => 0,
    }
}

fn compute_lengths_impl(
    item: &JsonItem,
    children: &[ItemRef<'_>],
    options: &FracturedJsonOptions,
) -> usize {
    match item.item_type {
        JsonItemType::Array => {
            let children_len: usize = children
                .iter()
                .map(|c| c.computed.minimum_total_length)
                .sum();
            let separators = if !children.is_empty() {
                (children.len() - 1) * if options.comma_padding { 2 } else { 1 }
            } else {
                0
            };
            let brackets = if options.simple_bracket_padding { 2 } else { 0 };
            children_len + separators + brackets + BRACKETS_SIZE
        }
        JsonItemType::Object => {
            let children_len: usize = children
                .iter()
                .map(|c| {
                    c.computed.name_length
                        + COLON_SIZE
                        + QUOTES_SIZE
                        + c.computed.minimum_total_length
                })
                .sum();
            let separators = if !children.is_empty() {
                (children.len() - 1) * if options.comma_padding { 2 } else { 1 }
            } else {
                0
            };
            let brackets = if options.simple_bracket_padding { 2 } else { 0 };
            children_len + separators + brackets + BRACKETS_SIZE
        }
        JsonItemType::String
        | JsonItemType::Number
        | JsonItemType::True
        | JsonItemType::False
        | JsonItemType::Null => item.value.len(),
        JsonItemType::LineComment | JsonItemType::BlockComment => item.value.len(),
        JsonItemType::BlankLine => 0,
    }
}

fn compute_requires_multiple_lines_impl(item: &JsonItem, children: &[ItemRef<'_>]) -> bool {
    match item.item_type {
        JsonItemType::Array | JsonItemType::Object => {
            let has_comments = children.iter().any(|c| c.has_comments());
            let child_requires_multiple =
                children.iter().any(|c| c.computed.requires_multiple_lines);
            has_comments || child_requires_multiple
        }
        _ => false,
    }
}
