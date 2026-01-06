use clap::Parser;
use fractured_json_rs::{
    format_jsonc, CommentPolicy, EolStyle, FracturedJsonOptions, NumberListAlignment,
    TableCommaPlacement,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fractured-json")]
#[command(about = "A human-friendly JSONC formatter", long_about = None)]
struct Cli {
    #[arg(short, long)]
    input: Option<PathBuf>,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short = 'j', long = "json")]
    is_json: bool,

    #[arg(long, default_value = "120")]
    max_line_length: usize,

    #[arg(long, default_value = "4")]
    indent: String,

    #[arg(long, default_value = "preserve")]
    comment_policy: String,

    #[arg(long, default_value = "1")]
    max_inline_complexity: u32,

    #[arg(long, default_value = "2")]
    max_compact_array_complexity: u32,

    #[arg(long, default_value = "1")]
    max_table_row_complexity: u32,

    #[arg(long, default_value = "40")]
    max_prop_name_padding: usize,

    #[arg(long, default_value = "false")]
    colon_before_prop_name_padding: bool,

    #[arg(long, default_value = "end-of-line")]
    table_comma_placement: String,

    #[arg(long, default_value = "4")]
    min_compact_array_row_items: usize,

    #[arg(long, default_value = "0")]
    always_expand_depth: i32,

    #[arg(long, default_value = "false")]
    no_nested_bracket_padding: bool,

    #[arg(long, default_value = "false")]
    no_simple_bracket_padding: bool,

    #[arg(long, default_value = "false")]
    no_colon_padding: bool,

    #[arg(long, default_value = "false")]
    no_comma_padding: bool,

    #[arg(long, default_value = "false")]
    no_comment_padding: bool,

    #[arg(long, default_value = "none")]
    number_list_alignment: String,

    #[arg(long, default_value = "")]
    prefix_string: String,

    #[arg(long, default_value = "false")]
    no_preserve_blank_lines: bool,

    #[arg(long, default_value = "false")]
    allow_trailing_commas: bool,

    #[arg(long, default_value = "false")]
    check: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut input = String::new();
    if let Some(input_path) = &cli.input {
        input = fs::read_to_string(input_path)?;
    } else {
        io::stdin().read_to_string(&mut input)?;
    }

    let mut options = FracturedJsonOptions {
        max_total_line_length: cli.max_line_length,
        max_inline_complexity: cli.max_inline_complexity,
        max_compact_array_complexity: cli.max_compact_array_complexity,
        max_table_row_complexity: cli.max_table_row_complexity,
        max_prop_name_padding: cli.max_prop_name_padding,
        colon_before_prop_name_padding: cli.colon_before_prop_name_padding,
        min_compact_array_row_items: cli.min_compact_array_row_items,
        always_expand_depth: cli.always_expand_depth,
        nested_bracket_padding: !cli.no_nested_bracket_padding,
        simple_bracket_padding: !cli.no_simple_bracket_padding,
        colon_padding: !cli.no_colon_padding,
        comma_padding: !cli.no_comma_padding,
        comment_padding: !cli.no_comment_padding,
        prefix_string: cli.prefix_string,
        allow_trailing_commas: cli.allow_trailing_commas,
        ..FracturedJsonOptions::default()
    };

    if cli.indent == "tab" {
        options.use_tab_to_indent = true;
    } else if let Ok(spaces) = cli.indent.parse::<usize>() {
        options.indent_spaces = spaces;
        options.use_tab_to_indent = false;
    }

    if cli.comment_policy == "remove" {
        options.comment_policy = CommentPolicy::Remove;
    } else {
        options.comment_policy = CommentPolicy::Preserve;
    }

    match cli.table_comma_placement.as_str() {
        "next-line" => {
            options.table_comma_placement = TableCommaPlacement::NextLine;
        }
        _ => {
            options.table_comma_placement = TableCommaPlacement::EndOfLine;
        }
    }

    match cli.number_list_alignment.as_str() {
        "left" => {
            options.number_list_alignment = NumberListAlignment::Left;
        }
        "decimal" => {
            options.number_list_alignment = NumberListAlignment::Decimal;
        }
        _ => {
            options.number_list_alignment = NumberListAlignment::None;
        }
    }

    if cli.no_preserve_blank_lines {
        options.preserve_blank_lines = false;
    }

    if cli.is_json {
        options.json_eol_style = EolStyle::Lf;
        options.comment_policy = CommentPolicy::Remove;
    }

    let output = format_jsonc(&input, &options)?;

    if cli.check {
        if input.trim() == output.trim() {
            println!("Formatted correctly");
            Ok(())
        } else {
            eprintln!("File needs formatting");
            std::process::exit(1);
        }
    } else {
        if let Some(output_path) = &cli.output {
            fs::write(output_path, output)?;
        } else {
            io::stdout().write_all(output.as_bytes())?;
        }
        Ok(())
    }
}
