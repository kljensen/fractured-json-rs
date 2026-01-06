#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EolStyle {
    Lf,
    Crlf,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableCommaPlacement {
    EndOfLine,
    NextLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberListAlignment {
    None,
    Left,
    Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPolicy {
    Preserve,
    Remove,
}

#[derive(Debug, Clone)]
pub struct FracturedJsonOptions {
    pub json_eol_style: EolStyle,
    pub max_total_line_length: usize,
    pub max_inline_complexity: u32,
    pub max_compact_array_complexity: u32,
    pub max_table_row_complexity: u32,
    pub max_prop_name_padding: usize,
    pub colon_before_prop_name_padding: bool,
    pub table_comma_placement: TableCommaPlacement,
    pub min_compact_array_row_items: usize,
    pub always_expand_depth: i32,
    pub nested_bracket_padding: bool,
    pub simple_bracket_padding: bool,
    pub colon_padding: bool,
    pub comma_padding: bool,
    pub comment_padding: bool,
    pub number_list_alignment: NumberListAlignment,
    pub indent_spaces: usize,
    pub use_tab_to_indent: bool,
    pub prefix_string: String,
    pub comment_policy: CommentPolicy,
    pub preserve_blank_lines: bool,
    pub allow_trailing_commas: bool,
}

impl Default for FracturedJsonOptions {
    fn default() -> Self {
        FracturedJsonOptions {
            json_eol_style: EolStyle::Default,
            max_total_line_length: 120,
            max_inline_complexity: 1,
            max_compact_array_complexity: 2,
            max_table_row_complexity: 1,
            max_prop_name_padding: 40,
            colon_before_prop_name_padding: false,
            table_comma_placement: TableCommaPlacement::EndOfLine,
            min_compact_array_row_items: 4,
            always_expand_depth: 0,
            nested_bracket_padding: true,
            simple_bracket_padding: true,
            colon_padding: true,
            comma_padding: true,
            comment_padding: true,
            number_list_alignment: NumberListAlignment::None,
            indent_spaces: 4,
            use_tab_to_indent: false,
            prefix_string: String::new(),
            comment_policy: CommentPolicy::Preserve,
            preserve_blank_lines: true,
            allow_trailing_commas: false,
        }
    }
}

impl FracturedJsonOptions {
    pub fn eol_string(&self) -> &'static str {
        match self.json_eol_style {
            EolStyle::Lf => "\n",
            EolStyle::Crlf => "\r\n",
            EolStyle::Default => "\n",
        }
    }
}
