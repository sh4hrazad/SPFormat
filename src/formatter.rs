use std::collections::HashSet;

use tree_sitter::Language;

use crate::{
    settings::Settings,
    writers::{self, source_file::write_source_file},
};

use super::parser;

pub fn format_string_language(
    input: &String,
    language: Language,
    settings: &Settings,
) -> anyhow::Result<String> {
    let mut parser = parser::sourcepawn(&language)?;
    let parsed = parser.parse(&input, None)?.unwrap();

    if parsed.root_node().has_error() {
        // todo what's the error?
        return Err(anyhow::Error::msg("internal writer error or something"));
    }

    #[cfg(debug_assertions)]
    println!("{}", parsed.root_node().to_sexp());

    let indent_string = if settings.use_space {
        " ".repeat(settings.space_size as usize)
    } else {
        "\t".to_string()
    };

    let mut writer = writers::Writer {
        output: String::new(),
        source: input.as_bytes(),
        // language: &language,
        indent: 0,
        indent_string,
        skip: 0,
        semicolon: false,
        settings,
        statement_kinds: HashSet::from_iter(vec![
            "block",
            "variable_declaration_statement",
            "old_variable_declaration_statement",
            "for_statement",
            "while_statement",
            "do_while_statement",
            "break_statement",
            "continue_statement",
            "condition_statement",
            "switch_statement",
            "return_statement",
            "delete_statement",
            "expression_statement",
        ]),
        expression_kinds: HashSet::from_iter(vec![
            "assignment_expression",
            "function_call",
            "array_indexed_access",
            "ternary_expression",
            "field_access",
            "scope_access",
            "binary_expression",
            "unary_expression",
            "update_expression",
            "sizeof_expression",
            "new_expression",
            "view_as",
            "old_type_cast",
            "symbol",
            "parenthesized_expression",
            "this",
            "new_instance",
        ]),
        literal_kinds: HashSet::from_iter(vec![
            "int_literal",
            "float_literal",
            "char_literal",
            "string_literal",
            "concatenated_string",
            "bool_literal",
            "array_literal",
            "null",
        ]),
    };

    write_source_file(parsed.root_node(), &mut writer)?;

    Ok(writer.output)
}
