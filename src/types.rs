#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonItemType {
    Null,
    False,
    True,
    String,
    Number,
    Object,
    Array,
    BlankLine,
    LineComment,
    BlockComment,
}

impl JsonItemType {
    /// Returns `true` if this is a comment type (line or block).
    #[inline]
    pub fn is_comment(self) -> bool {
        matches!(self, Self::LineComment | Self::BlockComment)
    }

    /// Returns `true` if this is a comment or blank line type.
    #[inline]
    pub fn is_comment_or_blank(self) -> bool {
        matches!(
            self,
            Self::LineComment | Self::BlockComment | Self::BlankLine
        )
    }

    /// Returns `true` if this is a structural container type (object or array).
    #[inline]
    pub fn is_structural(self) -> bool {
        matches!(self, Self::Object | Self::Array)
    }

    /// Returns `true` if this is a JSON value type.
    #[inline]
    pub fn is_value(self) -> bool {
        matches!(
            self,
            Self::String | Self::Number | Self::True | Self::False | Self::Null
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InputPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonItem {
    pub item_type: JsonItemType,
    pub input_position: InputPosition,
    pub name: String,
    pub value: String,
    pub prefix_comment: Option<String>,
    pub middle_comment: Option<String>,
    pub postfix_comment: Option<String>,
    pub is_post_comment_line_style: bool,
    pub children: Vec<JsonItem>,
}

impl JsonItem {
    pub fn new(item_type: JsonItemType) -> Self {
        JsonItem {
            item_type,
            input_position: InputPosition::default(),
            name: String::new(),
            value: String::new(),
            prefix_comment: None,
            middle_comment: None,
            postfix_comment: None,
            is_post_comment_line_style: false,
            children: Vec::new(),
        }
    }

    /// Create a JsonItem with a string value.
    pub fn with_value(mut self, value: String) -> Self {
        self.value = value;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn has_comments(&self) -> bool {
        self.prefix_comment.is_some()
            || self.middle_comment.is_some()
            || self.postfix_comment.is_some()
    }

    pub fn is_empty(&self) -> bool {
        match self.item_type {
            JsonItemType::Object | JsonItemType::Array => self.children.is_empty(),
            _ => false,
        }
    }
}
