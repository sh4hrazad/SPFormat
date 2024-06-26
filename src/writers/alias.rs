use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::write_old_type,
    functions::{write_argument_declarations, write_function_visibility},
    prev_sibling_kind,
    statements::{write_block, write_statement},
    variables::write_type,
    write_dimension, write_node, Writer,
};

pub fn write_alias_declaration(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.after_function_decl).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_")
        && prev_kind != ""
        && prev_kind != "comment"
        && prev_kind != "alias_declaration"
    {
        // Insert new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "function_visibility" => write_function_visibility(&child, writer)?,
            "type" => write_type(&child, writer)?,
            "old_type" => write_old_type(&child, writer)?,
            "dimension" => write_dimension(&child, writer, true)?,
            "alias_operator" | "operator" => write_node(&child, writer)?,
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            "block" => {
                if writer.settings.brace_wrapping.before_function {
                    writer.write_ln();
                    write_block(&child, writer, true)?;
                } else {
                    writer.write(' ');
                    write_block(&child, writer, false)?;
                }
            }
            _ => {
                if writer.is_statement(&kind) {
                    write_statement(&child, writer, false, false)?;
                } else {
                    println!("Unexpected kind {} in write_alias_declaration.", kind);
                }
            }
        }
    }
    writer.write_ln();

    Ok(())
}

pub fn write_alias_assignment(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_")
        && prev_kind != ""
        && prev_kind != "comment"
        && prev_kind != "alias_assignment"
    {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "function_definition_type" => write_function_visibility(&child, writer)?,
            "type" => write_type(&child, writer)?,
            "old_type" => write_old_type(&child, writer)?,
            "identifier" => write_node(&child, writer)?,
            "dimension" => write_dimension(&child, writer, true)?,
            "=" => writer.write_str(" = "),
            "alias_operator" | "operator" => write_node(&child, writer)?,
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            ";" => writer.write(';'),
            _ => {
                if writer.is_statement(&kind) {
                    write_statement(&child, writer, false, false)?;
                } else {
                    println!("Unexpected kind {} in write_alias_declaration.", kind);
                }
            }
        }
    }
    writer.write_ln();

    Ok(())
}
